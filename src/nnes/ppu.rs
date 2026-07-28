mod core;
mod io;
mod sprites;

use crate::cartridge::{Cartridge, Mirroring};

const PATTERN_TABLE_START: u16 = 0x0000;
const PATTERN_TABLE_END: u16 = 0x1FFF;
const NAMETABLE_START: u16 = 0x2000;
const NAMETABLE_END: u16 = 0x3EFF;
const PALETTE_START: u16 = 0x3F00;
const PALETTE_END: u16 = 0x3FFF;

const PRE_RENDER_LINE: u16 = 261;
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

#[derive(PartialEq)]
pub enum SpriteEvalState {
    YCoordinate,
    TileNumber,
    Attributes,
    XCoordinate,
    Done,
}

pub struct PPUStore {
    // Background state
    nametable_byte: u8,
    attribute_byte: u8,
    tile_lo_byte: u8,
    tile_hi_byte: u8,
    tile_addr: u16,

    // Sprite state
    sprite_eval_state: SpriteEvalState,
    curr_sprite: u8,
    accepted_sprite: u8,
    read_sprite: u8,
    curr_sprite_byte: u8,
    read_sprite_byte: u8,
    found_empty: bool,
    sprite_0_in_secondary: bool,
    y_coordinate: u8,
    tile_number: u8,
    attributes: u8,
    x_coordinate: u8,
    // Sprite fetch state carried between fetch dots 5 (low) and 7 (high)
    sprite_pattern_addr: u16,
    sprite_pattern_lo: u8,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct SpriteRender {
    pattern_lo: u8,
    pattern_hi: u8,
    attributes: u8,
    x_counter: u8,
    is_sprite_0: bool,
}

pub struct PPU {
    // Architectural state
    v: u16, // 15 bits: yyy NN YYYYY XXXXX (fine y, nametable select, coarse y, coarse x)
    t: u16, // 15 bits (temporary): contents eventually transferred to v
    x: u8,  // 3 bits
    w: u8,  // 1 bit
    f: u8,  // 1 bit
    chr_rom: Vec<u8>,
    chr_is_ram: bool,
    vram: [u8; 0x800],
    palette: [u8; 0x20],
    // Sprites are 4 bytes each:
    //   y_coordinate
    //   tile_number
    //   attributes
    //   x_coordinate
    pub oam: [u8; 64 * 4],
    secondary_oam: [u8; 8 * 4],

    // Background shift registers
    pattern_lo: u16,
    pattern_hi: u16,
    attribute_lo: u16,
    attribute_hi: u16,

    // Sprite rendering state (8 sprites per scanline)
    sprite_render: [SpriteRender; 8],
    sprite_count: u8,
    sprite_0_on_line: bool,
    // Last byte the sprite pipeline placed on the internal OAM bus; exposed
    // by OAMDATA reads while rendering
    oam_bus: u8,
    // OAMADDR at the start of sprite evaluation: evaluation scans OAM from
    // this base, and the entry at the base acts as "sprite 0" for hit purposes
    oam_addr_base: u8,

    // Open bus
    open_bus: u8,

    // Image buffers
    pub front: [u16; 256 * 240],
    pub back: [u16; 256 * 240],

