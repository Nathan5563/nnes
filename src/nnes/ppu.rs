mod registers;
mod render;

use registers::{PPUCTRL, PPUMASK, PPUSTATUS};

use crate::cartridge::{Cartridge, Mirroring};

pub struct PPU {
    // Architectural state
    v: u16, // 15 bits
    t: u16, // 15 bits
    x: u8,
    w: u8,
    chr_rom: Vec<u8>,
    vram: [u8; 0x1F00],
    palette: [u8; 0x20],
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

    // PPU metadata
    mirroring: Mirroring,
    odd_frame: bool,
    dot: u16,
    scanline: i16,

    // Debugging tools
    total_dots: u64,
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
            chr_rom: cartridge.chr_rom.clone(),
            vram: [0; 0x1F00],
            palette: [0; 0x20],
            oam: [0; 64 * 4],

            read_buffer: 0xFF,
            output_buffer: [0; 256 * 240],

            open_bus: 0,

            ppu_ctrl: PPUCTRL::empty(),
            ppu_mask: PPUMASK::empty(),
            ppu_status: PPUSTATUS::VBLANK_FLAG.union(PPUSTATUS::SPRITE_OVERFLOW),
            oam_addr: 0,

            mirroring: cartridge.mirroring,
            odd_frame: false,
            dot: 0,
            scanline: 0,

            total_dots: 0,
            total_scanlines: 0,
            total_frames: 0,
        }
    }

    pub fn reset(&mut self) {
        self.dot = 340;
        self.scanline = 240;
        self.reg_write(0, 0);
        self.reg_write(1, 0);
        self.reg_write(3, 0);
    }

    fn mem_read(&self, mut addr: u16) -> u8 {
        addr &= 0x3FFF;
        match addr {
            0x0000..0x2000 => {
                self.chr_rom[addr as usize]
            }
            0x2000..0x3F00 => {
                addr = self.mirror_addr(addr) & 0x7FF;
                self.vram[addr as usize]
            }
            0x3F00..0x4000 => {
                addr = (addr - 0x3F00) & 0x1F;
                if addr >= 16 && addr & 0x3 == 0 {
                    addr -= 16;
                }
                self.palette[addr as usize]
            }
            _ => unreachable!()
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
                addr = (addr - 0x3F00) & 0x1F;
                if addr >= 16 && addr & 0x3 == 0 {
                    addr -= 16;
                }
                self.palette[addr as usize] = data;
            }
            _ => unreachable!()
        };
    }

    fn peek(&self, addr: u16) -> u8 {
        self.mem_read(addr)
    }
    
    pub fn tick(&mut self) {
        //————————————————————————————————————————————————————————————————
        //  TODO: Work for current dot
        //————————————————————————————————————————————————————————————————
        

        //————————————————————————————————————————————————————————————————
        //  Timing calculations
        //————————————————————————————————————————————————————————————————
        self.dot += 1;
        self.total_dots += 1;

        // Skip the last cycle of the pre-render line on odd frames
        if (self.odd_frame && self.scanline == 261 && self.dot == 340) || self.dot == 341 {
            self.dot = 0;
            self.scanline += 1;
        }

        // Finalize frame
        if self.scanline == 262 {
            self.scanline = 0;
            self.odd_frame = !self.odd_frame;
            self.total_frames += 1;
        }
    }

    // Helpers
    fn mirror_addr(&self, mut addr: u16) -> u16 {
        // This folds any address in 0x2000..0x2FFF (or 0x3000..0x3EFF)
        // to the correct 2KB VRAM page, according to the cartridge's mirroring.
        //   Mirroring::VERTICAL   => tables {0,2} -> NT0,  {1,3} -> NT1
        //   Mirroring::HORIZONTAL => tables {0,1} -> NT0,  {2,3} -> NT1
        
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
}
