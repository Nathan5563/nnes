mod registers;
mod render;

use crate::cartridge::{Cartridge, Mirroring};

bitflags! {
    pub struct PPUCTRL: u8 {
        const NAMETABLE1 = 0b0000_0001;
        const NAMETABLE2 = 0b0000_0010;
        const VRAM_INCREMENT = 0b0000_0100;
        const SPRITE_TABLE = 0b0000_1000;
        const BACKGROUND_TABLE = 0b0001_0000;
        const SPRITE_SIZE = 0b0010_0000;
        const MASTER_SLAVE = 0b0100_0000;
        const VBLANK_NMI = 0b1000_0000;
    }

    pub struct PPUMASK: u8 {
        const GRAYSCALE = 0b0000_0001;
        const SHOW_BACKGROUND = 0b0000_0010;
        const SHOW_SPRITES = 0b0000_0100;
        const BACKGROUND_RENDERING = 0b0000_1000;
        const SPRITE_RENDERING = 0b0001_0000;
        const EMPH_RED = 0b0010_0000;
        const EMPH_GREEN = 0b0100_0000;
        const EMPH_BLUE = 0b1000_0000;
    }

    pub struct PPUSTATUS: u8 {
        // get bits [0, 4] from ppu open bus
        const SPRITE_OVERFLOW = 0b0010_0000;
        const SPRITE0_HIT = 0b0100_0000;
        const VBLANK_NMI = 0b1000_0000;
    }
}

pub struct PPU {
    // Architectural state
    v: u16, // 15 bits
    t: u16, // 15 bits
    x: u8,
    w: u8, // 1 bit
    f: u8, // 1 bit
    chr_rom: Vec<u8>,
    vram: [u8; 0x1F00],
    pub palette: [u8; 0x20],
    oam: [u8; 64 * 4],

    // Buffers
    read_buffer: u8,
    pub output_buffer: [u8; 256 * 240],

    // Open bus
    open_bus: u8,

    // I/O registers
    ppu_ctrl: PPUCTRL,
    ppu_mask: PPUMASK,
    ppu_status: PPUSTATUS,
    oam_addr: u8,

    // NMI handling
    nmi_request: bool,
    nmi_previous: bool,
    nmi_delay: u8,
    pub on_nmi: Option<Box<dyn FnMut()>>,

    // PPU metadata
    mirroring: Mirroring,
    cycle: u16,
    scanline: u16,

    // Background data
    nametable: u8,
    attribute_table: u8,
    low_tile: u8,
    high_tile: u8,
    tiles: u64,

    // Sprite data
    sprite_count: i32,
    sprite_patterns: [u32; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indices: [u8; 8],

    // Debugging tools
    debug_buffer: u8,
    total_cycles: u64,
    total_scanlines: u64,
    total_frames: u64,
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
            vram: [0; 0x1F00],
            palette: [0; 0x20],
            oam: [0; 64 * 4],

            read_buffer: 0xFF,
            output_buffer: [0; 256 * 240],

            open_bus: 0,

            ppu_ctrl: PPUCTRL::empty(),
            ppu_mask: PPUMASK::empty(),
            ppu_status: PPUSTATUS::empty(),
            oam_addr: 0,

            nmi_request: false,
            nmi_previous: false,
            nmi_delay: 0,
            on_nmi: None,

            mirroring: cartridge.mirroring,
            cycle: 0,
            scanline: 0,

            nametable: 0,
            attribute_table: 0,
            low_tile: 0,
            high_tile: 0,
            tiles: 0,

            sprite_count: 0,
            sprite_patterns: [0; 8],
            sprite_positions: [0; 8],
            sprite_priorities: [0; 8],
            sprite_indices: [0; 8],

            debug_buffer: 0,
            total_cycles: 0,
            total_scanlines: 0,
            total_frames: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.scanline = 261;
        self.reg_write(0, 0);
        self.reg_write(1, 0);
        self.reg_write(3, 0);
    }

    fn mem_read(&self, mut addr: u16) -> u8 {
        addr &= 0x3FFF;
        match addr {
            0x0000..0x2000 => self.chr_rom[addr as usize],
            0x2000..0x3F00 => {
                addr = self.mirror_addr(addr) & 0x7FF;
                self.vram[addr as usize]
            }
            0x3F00..0x4000 => {
                addr = self.palette_addr((addr - 0x3F00) & 0x1F);
                self.palette[addr as usize]
            }
            _ => unreachable!(),
        }
    }

