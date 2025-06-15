use super::{PPU, PPUCTRL, PPUMASK, PPUSTATUS};
use crate::utils::{bit_0, bit_1, bit_2, bit_3, bit_4, bit_7, byte_from_bits, hi_byte, lo_byte};

impl PPU {
    // 0x2000: PPUCTRL - W
    fn write_ppu_ctrl(&mut self, data: u8) {}

    // 0x2001: PPUMASK - W
    fn write_ppu_mask(&mut self, data: u8) {}

    // 0x2002: PPUSTATUS - R
    fn read_ppu_status(&mut self) -> u8 {
        // self.ppu_status.remove(PPUSTATUS::IS_VBLANK);
        // self.w = 0;
    }

    // 0x2003: OAMADDR - W
    fn write_oam_addr(&mut self, data: u8) {}

    // 0x2004: OAMDATA - R
    fn read_oam_data(&mut self) -> u8 {}

    // 0x2004: OAMDATA - W
    fn write_oam_data(&mut self, data: u8) {}

    // 0x2005: PPUSCROLL - W
    fn write_ppu_scroll(&mut self, data: u8) {}

    // 0x2006: PPUADDR - W
    fn write_ppu_addr(&mut self, data: u8) {}

    // 0x2007: PPUDATA - R
    fn read_ppu_data(&mut self) -> u8 {}

    // 0x2007: PPUDATA - R
    fn write_ppu_data(&mut self, data: u8) {}

    pub fn reg_read(&mut self, reg: u8) -> u8 {
        // Read from R/W and RONLY registers
    }

    pub fn reg_write(&mut self, reg: u8, data: u8) {
        // Write to R/W and WRONLY registers
    }

    pub fn reg_peek(&self, reg: u8) -> u8 {
        // Read from R/W and RONLY registers
        // Read last written byte from WRONLY registers
    }

    // Helpers
    fn increment_v(&mut self) {
        if self.ppu_ctrl.contains(PPUCTRL::VRAM_INCREMENT) {
            self.v = self.v.wrapping_add(32);
        } else {
            self.v = self.v.wrapping_add(1);
        }
    }
}