    // I/O operations
    ppu_ctrl: PPUCTRL,
    ppu_mask: PPUMASK,
    ppu_status: PPUSTATUS,
    oam_addr: u8,
    read_buffer: u8,
    open_bus_decay: [u64; 8],
    v_pending: u16,
    v_delay: u8,
    pub nmi_signal: bool,
    pub nmi_suppress: bool,
    vbl_race_skip: bool,

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
            chr_is_ram: cartridge.has_chr_ram,
            vram: [0; 0x800],
            palette: [0; 0x20],
            oam: [0; 64 * 4],
            secondary_oam: [0; 8 * 4],
            pattern_lo: 0,
            pattern_hi: 0,
            attribute_lo: 0,
            attribute_hi: 0,
            sprite_render: [SpriteRender::default(); 8],
            sprite_count: 0,
            sprite_0_on_line: false,
            oam_bus: 0,
            oam_addr_base: 0,
            open_bus: 0,
            front: [0; 256 * 240],
            back: [0; 256 * 240],
            ppu_ctrl: PPUCTRL::empty(),
            ppu_mask: PPUMASK::empty(),
            ppu_status: PPUSTATUS::empty(),
            oam_addr: 0,
            read_buffer: 0,
            open_bus_decay: [0; 8],
            v_pending: 0,
            v_delay: 0,
            nmi_signal: false,
            nmi_suppress: false,
            vbl_race_skip: false,
            mirroring: cartridge.mirroring,
            cycle: 0,
            scanline: 0,
            store: PPUStore {
                nametable_byte: 0,
                attribute_byte: 0,
                tile_lo_byte: 0,
                tile_hi_byte: 0,
                tile_addr: 0,
                sprite_eval_state: SpriteEvalState::YCoordinate,
                curr_sprite: 0,
                accepted_sprite: 0,
                read_sprite: 0,
                curr_sprite_byte: 0,
                read_sprite_byte: 0,
                found_empty: false,
                sprite_0_in_secondary: false,
                y_coordinate: 0,
                tile_number: 0,
                attributes: 0,
                x_coordinate: 0,
                sprite_pattern_addr: 0,
                sprite_pattern_lo: 0,
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
                if self.chr_is_ram {
                    self.chr_rom[addr as usize] = data;
                }
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

    #[allow(dead_code)]
    fn peek(&self, addr: u16) -> u8 {
        self.mem_read(addr)
    }

    pub fn tick(&mut self) {
        // Apply the delayed copy of t into v from the second PPUADDR write
        if self.v_delay > 0 {
            self.v_delay -= 1;
            if self.v_delay == 0 {
                self.v = self.v_pending;
            }
        }

        // ==================================================================
        // PPU Frame Structure (262 scanlines × 341 cycles):
        //   Scanline 261 (pre-render): Clear flags, prefetch first tiles
        //   Scanlines 0-239 (visible): Render + evaluate sprites for next line
        //   Scanline 240 (post-render): Idle
        //   Scanlines 241-260 (vblank): VBlank flag set, NMI triggered
        // ==================================================================

        let rendering_enabled = self
            .ppu_mask
            .intersects(PPUMASK::SHOW_BACKGROUND | PPUMASK::SHOW_SPRITES);

        match self.scanline {
            PRE_RENDER_LINE => self.tick_pre_render(rendering_enabled),
            0..=239 => self.tick_visible(rendering_enabled),
            240 => {}       // Post-render: idle
            241..=260 => {} // VBlank: handled separately
            _ => {}
        }

        // VBlank flag management (separate from scanline handling)
        self.handle_vblank_flags();

        // NMI edge detection and triggering
        self.poll_nmi();

        // Advance timing counters
        self.advance_timing(rendering_enabled);
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

    fn tick_pre_render(&mut self, rendering_enabled: bool) {
        // Cycle 0: Reset sprite state for new frame
        if self.cycle == 0 {
            self.reset_sprite_state();
        }

        if rendering_enabled {
            // Cycles 1-64: Clear secondary OAM to $FF
            if (1..=64).contains(&self.cycle) && (self.cycle - 1) % 2 == 0 {
                self.secondary_oam[((self.cycle - 1) / 2) as usize] = 0xFF;
                self.oam_bus = 0xFF;
            }

            // Cycles 280-304: Repeatedly copy vertical scroll from t to v
            if (280..=304).contains(&self.cycle) {
                self.copy_y();
            }

            // Background tile fetching (cycles 1-256 and 321-336)
            self.tick_background_fetch();

            // Sprite tile fetching (cycles 257-320)
            // All sprites are "empty" since secondary OAM is cleared
            if (257..=320).contains(&self.cycle) {
                // OAMADDR is reset to 0 during each of these dots
                self.oam_addr = 0;
                self.tick_sprite_fetch();
            }

            // Cycles 321-340 and 0: the OAM bus shows secondary OAM byte 0
            if self.cycle == 0 || (321..=340).contains(&self.cycle) {
                self.oam_bus = self.secondary_oam[0];
            }
        }
    }

    fn tick_visible(&mut self, rendering_enabled: bool) {
        // Cycle 0: Reset sprite evaluation state for this scanline
        if self.cycle == 0 {
            self.reset_sprite_state();
        }

        if rendering_enabled {
            // Cycles 1-64: Clear secondary OAM
            if (1..=64).contains(&self.cycle) && (self.cycle - 1) % 2 == 0 {
                self.secondary_oam[((self.cycle - 1) / 2) as usize] = 0xFF;
                self.oam_bus = 0xFF;
            }

            // Cycles 65-256: Sprite evaluation (for next scanline)
            if (65..=256).contains(&self.cycle) {
                self.tick_sprite_evaluation();
            }

            // Cycles 257-320: Sprite tile fetching (for next scanline)
            if (257..=320).contains(&self.cycle) {
                // OAMADDR is reset to 0 during each of these dots
                self.oam_addr = 0;
                self.tick_sprite_fetch();
            }

            // Cycles 321-340 and 0: the OAM bus shows secondary OAM byte 0
            if self.cycle == 0 || (321..=340).contains(&self.cycle) {
                self.oam_bus = self.secondary_oam[0];
            }

            // Cycles 1-256: Render pixels
            if VISIBLE_CYCLES.contains(&self.cycle) {
                self.draw_pixel();
            }

            // Background tile fetching
            self.tick_background_fetch();
        } else if VISIBLE_CYCLES.contains(&self.cycle) {
            // Forced blank: the screen shows the backdrop color
            self.draw_backdrop();
        }
    }

    fn tick_background_fetch(&mut self) {
        // Shift registers advance on cycles 1-256 and 321-336
        if PRE_FETCH_CYCLES.contains(&self.cycle)
            || VISIBLE_CYCLES.contains(&self.cycle)
        {
            // Shift all registers left by 1
            self.pattern_lo <<= 1;
            self.pattern_hi <<= 1;
            self.attribute_lo <<= 1;
            self.attribute_hi <<= 1;

            // Fetch new tile data every 8 cycles
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

            // Increment Y at end of visible portion
            if self.cycle == 256 {
                self.increment_y();
            }
        } else if self.cycle == 257 {
            // Copy horizontal scroll from t to v
            self.copy_x();
        } else if self.cycle == 337 || self.cycle == 339 {
            // Dummy nametable fetches at the end of the scanline; the fetched
            // byte is overwritten by the next real fetch, but hardware issues
            // these reads (MMC5 counts them to clock its scanline counter)
            self.fetch_nametable();
        }
    }

    fn handle_vblank_flags(&mut self) {
        // Scanline 241, cycle 1: Enter VBlank
        if self.scanline == 241 && self.cycle == 1 {
            // A PPUSTATUS read one dot earlier keeps the flag (and the NMI)
            // from being set for this frame
            if self.vbl_race_skip {
                self.vbl_race_skip = false;
            } else {
                self.ppu_status.insert(PPUSTATUS::IS_VBLANK);
            }
            // Swap front/back buffers to present completed frame
            std::mem::swap(&mut self.front, &mut self.back);
        }

        // Pre-render line, cycle 1: Exit VBlank and clear flags
        if self.scanline == PRE_RENDER_LINE && self.cycle == 1 {
            self.ppu_status.remove(PPUSTATUS::IS_VBLANK);
            self.ppu_status.remove(PPUSTATUS::SPRITE0_HIT);
            self.ppu_status.remove(PPUSTATUS::SPRITE_OVERFLOW);
            self.nmi_prev = false;
        }
    }

    fn poll_nmi(&mut self) {
        // NMI is edge-triggered: fires when (NMI_ON_VBLANK && IS_VBLANK) transitions low->high
        let nmi_now = self.ppu_ctrl.contains(PPUCTRL::NMI_ON_VBLANK)
            && self.ppu_status.contains(PPUSTATUS::IS_VBLANK);
        if nmi_now && !self.nmi_prev {
            self.nmi_signal = true;
        }
        self.nmi_prev = nmi_now;
    }

    fn advance_timing(&mut self, rendering_enabled: bool) {
        self.cycle += 1;
        self.total_cycles += 1;

        // Odd frame skip: on odd frames with rendering enabled, skip cycle 340 of pre-render
        let odd_skip = self.f == 1
            && self.scanline == PRE_RENDER_LINE
            && self.cycle == 340
            && rendering_enabled;

        if odd_skip || self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;
            self.total_scanlines += 1;
        }

        // Frame complete: wrap scanline back to 0
        if self.scanline == 262 {
            self.scanline = 0;
            self.f ^= 1;
            self.total_frames += 1;
        }
    }
}
