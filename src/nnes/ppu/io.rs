use super::{PPU, PPUCTRL, PPUMASK, PPUSTATUS};
use crate::utils::{
    bit_0, bit_1, bit_2, bit_3, bit_4, bit_7, byte_from_bits, hi_byte, lo_byte,
};

impl PPU {
    // 0x2000: PPUCTRL - W
    fn write_ppu_ctrl(&mut self, data: u8) {
        let prev = self.ppu_ctrl.contains(PPUCTRL::NMI_ON_VBLANK);
        self.ppu_ctrl = PPUCTRL::from_bits_truncate(data);

        // fire an NMI if NMI_ON_VBLANK is set during VBlank
        let now = self.ppu_ctrl.contains(PPUCTRL::NMI_ON_VBLANK)
            && self.ppu_status.contains(PPUSTATUS::IS_VBLANK);
        if !prev && now {
            (self.on_nmi.as_mut())();
            self.nmi_prev = true;
        }

        // get nametable select from data[1:0]: ... NN ..... .....
        let nametable = (data as u16 & 0b11) << 10;
        // final address: ttt NN ttttt ttttt
        self.t = (self.t & 0b1_111_00_11111_11111) | nametable;
    }

    // 0x2001: PPUMASK - W
    fn write_ppu_mask(&mut self, data: u8) {
        self.ppu_mask = PPUMASK::from_bits_truncate(data);
    }

    // 0x2002: PPUSTATUS - R
    fn read_ppu_status(&mut self) -> u8 {
        let bit_7 = self.ppu_status.contains(PPUSTATUS::IS_VBLANK) as u8;
        let bit_6 = self.ppu_status.contains(PPUSTATUS::SPRITE0_HIT) as u8;
        let bit_5 = self.ppu_status.contains(PPUSTATUS::SPRITE_OVERFLOW) as u8;
        let bit_4 = bit_4(self.open_bus);
        let bit_3 = bit_3(self.open_bus);
        let bit_2 = bit_2(self.open_bus);
        let bit_1 = bit_1(self.open_bus);
        let bit_0 = bit_0(self.open_bus);

        // side effects of reading PPUSTATUS
        self.ppu_status.remove(PPUSTATUS::IS_VBLANK);
        self.w = 0;
        self.nmi_prev = false;

        byte_from_bits(bit_7, bit_6, bit_5, bit_4, bit_3, bit_2, bit_1, bit_0)
    }

    // 0x2003: OAMADDR - W
    fn write_oam_addr(&mut self, data: u8) {
        self.oam_addr = data;
    }

    // 0x2004: OAMDATA - R
    fn read_oam_data(&mut self) -> u8 {
        let mut data = self.oam[self.oam_addr as usize];
        if (self.oam_addr & 0b11) == 0b10 {
            data &= 0b11100011;
        }
        data
    }

    // 0x2004: OAMDATA - W
    fn write_oam_data(&mut self, data: u8) {
        self.oam[self.oam_addr as usize] = data;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    // 0x2005: PPUSCROLL - W
    fn write_ppu_scroll(&mut self, data: u8) {
        if self.w == 0 {
            // get fine x from data[2:0]: xxx
            self.x = data & 0b111;
            // get coarse x from data[7:3]: ... .. ..... XXXXX
            let coarse_x = data as u16 >> 3;
            // final address: ttt tt ttttt XXXXX
            self.t = (self.t & 0b1_111_11_11111_00000) | coarse_x;
        } else {
            // get coarse y from data[7:3]: ... .. YYYYY .....
            let coarse_y = (data as u16 >> 3) << 5;
            // get fine y from data[2:0]: yyy .. ..... .....
            let fine_y = (data as u16 & 0b111) << 12;
            // final address: yyy tt YYYYY ttttt
            self.t = (self.t & 0b1_000_11_00000_11111) | coarse_y | fine_y;
        }
        self.w ^= 1;
    }

    // 0x2006: PPUADDR - W
    fn write_ppu_addr(&mut self, data: u8) {
        if self.w == 0 {
            // get hi "byte" of addr from data[5:0]: .hh hh hh... .....
            let hi = (data as u16 & 0b111111) << 8;
            // pull bit 14 to 0, final address: 0hh hh hh... .....
            self.t = (self.t & 0b1_000_00_00111_11111) | hi;
        } else {
            // get lo byte of addr from data[7:0]: ... .. ..lll lllll
            let lo = data as u16;
            // final address: ttt tt ttlll lllll
            self.t = (self.t & 0b1_111_11_11000_00000) | lo;
            // copy finalized temporary address into vram address
            self.v = self.t;
        }
        self.w ^= 1;
    }

    // 0x2007: PPUDATA - R
    fn read_ppu_data(&mut self) -> u8 {
        let mut data = self.mem_read(self.v);
        if self.v & 0x3FFF < 0x3F00 {
            // buffer non-palette reads
            let buf = self.read_buffer;
            self.read_buffer = data;
            data = buf;
        } else {
            // access palette directly
            self.read_buffer = self.mem_read(self.v - 0x1000);
        }
        self.increment_v();
        data
    }

    // 0x2007: PPUDATA - W
    fn write_ppu_data(&mut self, data: u8) {
        self.mem_write(self.v, data);
        self.increment_v();
    }

    // Public register APIs
    pub fn reg_read(&mut self, reg: u8) -> u8 {
        let data = match reg {
            2 => self.read_ppu_status(),
            4 => self.read_oam_data(),
            7 => self.read_ppu_data(),
            _ => self.open_bus,
        };
        self.open_bus = data;
        data
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
            _ => {}
        }
        self.open_bus = data;
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
