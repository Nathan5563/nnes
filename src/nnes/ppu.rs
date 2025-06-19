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
        const CLIP_BACKGROUND = 0b0000_0010;    // 1: no clipping
        const CLIP_SPRITES = 0b0000_0100;       // 1: no clipping
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
    oam: [u8; 64 * 4], // 64 sprites of size 4 bytes each
    secondary_oam: [u8; 8 * 4],

    // 4 16-bit background shift registers
    tiles: u64,

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
    pub on_oam_dma: Box<dyn FnMut()>,

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
            tiles: 0,
            open_bus: 0,
            front: [0; 256 * 240],
            back: [0; 256 * 240],
            ppu_ctrl: PPUCTRL::empty(),
            ppu_mask: PPUMASK::empty(),
            ppu_status: PPUSTATUS::empty(),
            oam_addr: 0,
            read_buffer: 0,
            on_nmi: Box::new(|| {}),
            on_oam_dma: Box::new(|| {}),
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
        addr = if addr >= PALETTE_START {
            addr - 0x3F00
        } else {
            addr
        } & 0x1F;
        if addr >= 16 && addr % 4 == 0 {
            addr -= 16;
        }
        addr
    }

    fn handle_pre_render_line(&mut self) {
        if self
            .ppu_mask
            .intersects(PPUMASK::SHOW_BACKGROUND /* TODO: add sprites */)
        {
            if (280..=304).contains(&self.cycle) {
                self.copy_y();
            }
            self.handle_fetch_cycles();
        }

        // TODO: handle pre-render line evaluation
    }

    fn handle_render_lines(&mut self) {
        if self
            .ppu_mask
            .intersects(PPUMASK::SHOW_BACKGROUND /* TODO: add sprites */)
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
            self.tiles <<= 4;
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
        match self.cycle {
            0 => {}
            1..=64 => {
                // clear secondary OAM
                // hack memory accesses to simulate read -> write
                // one cycle "read", one cycle write
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
                // TODO: perform sprite evaluation
            }
            257..=320 => {
                // TODO: perform sprite fetches
            }
            321..=340 => {
                // TODO: initialize background render pipeline
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
