mod core;
mod io;

use crate::cartridge::{Cartridge, Mirroring};

const PATTERN_TABLE_START: u16 = 0x0000;
const PATTERN_TABLE_END: u16 = 0x1FFF;
const NAMETABLE_START: u16 = 0x2000;
const NAMETABLE_END: u16 = 0x3EFF;
const PALETTE_START: u16 = 0x3F00;
const PALETTE_END: u16 = 0x3FFF;

const PRE_RENDER_LINE: u16 = 261;
const VISIBLE_LINES: std::ops::RangeInclusive<u16> = 0..=239;

const PRE_FETCH_CYCLES: std::ops::RangeInclusive<u16> = 321..=336;
const VISIBLE_CYCLES: std::ops::RangeInclusive<u16> = 1..=256;

bitflags! {
    pub struct PPUCTRL: u8 {
        const HORZ_NAMETABLE = 0b0000_0001;
        const VERT_NAMETABLE = 0b0000_0010;
        const VRAM_INCREMENT = 0b0000_0100;
        const SPRITE_PATTERN_TABLE = 0b0000_1000;
        const BACKGROUND_PATTERN_TABLE = 0b0001_0000;
        const SPRITE_SIZE = 0b0010_0000;
        const MASTER_SLAVE = 0b0100_0000;
        const NMI_ON_VBLANK = 0b1000_0000;
    }

    pub struct PPUMASK: u8 {
        const GRAYSCALE = 0b0000_0001;
        const NO_CLIP_BACKGROUND = 0b0000_0010;
        const NO_CLIP_SPRITES = 0b0000_0100;
        const SHOW_BACKGROUND = 0b0000_1000;
        const SHOW_SPRITES = 0b0001_0000;
        const EMPH_RED = 0b0010_0000;
        const EMPH_GREEN = 0b0100_0000;
        const EMPH_BLUE = 0b1000_0000;
    }

    pub struct PPUSTATUS: u8 {
        // get bits [0, 4] from ppu open bus
        const SPRITE_OVERFLOW = 0b0010_0000;
        const SPRITE0_HIT = 0b0100_0000;
        const IS_VBLANK = 0b1000_0000;
    }
}

pub struct PPUStore {
    nametable_byte: u8,
    attribute_byte: u8,
    tile_lo_byte: u8,
    tile_hi_byte: u8,
    tile_addr: u16,
}

#[derive(Debug, Clone, Copy)]
enum SpriteEvalState {
    Clear,
    Copy,
    Evaluate,
    OverflowCheck,
    Done,
}
pub struct SpriteEvaluator {
    state: SpriteEvalState,
    n: u8,                  // Current sprite index (0-63)
    m: u8,                  // Current byte within sprite (0-3)
    sprites_found: u8,      // Number of sprites found so far (0-8)
    temp_byte: u8,          // Temporary storage for read data
    secondary_oam_addr: u8, // Address in secondary OAM (0-31)
    overflow_flag: bool,
    sprite_0_on_line: bool,
}

impl SpriteEvaluator {
    fn new() -> Self {
        Self {
            state: SpriteEvalState::Clear,
            n: 0,
            m: 0,
            sprites_found: 0,
            temp_byte: 0,
            secondary_oam_addr: 0,
            overflow_flag: false,
            sprite_0_on_line: false,
        }
    }

    fn reset_for_scanline(&mut self) {
        self.state = SpriteEvalState::Clear;
        self.n = 0;
        self.m = 0;
        self.sprites_found = 0;
        self.temp_byte = 0;
        self.secondary_oam_addr = 0;
        self.overflow_flag = false;
        self.sprite_0_on_line = false;
    }
}

pub struct PPU {
    // Architectural state
    v: u16, // 15 bits: yyy NN YYYYY XXXXX (fine y, nametable select, coarse y, coarse x)
    t: u16, // 15 bits (temporary): contents eventually transferred to v
    x: u8,  // 3 bits
    w: u8,  // 1 bit
    f: u8,  // 1 bit
    chr_rom: Vec<u8>,
    vram: [u8; 0x800],
    palette: [u8; 0x20],
    pub oam: [u8; 64 * 4], // 64 sprites of size 4 bytes each
    secondary_oam: [u8; 8 * 4],

