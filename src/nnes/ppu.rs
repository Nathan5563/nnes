mod registers;
mod render;

use crate::cartridge::{Cartridge, Mirroring};

const PATTERN_TABLE_START: u16 = 0x0000;
const PATTERN_TABLE_END: u16 = 0x1FFF;
const NAMETABLE_START: u16 = 0x2000;
const NAMETABLE_END: u16 = 0x3EFF;
const PALETTE_START: u16 = 0x3F00;
const PALETTE_END: u16 = 0x3FFF;

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
        const CLIP_BACKGROUND = 0b0000_0010;    // 1: No clipping
        const CLIP_SPRITES = 0b0000_0100;       // 1: No clipping
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
    oam: [u8; 64 * 4],

    // Buses
    address_bus: u8,
    data_bus: u8,

    // Image buffers
    pub front: [u8; 256 * 240],
    pub back: [u8; 256 * 240],

    // I/O operations
    ppu_ctrl: PPUCTRL,
    ppu_mask: PPUMASK,
    ppu_status: PPUSTATUS,
    oam_addr: u8,
    read_buffer: u8,

    // PPU metadata
    pub on_nmi: Box<dyn FnMut()>,
    mirroring: Mirroring,
    pub cycle: u16,
    pub scanline: u16,
    store: PPUStore,

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

            address_bus: 0,
            data_bus: 0,

            front: [0; 256 * 240],
            back: [0; 256 * 240],

            ppu_ctrl: PPUCTRL::empty(),
            ppu_mask: PPUMASK::empty(),
            ppu_status: PPUSTATUS::empty(),
            oam_addr: 0,
            read_buffer: 0xFF,

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

            total_cycles: 0,
            total_scanlines: 0,
            total_frames: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.scanline = 241;
    }

    fn mem_read(&self, mut addr: u16) -> u8 {
        addr &= 0x3FFF;
        match addr {
            PATTERN_TABLE_START..=PATTERN_TABLE_END => {}
            NAMETABLE_START..=NAMETABLE_END => {}
            PALETTE_START..=PALETTE_END => {}
            _ => unreachable!(),
        }
    }

    fn mem_write(&mut self, mut addr: u16, data: u8) {
        addr &= 0x3FFF;
        match addr {
            PATTERN_TABLE_START..=PATTERN_TABLE_END => {}
            NAMETABLE_START..=NAMETABLE_END => {}
            PALETTE_START..=PALETTE_END => {}
            _ => unreachable!(),
        };
    }

    fn peek(&self, addr: u16) -> u8 {
        self.mem_read(addr)
    }

    pub fn tick(&mut self) {
        //————————————————————————————————————————————————————————————————
        //  TODO: Work for current cycle
        //————————————————————————————————————————————————————————————————
        let prerender_line = self.scanline == 261;
        let visible_line = (0..=239).contains(&self.scanline);
        let fetch_line = prerender_line || visible_line;
        let vblank_line = (241..=260).contains(&self.scanline);
        let fetch_cycle = (1..=340).contains(&self.cycle);
        let can_render = self
            .ppu_mask
            .intersects(PPUMASK::SHOW_BACKGROUND | PPUMASK::SHOW_SPRITES);

        // Handle nmi polling here

        if can_render {
            if fetch_line && fetch_cycle {
                match self.cycle & 0x7 {
                    1 => self.fetch_nametable(),
                    3 => self.fetch_attribute(),
                    5 => self.fetch_tile_lo(),
                    7 => self.fetch_tile_hi(),
                    _ => {}
                }
            } else if vblank_line {
                if self.cycle == 1 {
                    self.ppu_status.insert(PPUSTATUS::IS_VBLANK);
                    (self.on_nmi.as_mut())();
                }
            }
        }

        //————————————————————————————————————————————————————————————————
        //  Timing calculations
        //————————————————————————————————————————————————————————————————
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
        addr = (addr - 0x3F00) & 0x1F;
        if addr >= 0x10 && addr & 0x3 == 0 {
            addr -= 0x10;
        }
        addr
    }

    fn update_timing(&mut self) {
        self.cycle += 1;
        self.total_cycles += 1;

        // Skip the last cycle of the pre-render line on odd frames
        if (self.f == 1 && self.scanline == 261 && self.cycle == 340) || self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;
            self.total_scanlines += 1;
        }

        // Finalize frame
        if self.scanline == 262 {
            self.scanline = 0;
            self.f ^= 1;
            self.total_frames += 1;
        }
    }
}
