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
    sprite_height: u8,
    curr_sprite: u8,
    accepted_sprite: u8,
    read_sprite: u8,
    curr_sprite_byte: u8,
    read_sprite_byte: u8,
    found_empty: bool,
    y_coordinate: u8,
    tile_number: u8,
    attributes: u8,
    x_coordinate: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    y_coordinate: u8,
    tile_number: u8,
    attributes: u8,
    x_coordinate: u8,
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

    // Fetched sprites
    sprites: [Sprite; 8],

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
            sprites: [
                Sprite { 
                    y_coordinate: 0, 
                    tile_number: 0, 
                    attributes: 0, 
                    x_coordinate: 0 
                }; 8],
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
                sprite_eval_state: SpriteEvalState::YCoordinate,
                sprite_height: 0,
                curr_sprite: 0,
                accepted_sprite: 0,
                read_sprite: 0,
                curr_sprite_byte: 0,
                read_sprite_byte: 0,
                found_empty: false,
                y_coordinate: 0,
                tile_number: 0,
                attributes: 0,
                x_coordinate: 0,
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

    fn handle_evaluation_lines(&mut self) {
        self.store.sprite_height = 
            if self.ppu_ctrl.contains(PPUCTRL::SPRITE_SIZE) { 16 } else { 8 };
        match self.cycle {
            0 => {}
            1..=64 => {
                // clear secondary OAM
                // hack memory accesses to simulate one cycle read, one cycle write
                let idx = if (self.cycle - 1) % 2 == 0 {
                    Some((self.cycle - 1) / 2)
                } else {
                    None
                };
                if let Some(idx) = idx {
                    self.secondary_oam[idx as usize] = 0xFF;
                }
            }
            65..=256 => {
                // note: oam[n][m] = oam[4*n + m]
                // 1. starting at n = 0,
                //    read oam[n][0] (y coordinate) and copy to secondary oam if not full
                //    if the y coordinate is in range, copy remaining 3 bytes of oam[n]
                // 2. increment n (merged with step 1 in the code below)
                //    if n == 0, go to step 4
                //    if less than 8 sprites have been found, go to step 1
                //    if 8 sprites have been found, disable writes to secondary oam
                // 3. starting at m = 0, (mergec with step 1 in the code below)
                //    evaluate oam[n][m] as y coordinate
                //    if the y coordinate is in range, set sprite overflow and read next 3 oam
                //      entries incrementing m, and incrementing n when m overflows from 3 -> 0
                //    if the y coordinate is not in range, increment n and m without carry.
                //      if n == 0, go to step 4, else go to step 3
                // 4. attempt (and fail) to copy oam[n][0] into secondary OAM, and increment n.
                //      repeat until hblank (cycle 257?) is reached
                let oam_idx = 4 * self.store.curr_sprite + self.store.curr_sprite_byte;
                let secondary_oam_idx = 4 * self.store.accepted_sprite + self.store.curr_sprite_byte;
                if self.cycle % 2 == 1 {
                    // read from primary OAM
                    match self.store.sprite_eval_state {
                        SpriteEvalState::YCoordinate => {
                            self.store.y_coordinate = self.oam[oam_idx as usize];
                        }
                        SpriteEvalState::TileNumber => {
                            self.store.tile_number = self.oam[oam_idx as usize];
                        }
                        SpriteEvalState::Attributes => {
                            self.store.attributes = self.oam[oam_idx as usize];
                        }
                        SpriteEvalState::XCoordinate => {
                            self.store.x_coordinate = self.oam[oam_idx as usize];
                        }
                        SpriteEvalState::Done => {
                            let _ = self.oam[oam_idx as usize];
                        }
                    }
                } else {
                    // if secondary oam is not full, write to it. if it is full, read from it.
                    if self.store.accepted_sprite < 8 {
                        self.secondary_oam[secondary_oam_idx as usize] = match self.store.sprite_eval_state {
                            SpriteEvalState::YCoordinate => {
                                self.store.y_coordinate
                            }
                            SpriteEvalState::TileNumber => {
                                self.store.tile_number
                            }
                            SpriteEvalState::Attributes => {
                                self.store.attributes
                            }
                            SpriteEvalState::XCoordinate => {
                                self.store.x_coordinate
                            }
                            SpriteEvalState::Done => unreachable!()
                        };
                    } else {
                        let _ = self.secondary_oam[0];
                    }

                    if self.store.sprite_eval_state != SpriteEvalState::Done {
                        // if y coordinate is in range
                        if (self.scanline >= self.store.y_coordinate as u16)
                            && (self.scanline < (self.store.y_coordinate + self.store.sprite_height) as u16)
                        {
                            self.store.curr_sprite_byte = match self.store.curr_sprite_byte {
                                0 => {
                                    self.store.sprite_eval_state = SpriteEvalState::TileNumber;
                                    1
                                }
                                1 => {
                                    self.store.sprite_eval_state = SpriteEvalState::Attributes;
                                    2
                                }
                                2 => {
                                    self.store.sprite_eval_state = SpriteEvalState::XCoordinate;
                                    3
                                }
                                3 => {
                                    self.store.sprite_eval_state = SpriteEvalState::YCoordinate;
                                    0
                                }
                                _ => unreachable!()
                            };

                            // if m 3 -> 0, n += 1
                            if self.store.curr_sprite_byte == 0 {
                                self.store.curr_sprite = self.store.curr_sprite.wrapping_add(1);
                                // if done reading from oam
                                if self.store.curr_sprite == 0 {
                                    self.store.sprite_eval_state = SpriteEvalState::Done;
                                }
                            }

                            // if overflow, flag. else, if m 3 -> 0, num sprites += 1
                            if self.store.accepted_sprite >= 8 {
                                self.ppu_status.insert(PPUSTATUS::SPRITE_OVERFLOW);
                            } else {
                                if self.store.curr_sprite_byte == 0 {
                                    self.store.accepted_sprite = self.store.accepted_sprite.wrapping_add(1);
                                }
                            }
                        } else {
                            // check next sprite
                            self.store.curr_sprite = self.store.curr_sprite.wrapping_add(1);
                            // if done reading from oam
                            if self.store.curr_sprite == 0 {
                                self.store.sprite_eval_state = SpriteEvalState::Done;
                            }
                            if self.store.accepted_sprite >= 8 {
                                // m += 1 bug if overflow and y in range
                                self.store.curr_sprite_byte = match self.store.curr_sprite_byte {
                                    0 => 1,
                                    1 => 2,
                                    2 => 3,
                                    3 => 0,
                                    _ => unreachable!()
                                };
                            }
                        }
                    }
                }
            }
            257..=320 => {
                // cycles 1-4: read the y coordinate, tile number, attributes, and x coordinate
                //   from secondary OAM
                // cycles 5-8: dummy reads of the x coordinate from secondary OAM 4 times
                // for the first empty sprite slot, sprite 63's y coordinate followed by 3 0xFF bytes
                // for subsequent empty sprite slots, this will be four 0xFF bytes
                // 8 sprites, 8 cycles each
                let idx = 4 * self.store.read_sprite + self.store.read_sprite_byte;
                let is_empty = self.store.read_sprite >= self.store.accepted_sprite;
                if self.store.read_sprite != 8 {
                    match self.cycle % 8 {
                        1 => {
                            if is_empty {
                                if self.store.found_empty {
                                    self.sprites[self.store.read_sprite as usize].y_coordinate = 0xFF;
                                } else {
                                    self.store.found_empty = true;
                                    self.sprites[self.store.read_sprite as usize].y_coordinate = self.oam[4 * 63]; // sprite 63 y
                                }
                            } else {
                                self.sprites[self.store.read_sprite as usize].y_coordinate = self.secondary_oam[idx as usize];
                            }
                            self.store.read_sprite_byte = 1;
                        }
                        2 => {
                            self.sprites[self.store.read_sprite as usize].tile_number = self.secondary_oam[idx as usize];
                            self.store.read_sprite_byte = 2;
                        }
                        3 => {
                            self.sprites[self.store.read_sprite as usize].attributes = self.secondary_oam[idx as usize];
                            self.store.read_sprite_byte = 3;
                        }
                        4 => {
                            self.sprites[self.store.read_sprite as usize].x_coordinate = self.secondary_oam[idx as usize];
                            self.store.read_sprite = self.store.read_sprite.wrapping_add(1);
                            self.store.read_sprite_byte = 0;
                        }
                        5..=7 | 0 => {
                            let _ = self.secondary_oam[idx as usize - 1];
                        }
                        _ => unreachable!()
                    }
                }
            }
            320..=340 => {
                // TODO: initialize background render pipeline
                // read the first byte in secondary OAM
                // in parallel, the ppu fetches the first two background tiles for the next scanline
                let _ = self.secondary_oam[0];
            }
            _ => unreachable!(),
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
