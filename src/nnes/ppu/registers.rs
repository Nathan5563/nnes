use super::PPU;
use crate::utils::{
    bit_0, bit_1, bit_2, bit_3, bit_4, 
    byte_from_bits, hi_byte, lo_byte
};

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
        const VBLANK_FLAG = 0b1000_0000;
    }
}    

impl PPU {
    // 0x2000
    fn write_ppu_ctrl(&mut self, data: u8) {
        self.ppu_ctrl = PPUCTRL::from_bits_truncate(data);
        // TODO: check if nmi flag changed
        self.t = (self.t & 0xF3FF) | (data as u16 & 0x03) << 10;
    }

    // 0x2001
    fn write_ppu_mask(&mut self, data: u8) {
        self.ppu_mask = PPUMASK::from_bits_truncate(data);
    }

    // 0x2002
    fn read_ppu_status(&mut self) -> u8 {
        // TODO: investigate bit 7 and how nmi checking should follow here
        let bit_7 = self.ppu_status.contains(PPUSTATUS::VBLANK_FLAG) as u8;
        let bit_6 = self.ppu_status.contains(PPUSTATUS::SPRITE0_HIT) as u8;
        let bit_5 = self.ppu_status.contains(PPUSTATUS::SPRITE_OVERFLOW) as u8;
        let bit_4 = bit_4(self.open_bus);
        let bit_3 = bit_3(self.open_bus);
        let bit_2 = bit_2(self.open_bus);
        let bit_1 = bit_1(self.open_bus);
        let bit_0 = bit_0(self.open_bus);

        // Side effect of reading 0x2002
        self.w = 0;

        byte_from_bits(
            bit_7, bit_6, bit_5, bit_4, 
            bit_3, bit_2, bit_1, bit_0
        )
    }

    // 0x2003
    fn write_oam_addr(&mut self, data: u8) {
        self.oam_addr = data;
    }

    // 0x2004
    fn read_oam_data(&mut self) -> u8 {
        let mut data = self.oam[self.oam_addr as usize];
        if (self.oam_addr & 0x03) == 0x02 {
            // zero out the bits that are not driven on OAM attribute read
            data = data & 0xE3;
        }
        data
    }

    // 0x2004
    fn write_oam_data(&mut self, data: u8) {
        self.oam[self.oam_addr as usize] = data;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    // 0x2005
    fn write_ppu_scroll(&mut self, data: u8) {
        if self.w == 0 {
            let coarse_x = data >> 3;
            let fine_x = data & 0x07;

            // replace bits [0,4] with coarse X
            self.t = (self.t & !0x1F) | (coarse_x as u16);
            self.x = fine_x;
        } else {
            let coarse_y = data >> 3;
            let fine_y = data & 0x07;

            //  replace [12,14] with fine Y, bits [5,9] with coarse Y
            self.t = (self.t & !0x73E0) | 
                ((fine_y as u16) << 12) | 
                ((coarse_y as u16) << 5);
        }
        self.w ^= 1;
    }

    // 0x2006
    fn write_ppu_addr(&mut self, data: u8) {
        if self.w == 0 {
            // replace bits [13,8], set bit 14 = 0
            self.t = u16::from_le_bytes([lo_byte(self.t), data & 0x3F]);
        } else {
            // replace bits [7,0]
            self.t = u16::from_le_bytes([data, hi_byte(self.t)]);
            self.v = self.t;
        }
        self.w ^= 1;
    }

    // 0x2007
    fn read_ppu_data(&mut self) -> u8 {
        let mut data = self.mem_read(self.v);
        if self.v & 0x3FFF < 0x3F00 {
            let buf = self.read_buffer;
            self.read_buffer = data;
            data = buf;
        } else {
            self.read_buffer = self.mem_read(self.v - 0x1000);
        }
        self.increment_v();
        data
    }

    // 0x2007
    fn write_ppu_data(&mut self, data: u8) {
        self.mem_write(self.v, data);
        self.increment_v();
    }

    pub fn reg_read(&mut self, reg: u8) -> u8 {
        match reg {
            2 => self.read_ppu_status(),
            4 => self.read_oam_data(),
            7 => self.read_ppu_data(),
            _ => unimplemented!(),
        }
    }

    pub fn reg_write(&mut self, reg: u8, data: u8) {
        match reg {
            0 => self.write_ppu_ctrl(data),
            1 => self.write_ppu_mask(data),
            3 => self.write_oam_addr(data),
            4 => self.write_oam_data(data),
            5 => self.write_ppu_scroll(data),
            6 => self.write_ppu_addr(data),
            7 => self.write_ppu_data(data),
            _ => unimplemented!(),
        }
    }

    pub fn reg_peek(&self, reg: u8) -> u8 {
        match reg {
            2 => self.ppu_ctrl.bits(),
            4 => self.oam[self.oam_addr as usize],
            7 => self.peek(self.v),
            _ => unimplemented!(),
        }
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
