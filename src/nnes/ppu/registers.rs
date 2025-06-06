use super::{PPU, PPUCTRL, PPUMASK, PPUSTATUS};
use crate::utils::{bit_0, bit_1, bit_2, bit_3, bit_4, bit_7, byte_from_bits, hi_byte, lo_byte};

impl PPU {
    // 0x2000
    fn write_ppu_ctrl(&mut self, data: u8) {
        // println!(
        //     "[PPU] write_ppu_ctrl called with data={:02X}, old_ctrl={:02X}",
        //     data,
        //     self.ppu_ctrl.bits()
        // );

        self.ppu_ctrl = PPUCTRL::from_bits_truncate(data);
        self.nmi_request = if bit_7(data) != 0 { true } else { false };
        self.nmi_change();
        self.t = (self.t & 0xF3FF) | (data as u16 & 0x03) << 10;
    }

    // 0x2001
    fn write_ppu_mask(&mut self, data: u8) {
        self.ppu_mask = PPUMASK::from_bits_truncate(data);
    }

    // 0x2002
    fn read_ppu_status(&mut self) -> u8 {
        let bit_7 = self.ppu_status.contains(PPUSTATUS::VBLANK_NMI) as u8;
        self.ppu_status.remove(PPUSTATUS::VBLANK_NMI);
        self.nmi_change();

        let bit_6 = self.ppu_status.contains(PPUSTATUS::SPRITE0_HIT) as u8;
        let bit_5 = self.ppu_status.contains(PPUSTATUS::SPRITE_OVERFLOW) as u8;
        let bit_4 = bit_4(self.open_bus);
        let bit_3 = bit_3(self.open_bus);
        let bit_2 = bit_2(self.open_bus);
        let bit_1 = bit_1(self.open_bus);
        let bit_0 = bit_0(self.open_bus);

        // Side effect of reading 0x2002
        self.w = 0;

        let res = byte_from_bits(bit_7, bit_6, bit_5, bit_4, bit_3, bit_2, bit_1, bit_0);

        // println!(
        //     "[PPU] read_ppu_status @ scanline={}, cycle={} → returns ${:02X}",
        //     self.scanline, self.cycle, res
        // );

        res
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
            self.t = (self.t & !0x73E0) | ((fine_y as u16) << 12) | ((coarse_y as u16) << 5);
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

    // I/O Register API
    pub fn reg_read(&mut self, reg: u8) -> u8 {
        let data = match reg {
            2 => self.read_ppu_status(),
            4 => self.read_oam_data(),
            7 => self.read_ppu_data(),
            _ => unimplemented!(),
        };
        self.open_bus = data;
        data
    }

    pub fn reg_write(&mut self, reg: u8, data: u8) {
        self.debug_buffer = data;
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
        self.open_bus = data;
    }

    pub fn reg_peek(&self, reg: u8) -> u8 {
        match reg {
            0 => self.ppu_ctrl.bits(),
            1 => self.ppu_mask.bits(),
            2 => self.ppu_status.bits(),
            3 => self.oam_addr,
            4 => self.oam[self.oam_addr as usize],
            5 => self.debug_buffer,
            6 => self.debug_buffer,
            7 => self.peek(self.v),
            _ => unreachable!(),
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