    // Background shift registers
    pattern_lo: u16,
    pattern_hi: u16,
    attribute_lo: u16,
    attribute_hi: u16,

    // Sprite stuff
    sprite_evaluator: SpriteEvaluator,
    sprite_y_coords: [u8; 8],
    sprite_tile_nums: [u8; 8],
    sprite_attributes: [u8; 8],
    sprite_x_coords: [u8; 8],
    flag_sprite_size: u8,
    sprite_pattern_temp_low: [u8; 8],
    sprite_pattern_temp_high: [u8; 8],
    sprite_patterns: [u32; 8],

    // Open bus
    open_bus: u8,

    // Image buffers
    pub front: [u8; 256 * 240],
    pub back: [u8; 256 * 240],

    // I/O operations
    ppu_ctrl: PPUCTRL,
    ppu_mask: PPUMASK,
    ppu_status: PPUSTATUS,
    oam_addr: u8,
    read_buffer: u8,
    pub on_nmi: Box<dyn FnMut()>,

    // PPU metadata
    mirroring: Mirroring,
    pub cycle: u16,
    pub scanline: u16,
    store: PPUStore,
    nmi_prev: bool,

    // Debugging tools
    total_cycles: u64,
    total_scanlines: u64,
    pub total_frames: u64,
}

impl PPU {
    pub fn new(cartridge: &Cartridge) -> Self {
        PPU {
            v: 0,
            t: 0,
            x: 0,
            w: 0,
            f: 0,
            chr_rom: cartridge.chr_rom.clone(),
            vram: [0; 0x800],
            palette: [0; 0x20],
            oam: [0; 64 * 4],
            secondary_oam: [0; 8 * 4],
            pattern_lo: 0,
            pattern_hi: 0,
            attribute_lo: 0,
            attribute_hi: 0,
            sprite_evaluator: SpriteEvaluator::new(),
            sprite_y_coords: [0; 8],
            sprite_tile_nums: [0; 8],
            sprite_attributes: [0; 8],
            sprite_x_coords: [0; 8],
            flag_sprite_size: 0,
            sprite_pattern_temp_low: [0; 8],
            sprite_pattern_temp_high: [0; 8],
            sprite_patterns: [0; 8], // Final interleaved pattern data
            open_bus: 0,
            front: [0; 256 * 240],
            back: [0; 256 * 240],
            ppu_ctrl: PPUCTRL::empty(),
            ppu_mask: PPUMASK::empty(),
            ppu_status: PPUSTATUS::empty(),
            oam_addr: 0,
            read_buffer: 0,
            on_nmi: Box::new(|| {}),
            mirroring: cartridge.mirroring,
            cycle: 0,
            scanline: 0,
            store: PPUStore {
                nametable_byte: 0,
                attribute_byte: 0,
                tile_lo_byte: 0,
                tile_hi_byte: 0,
                tile_addr: 0,
            },
            nmi_prev: false,
            total_cycles: 0,
            total_scanlines: 0,
            total_frames: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.scanline = 261;
    }

    fn mem_read(&self, mut addr: u16) -> u8 {
        addr &= 0x3FFF;
        match addr {
            PATTERN_TABLE_START..=PATTERN_TABLE_END => {
                self.chr_rom[addr as usize]
            }
            NAMETABLE_START..=NAMETABLE_END => {
                addr = self.get_vram_addr(addr);
                assert!(addr < 0x800);
                self.vram[addr as usize]
            }
            PALETTE_START..=PALETTE_END => {
                addr = self.get_palette_addr(addr);
                assert!(addr < 0x20);
                self.palette[addr as usize]
            }
            _ => unreachable!(),
        }
    }

    fn mem_write(&mut self, mut addr: u16, data: u8) {
        addr &= 0x3FFF;
        match addr {
            PATTERN_TABLE_START..=PATTERN_TABLE_END => {
                self.chr_rom[addr as usize] = data
            }
            NAMETABLE_START..=NAMETABLE_END => {
                addr = self.get_vram_addr(addr);
                assert!(addr < 0x800);
                self.vram[addr as usize] = data;
            }
            PALETTE_START..=PALETTE_END => {
                addr = self.get_palette_addr(addr);
                assert!(addr < 0x20);
                self.palette[addr as usize] = data;
            }
            _ => unreachable!(),
        };
    }

    fn peek(&self, addr: u16) -> u8 {
        self.mem_read(addr)
    }

    pub fn tick(&mut self) {
        //———————————————————————————————————————————————————————————————————
        //  Pre-render -> {Render AND Evaluate} -> VBlank -> NMI
        //———————————————————————————————————————————————————————————————————
        if self.scanline == PRE_RENDER_LINE {
            self.handle_pre_render_line();
        }

        if VISIBLE_LINES.contains(&self.scanline) {
            // current scanline rendering
            self.handle_render_lines();
            // next scanline evaluation
            self.handle_evaluation_lines();
        }

        // vblank entrance/exit and NMI detection
        self.handle_vblank_lines();

        // NMI polling
        self.handle_nmi_polling();

        //———————————————————————————————————————————————————————————————————
        //  Timing calculations
        //———————————————————————————————————————————————————————————————————
        self.update_timing();
    }

    // Helpers
    fn get_vram_addr(&self, mut addr: u16) -> u16 {
        addr &= 0xFFF;
        let table = addr / 0x400;
        let offset = addr & 0x3FF;

        let mirrored = match self.mirroring {
            Mirroring::VERTICAL => table & 1, // 0,2 -> 0 (NT1), 1,3 -> 1 (NT2)
            Mirroring::HORIZONTAL => table >> 1, // 0,1 -> 0 (NT1), 2,3 -> 1 (NT2)
            Mirroring::ALTERNATIVE => unimplemented!(),
        };

        (mirrored << 10) + offset
    }

    fn get_palette_addr(&self, mut addr: u16) -> u16 {
        addr &= 0x1F;
        // Mirror $3F10/$3F14/$3F18/$3F1C to $3F00/$3F04/$3F08/$3F0C
        if addr >= 0x10 && addr % 4 == 0 {
            addr &= 0x0F;
        }
        addr
    }

    fn handle_pre_render_line(&mut self) {
        if self
            .ppu_mask
            .intersects(PPUMASK::SHOW_BACKGROUND | PPUMASK::SHOW_SPRITES)
        {
            if (280..=304).contains(&self.cycle) {
                self.copy_y();
            }
            self.handle_fetch_cycles();
        }

        // Handle sprite evaluation on pre-render line
        self.handle_evaluation_lines();
    }

    fn handle_render_lines(&mut self) {
        if self
            .ppu_mask
            .intersects(PPUMASK::SHOW_BACKGROUND | PPUMASK::SHOW_SPRITES)
        {
            if VISIBLE_CYCLES.contains(&self.cycle) {
                self.draw_pixel();
            }
            self.handle_fetch_cycles();
        }
    }

    fn handle_fetch_cycles(&mut self) {
        if PRE_FETCH_CYCLES.contains(&self.cycle)
            || VISIBLE_CYCLES.contains(&self.cycle)
        {
            self.pattern_lo <<= 1;
            self.pattern_hi <<= 1;
            self.attribute_lo <<= 1;
            self.attribute_hi <<= 1;

            match self.cycle % 8 {
                1 => self.fetch_nametable(),
                3 => self.fetch_attribute(),
                5 => self.fetch_tile_lo(),
                7 => self.fetch_tile_hi(),
                0 => {
                    self.increment_x();
                    self.store_tiles();
                }
                _ => {}
            }

            if self.cycle == 256 {
                self.increment_y();
            }
        } else if self.cycle == 257 {
            self.copy_x();
        }
    }

    fn fetch_sprite_x_dummy(&mut self, sprite_idx: u64) {
        // Cycles 4-7: Fetch sprite pattern data while doing dummy X coordinate reads
        if sprite_idx >= 8 {
            return;
        }

        let sprite_idx = sprite_idx as usize;
        let cycle_in_pattern_fetch = (self.cycle - 257) % 8 - 4; // 0-3 for cycles 4-7

        // Still do the dummy read from secondary OAM as hardware does
        let addr = sprite_idx * 4 + 3;
        let _ = if addr < 32 {
            self.secondary_oam[addr]
        } else {
            0xFF
        };

        // But also fetch the actual sprite pattern data
        match cycle_in_pattern_fetch {
            0 => self.fetch_sprite_pattern_low(sprite_idx),
            1 => self.fetch_sprite_pattern_high(sprite_idx),
            2 => self.finalize_sprite_pattern(sprite_idx),
            3 => {} // Additional dummy cycle
            _ => unreachable!(),
        }
    }

    fn fetch_sprite_pattern_low(&mut self, sprite_idx: usize) {
        if self.sprite_y_coords[sprite_idx] == 0xFF {
            self.sprite_patterns[sprite_idx] = 0;
            return;
        }
        let sprite_y = self.sprite_y_coords[sprite_idx];
        let tile_num = self.sprite_tile_nums[sprite_idx];
        let attributes = self.sprite_attributes[sprite_idx];
        let next_scanline = if self.scanline == 261 {
            0
        } else {
            self.scanline + 1
        };
        let mut row = next_scanline.wrapping_sub(sprite_y as u16);
        let sprite_height = if self.flag_sprite_size == 0 { 8 } else { 16 };
        if row >= sprite_height {
            self.sprite_patterns[sprite_idx] = 0;
            return;
        }
        // 8x8 sprite
        if self.flag_sprite_size == 0 {
            let base = if self.ppu_ctrl.contains(PPUCTRL::SPRITE_PATTERN_TABLE)
            {
                0x1000
            } else {
                0x0000
            };
            let mut fine_y = row;
            if (attributes & 0x80) != 0 {
                fine_y = 7 - fine_y;
            } // vertical flip
            let tile_addr = base + (tile_num as u16) * 16 + fine_y;
            self.sprite_pattern_temp_low[sprite_idx] =
                self.mem_read(tile_addr);
        } else {
            // 8x16 sprite
            let table = (tile_num & 1) as u16;
            let base = table * 0x1000;
            let tile_index = (tile_num & 0xFE) as u16;
            let mut fine_y = row;
            let mut tile = tile_index;
            // vertical flip for 8x16: flip within the 16-pixel region
            if (attributes & 0x80) != 0 {
                fine_y = 15 - fine_y;
            }
            if fine_y >= 8 {
                tile += 1;
                fine_y -= 8;
            }
            let tile_addr = base + tile * 16 + fine_y;
            self.sprite_pattern_temp_low[sprite_idx] =
                self.mem_read(tile_addr);
        }
    }

    fn fetch_sprite_pattern_high(&mut self, sprite_idx: usize) {
        if self.sprite_y_coords[sprite_idx] == 0xFF {
            return;
        }
        let sprite_y = self.sprite_y_coords[sprite_idx];
        let tile_num = self.sprite_tile_nums[sprite_idx];
        let attributes = self.sprite_attributes[sprite_idx];
        let next_scanline = if self.scanline == 261 {
            0
        } else {
            self.scanline + 1
        };
        let mut row = next_scanline.wrapping_sub(sprite_y as u16);
        let sprite_height = if self.flag_sprite_size == 0 { 8 } else { 16 };
        if row >= sprite_height {
            self.sprite_pattern_temp_high[sprite_idx] = 0;
            return;
        }
        if self.flag_sprite_size == 0 {
            let base = if self.ppu_ctrl.contains(PPUCTRL::SPRITE_PATTERN_TABLE)
            {
                0x1000
            } else {
                0x0000
            };
            let mut fine_y = row;
            if (attributes & 0x80) != 0 {
                fine_y = 7 - fine_y;
            }
            let tile_addr = base + (tile_num as u16) * 16 + fine_y + 8;
            self.sprite_pattern_temp_high[sprite_idx] =
                self.mem_read(tile_addr);
        } else {
            let table = (tile_num & 1) as u16;
            let base = table * 0x1000;
            let tile_index = (tile_num & 0xFE) as u16;
            let mut fine_y = row;
            let mut tile = tile_index;
            if (attributes & 0x80) != 0 {
                fine_y = 15 - fine_y;
            }
            if fine_y >= 8 {
                tile += 1;
                fine_y -= 8;
            }
            let tile_addr = base + tile * 16 + fine_y + 8;
            self.sprite_pattern_temp_high[sprite_idx] =
                self.mem_read(tile_addr);
        }
    }

    fn finalize_sprite_pattern(&mut self, sprite_idx: usize) {
        if self.sprite_y_coords[sprite_idx] == 0xFF {
            return;
        }
        let low_byte = self.sprite_pattern_temp_low[sprite_idx];
        let high_byte = self.sprite_pattern_temp_high[sprite_idx];
        let attributes = self.sprite_attributes[sprite_idx];
        let flip_h = (attributes & 0x40) != 0;
        let mut pattern = 0u32;
        for i in 0..8 {
            let bit = if flip_h { i } else { 7 - i };
            let low_bit = (low_byte >> bit) & 1;
            let high_bit = (high_byte >> bit) & 1;
            let color = (high_bit << 1) | low_bit;
            pattern |= (color as u32) << (i * 4);
        }
        self.sprite_patterns[sprite_idx] = pattern;
    }

    fn handle_evaluation_lines(&mut self) {
        let sprite_height = if self.flag_sprite_size == 0 { 8 } else { 16 };
        let next_scanline = if self.scanline == 261 {
            0
        } else {
            self.scanline + 1
        };

        match self.cycle {
            0 => {
                // Initialize sprite evaluator for this scanline
                self.sprite_evaluator.reset_for_scanline();
                // Set sprite size flag from PPUCTRL
                self.flag_sprite_size =
                    if self.ppu_ctrl.contains(PPUCTRL::SPRITE_SIZE) {
                        1
                    } else {
                        0
                    };
            }

            1..=64 => {
                // Clear secondary OAM - cycles 1-64
                // Hardware actually reads from OAM and writes to secondary OAM,
                // but forces the read to return $FF
                let cycle_in_clear = self.cycle - 1;

                if cycle_in_clear % 2 == 0 {
                    // Odd cycle: "read" from OAM (always returns $FF during clear)
                    self.sprite_evaluator.temp_byte = 0xFF;
                } else {
                    // Even cycle: write to secondary OAM
                    let secondary_addr = cycle_in_clear / 2;
                    if secondary_addr < 32 {
                        self.secondary_oam[secondary_addr as usize] = 0xFF;
                    }
                }

                // Transition to Copy state after clearing is done
                if self.cycle == 64 {
                    self.sprite_evaluator.state = SpriteEvalState::Copy;
                }
            }

            65..=256 => {
                // Sprite evaluation - cycles 65-256
                let eval_cycle = self.cycle - 65;

                if eval_cycle % 2 == 0 {
                    // Odd cycle: read from primary OAM
                    self.handle_sprite_eval_read();
                } else {
                    // Even cycle: write to secondary OAM (or read if full)
                    self.handle_sprite_eval_write_with_cache(
                        sprite_height,
                        next_scanline,
                    );
                }
            }

            257..=320 => {
                // Sprite fetches - 8 sprites, 8 cycles each
                let fetch_cycle = self.cycle - 257;
                let sprite_idx = fetch_cycle / 8;
                let cycle_in_sprite = fetch_cycle % 8;

                match cycle_in_sprite {
                    0 => self.fetch_sprite_y(sprite_idx as u64),
                    1 => self.fetch_sprite_tile(sprite_idx as u64),
                    2 => self.fetch_sprite_attr(sprite_idx as u64),
                    3 => self.fetch_sprite_x(sprite_idx as u64),
                    4..=7 => self.fetch_sprite_x_dummy(sprite_idx as u64), // Dummy reads
                    _ => unreachable!(),
                }
            }

            321..=340 => {
                // Background render pipeline initialization
                // Read first byte of secondary OAM while fetching next scanline's BG tiles
                if self.cycle == 321 {
                    let _ = self.secondary_oam[0]; // Dummy read
                }
            }

            _ => unreachable!(),
        }
    }

    fn handle_sprite_eval_read(&mut self) {
        match self.sprite_evaluator.state {
            SpriteEvalState::Copy => {
                // Read sprite data from primary OAM
                let oam_addr = (self.sprite_evaluator.n as usize * 4)
                    + self.sprite_evaluator.m as usize;
                if oam_addr < 256 {
                    self.sprite_evaluator.temp_byte = self.oam[oam_addr];
                } else {
                    self.sprite_evaluator.temp_byte = 0;
                }
            }

            SpriteEvalState::OverflowCheck => {
                // Read for overflow detection (with bug)
                let oam_addr = (self.sprite_evaluator.n as usize * 4)
                    + self.sprite_evaluator.m as usize;
                if oam_addr < 256 {
                    self.sprite_evaluator.temp_byte = self.oam[oam_addr];
                } else {
                    self.sprite_evaluator.temp_byte = 0;
                }
            }

            SpriteEvalState::Done => {
                // Continue reading but ignore results
                let oam_addr = self.sprite_evaluator.n as usize * 4;
                if oam_addr < 256 {
                    self.sprite_evaluator.temp_byte = self.oam[oam_addr];
                } else {
                    self.sprite_evaluator.temp_byte = 0;
                }
            }

            _ => {}
        }
    }

    fn handle_sprite_eval_write_with_cache(
        &mut self,
        sprite_height: u8,
        next_scanline: u16,
    ) {
        match self.sprite_evaluator.state {
            SpriteEvalState::Copy => {
                if self.sprite_evaluator.m == 0 {
                    // First byte: check Y coordinate
                    let sprite_y = self.sprite_evaluator.temp_byte;
                    let row = next_scanline.wrapping_sub(sprite_y as u16);

                    if row < sprite_height as u16 {
                        // Sprite is in range
                        if self.sprite_evaluator.sprites_found < 8 {
                            // Copy Y coordinate to secondary OAM
                            self.secondary_oam[self
                                .sprite_evaluator
                                .secondary_oam_addr
                                as usize] = sprite_y;
                            self.sprite_evaluator.secondary_oam_addr += 1;

                            if self.sprite_evaluator.n == 0 {
                                self.sprite_evaluator.sprite_0_on_line = true;
                            }

                            // Continue copying remaining bytes
                            self.sprite_evaluator.m = 1;
                        } else {
                            // 8 sprites already found, enter overflow check
                            self.sprite_evaluator.state =
                                SpriteEvalState::OverflowCheck;
                            self.sprite_evaluator.m = 0;
                            self.sprite_evaluator.overflow_flag = true;
                            // Set sprite overflow flag in PPU status
                            self.ppu_status.insert(PPUSTATUS::SPRITE_OVERFLOW);
                        }
                    } else {
                        // Sprite not in range, move to next sprite
                        self.advance_to_next_sprite();
                    }
                } else {
                    // Copy remaining bytes (tile, attr, x)
                    if self.sprite_evaluator.sprites_found < 8 {
                        self.secondary_oam[self
                            .sprite_evaluator
                            .secondary_oam_addr
                            as usize] = self.sprite_evaluator.temp_byte;
                        self.sprite_evaluator.secondary_oam_addr += 1;
                    }

                    self.sprite_evaluator.m += 1;
                    if self.sprite_evaluator.m >= 4 {
                        // Finished copying this sprite
                        self.sprite_evaluator.sprites_found += 1;
                        self.advance_to_next_sprite();
                    }
                }
            }

            SpriteEvalState::OverflowCheck => {
                // Sprite overflow bug: check Y coordinate but with buggy address increment
                let sprite_y = self.sprite_evaluator.temp_byte;
                let row = next_scanline.wrapping_sub(sprite_y as u16);

                if row < sprite_height as u16 {
                    // Found another in-range sprite
                    self.sprite_evaluator.overflow_flag = true;
                    // Set sprite overflow flag in PPU status
                    self.ppu_status.insert(PPUSTATUS::SPRITE_OVERFLOW);

                    // Increment m 3 more times (hardware bug)
                    for _ in 0..3 {
                        self.sprite_evaluator.m =
                            (self.sprite_evaluator.m + 1) % 4;
                        if self.sprite_evaluator.m == 0 {
                            self.sprite_evaluator.n =
                                self.sprite_evaluator.n.wrapping_add(1);
                            if self.sprite_evaluator.n == 0 {
                                self.sprite_evaluator.state =
                                    SpriteEvalState::Done;
                                return;
                            }
                        }
                    }

                    if self.sprite_evaluator.m == 3 {
                        self.sprite_evaluator.n =
                            self.sprite_evaluator.n.wrapping_add(1);
                        if self.sprite_evaluator.n == 0 {
                            self.sprite_evaluator.state =
                                SpriteEvalState::Done;
                            return;
                        }
                    }

                    self.sprite_evaluator.state = SpriteEvalState::Done;
                } else {
                    // Not in range, increment n and m (bug: m increments without carry)
                    self.sprite_evaluator.n =
                        self.sprite_evaluator.n.wrapping_add(1);
                    self.sprite_evaluator.m =
                        (self.sprite_evaluator.m + 1) % 4;

                    if self.sprite_evaluator.n == 0 {
                        self.sprite_evaluator.state = SpriteEvalState::Done;
                    }
                }
            }

            SpriteEvalState::Done => {
                // Try to copy but fail (no actual write)
                self.sprite_evaluator.n =
                    self.sprite_evaluator.n.wrapping_add(1);
            }

            _ => {}
        }
    }

    fn advance_to_next_sprite(&mut self) {
        self.sprite_evaluator.n = self.sprite_evaluator.n.wrapping_add(1);
        self.sprite_evaluator.m = 0;

        if self.sprite_evaluator.n == 0 {
            // Evaluated all 64 sprites
            self.sprite_evaluator.state = SpriteEvalState::Done;
        } else if self.sprite_evaluator.sprites_found < 8 {
            // Continue looking for sprites
            self.sprite_evaluator.state = SpriteEvalState::Copy;
        } else {
            // Found 8 sprites, enter overflow check mode
            self.sprite_evaluator.state = SpriteEvalState::OverflowCheck;
        }
    }

    fn fetch_sprite_y(&mut self, sprite_idx: u64) {
        if sprite_idx < 8 {
            let addr = sprite_idx * 4;
            let y_coord = if addr < 32 {
                self.secondary_oam[addr as usize]
            } else {
                0xFF // Empty sprite slot
            };
            // Store for sprite rendering
            self.sprite_y_coords[sprite_idx as usize] = y_coord;
        }
    }

    fn fetch_sprite_tile(&mut self, sprite_idx: u64) {
        if sprite_idx < 8 {
            let addr = sprite_idx * 4 + 1;
            let tile_num = if addr < 32 {
                self.secondary_oam[addr as usize]
            } else {
                0xFF
            };
            self.sprite_tile_nums[sprite_idx as usize] = tile_num;
        }
    }

    fn fetch_sprite_attr(&mut self, sprite_idx: u64) {
        if sprite_idx < 8 {
            let addr = sprite_idx * 4 + 2;
            let attributes = if addr < 32 {
                self.secondary_oam[addr as usize]
            } else {
                0xFF
            };
            self.sprite_attributes[sprite_idx as usize] = attributes;
        }
    }

    fn fetch_sprite_x(&mut self, sprite_idx: u64) {
        if sprite_idx < 8 {
            let addr = sprite_idx * 4 + 3;
            let x_coord = if addr < 32 {
                self.secondary_oam[addr as usize]
            } else {
                0xFF
            };
            self.sprite_x_coords[sprite_idx as usize] = x_coord;
        }
    }

    fn handle_vblank_lines(&mut self) {
        if self.scanline == 241 && self.cycle == 1 {
            // enter VBlank
            self.ppu_status.insert(PPUSTATUS::IS_VBLANK);
            // present completed frame
            std::mem::swap(&mut self.front, &mut self.back);
            // self.back.fill(0); // MAYBE BUG: reset buffer or not?
        }

        if self.scanline == PRE_RENDER_LINE && self.cycle == 1 {
            // exit VBlank
            self.ppu_status.remove(PPUSTATUS::IS_VBLANK);
            self.nmi_prev = false;
            self.ppu_status.remove(PPUSTATUS::SPRITE0_HIT);
            self.ppu_status.remove(PPUSTATUS::SPRITE_OVERFLOW);
        }
    }

    fn handle_nmi_polling(&mut self) {
        let nmi_now = self.ppu_ctrl.contains(PPUCTRL::NMI_ON_VBLANK)
            && self.ppu_status.contains(PPUSTATUS::IS_VBLANK);
        if nmi_now && !self.nmi_prev {
            (self.on_nmi.as_mut())();
        }
        self.nmi_prev = nmi_now;
    }

    fn update_timing(&mut self) {
        self.cycle += 1;
        self.total_cycles += 1;

        // skip the last cycle of the pre-render line on odd frames
        if (self.f == 1 && self.scanline == 261 && self.cycle == 340)
            || self.cycle == 341
        {
            self.cycle = 0;
            self.scanline += 1;
            self.total_scanlines += 1;
        }

        // finalize frame
        if self.scanline == 262 {
            self.scanline = 0;
            self.f ^= 1;
            self.total_frames += 1;
        }
    }
}