    fn mem_write(&mut self, mut addr: u16, data: u8) {
        addr &= 0x3FFF;
        match addr {
            0x0000..0x2000 => {
                self.chr_rom[addr as usize] = data;
            }
            0x2000..0x3F00 => {
                addr = self.mirror_addr(addr - 0x2000) & 0x7FF;
                self.vram[addr as usize] = data;
            }
            0x3F00..0x4000 => {
                addr = self.palette_addr((addr - 0x3F00) & 0x1F);
                self.palette[addr as usize] = data;
            }
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
        // println!(
        //     "[PPU] tick() @ scanline={}, cycle={}, total_cycles={}",
        //     self.scanline, self.cycle, self.total_cycles
        // );
        self.poll_nmi();

        let can_render = self
            .ppu_mask
            .contains(PPUMASK::BACKGROUND_RENDERING | PPUMASK::SPRITE_RENDERING);
        let prerender_line = self.scanline == 261;
        let visible_line = self.scanline < 240;
        let render_line = prerender_line || visible_line;
        let prefetch_cycle = (320..336).contains(&self.cycle);
        let visible_cycle = (0..256).contains(&self.cycle);
        let fetch_cycle = prefetch_cycle || visible_cycle;

        // println!(
        //     "[PPU] scanline={} cycle={}  can_render={}  render_line={}  fetch_cycle={}",
        //     self.scanline, self.cycle, can_render, render_line, fetch_cycle
        // );

        // Background
        if can_render {
            if visible_line && visible_cycle {
                self.draw_pixel();
            }
            if render_line && fetch_cycle {
                self.tiles <<= 4;
                match (self.cycle + 1) % 8 {
                    1 => self.fetch_nametable(),
                    3 => self.fetch_attribute_table(),
                    5 => self.fetch_low_tile(),
                    7 => self.fetch_high_tile(),
                    0 => self.store_tiles(),
                    _ => {}
                }
            }
            if prerender_line && (279..304).contains(&self.cycle) {
                self.copy_y();
            }
            if render_line {
                if fetch_cycle && (self.cycle + 1) % 8 == 0 {
                    self.increment_x();
                }
                if self.cycle == 255 {
                    self.increment_y();
                }
                if self.cycle == 256 {
                    self.copy_x();
                }
            }
        }

        // Sprites
        if can_render {
            if self.cycle == 256 {
                if visible_line {
                    self.evaluate_sprites()
                } else {
                    self.sprite_count = 0
                }
            }
        }

        // Vblank
        if self.scanline == 241 && self.cycle == 0 {
            // println!(
            //     "[PPU] VBlank rising at scanline=241, cycle=1;  PPUCTRL={:02X}",
            //     self.ppu_ctrl.bits()
            // );
            self.ppu_status.insert(PPUSTATUS::VBLANK_NMI);
            self.nmi_change();
        }
        if prerender_line && self.cycle == 0 {
            self.tiles = 0;
            self.ppu_status.remove(PPUSTATUS::VBLANK_NMI);
            self.nmi_change();
            self.ppu_status.remove(PPUSTATUS::SPRITE0_HIT);
            self.ppu_status.remove(PPUSTATUS::SPRITE_OVERFLOW);
        }

        //————————————————————————————————————————————————————————————————
        //  Timing calculations
        //————————————————————————————————————————————————————————————————
        self.update_timing();
    }

    // Helpers
    fn mirror_addr(&self, mut addr: u16) -> u16 {
        addr &= 0xFFF;
        let table = addr / 0x400;
        let offset = addr & 0x3FF;

        let mirrored = match self.mirroring {
            Mirroring::VERTICAL => table & 1,
            Mirroring::HORIZONTAL => table >> 1,
            Mirroring::ALTERNATIVE => unimplemented!(),
        };

        (mirrored << 10) + offset
    }

    fn palette_addr(&self, mut addr: u16) -> u16 {
        if addr >= 16 && addr % 4 == 0 {
            addr -= 16;
        }
        addr
    }

    fn poll_nmi(&mut self) {
        if self.nmi_delay > 0 {
            // println!(
            //     "[PPU] poll_nmi(): nmi_delay was {}, nmi_request={}, VBlank={} at scanline={}, cycle={}",
            //     self.nmi_delay,
            //     self.nmi_request,
            //     self.ppu_status.contains(PPUSTATUS::VBLANK_NMI),
            //     self.scanline,
            //     self.cycle
            // );
            self.nmi_delay -= 1;
            if self.nmi_delay == 0
                && self.nmi_request
                && self.ppu_status.contains(PPUSTATUS::VBLANK_NMI)
            {
                // println!("[PPU] about to call on_nmi() now");
                (self.on_nmi.as_mut().unwrap())();
            }
        }
    }

    fn nmi_change(&mut self) {
        let nmi = self.nmi_request && self.ppu_status.contains(PPUSTATUS::VBLANK_NMI);
        // println!(
        //     "[PPU] nmi_change(): nmi_request={}, VBLANK_NMI={}, nmi_previous={}",
        //     self.nmi_request,
        //     self.ppu_status.contains(PPUSTATUS::VBLANK_NMI),
        //     self.nmi_previous
        // );
        if nmi && !self.nmi_previous {
            self.nmi_delay = 2;
            // println!("[PPU] nmi_delay set to 2");
        }
        self.nmi_previous = nmi;
    }

    fn update_timing(&mut self) {
        self.cycle += 1;
        self.total_cycles += 1;

        // Skip the last cycle of the pre-render line on odd frames
        // Reset after the last cycle
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
