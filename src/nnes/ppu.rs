mod registers;
mod render;

use super::CPU;
use crate::cartridge::{Cartridge, Mirroring};
use registers::{PPUCTRL, PPUMASK, PPUSTATUS};
use std::{cell::RefCell, rc::Rc};

pub struct PPU {
    // Architectural state
    v: u16, // 15 bits
    t: u16, // 15 bits
    x: u8,
    w: u8, // 1 bit
    f: u8, // 1 bit
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

    // NMI handling
    nmi_request: bool,
    nmi_previous: bool,
    nmi_delay: u8,
    pub on_nmi: Option<Box<dyn FnMut()>>,

    // PPU metadata
    mirroring: Mirroring,
    cycle: u16,
    scanline: i16,

    // Debugging tools
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
            ppu_status: PPUSTATUS::SPRITE_OVERFLOW,
            oam_addr: 0,

            nmi_request: false,
            nmi_previous: false,
            nmi_delay: 0,
            on_nmi: None,

            mirroring: cartridge.mirroring,
            cycle: 0,
            scanline: 0,

            total_cycles: 0,
            total_scanlines: 0,
            total_frames: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 340;
        self.scanline = 240;
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
                addr = (addr - 0x3F00) & 0x1F;
                if addr >= 16 && addr & 0x3 == 0 {
                    addr -= 16;
                }
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
                addr = (addr - 0x3F00) & 0x1F;
                if addr >= 16 && addr & 0x3 == 0 {
                    addr -= 16;
                }
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
        self.poll_nmi();

        //————————————————————————————————————————————————————————————————
        //  Timing calculations
        //————————————————————————————————————————————————————————————————
        self.update_timing();
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

    fn poll_nmi(&mut self) {
        if self.nmi_delay > 0 {
            self.nmi_delay -= 1;
            if self.nmi_delay == 0
                && self.nmi_request
                && self.ppu_status.contains(PPUSTATUS::VBLANK_NMI)
            {
                (self.on_nmi.as_mut().unwrap())();
            }
        }
    }

    fn update_timing(&mut self) {
        self.cycle += 1;
        self.total_cycles += 1;

        // Skip the last cycle of the pre-render line on odd frames
        // Reset after the last cycle
        if (self.f == 1 && self.scanline == 261 && self.cycle == 340) || self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;
        }

        // Finalize frame
        if self.scanline == 262 {
            self.scanline = 0;
            self.f ^= 1;
            self.total_frames += 1;
        }
    }
}
