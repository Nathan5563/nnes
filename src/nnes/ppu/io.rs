use super::{PPU, PPUCTRL, PPUMASK, PPUSTATUS, PRE_RENDER_LINE};
use crate::utils::{bit_0, bit_1, bit_2, bit_3, bit_4, byte_from_bits};

impl PPU {
    // 0x2000: PPUCTRL - W
    fn write_ppu_ctrl(&mut self, data: u8) {
        let prev = self.ppu_ctrl.contains(PPUCTRL::NMI_ON_VBLANK);
        self.ppu_ctrl = PPUCTRL::from_bits_truncate(data);

        // fire an NMI if NMI_ON_VBLANK is set during VBlank
        let now = self.ppu_ctrl.contains(PPUCTRL::NMI_ON_VBLANK)
            && self.ppu_status.contains(PPUSTATUS::IS_VBLANK);
        if !prev && now {
            self.nmi_signal = true;
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
        let open_bus = self.decayed_open_bus();
        let bit_4 = bit_4(open_bus);
        let bit_3 = bit_3(open_bus);
        let bit_2 = bit_2(open_bus);
        let bit_1 = bit_1(open_bus);
        let bit_0 = bit_0(open_bus);

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
        let rendering = self
            .ppu_mask
            .intersects(PPUMASK::SHOW_BACKGROUND | PPUMASK::SHOW_SPRITES);
        if rendering
            && (self.scanline < 240 || self.scanline == PRE_RENDER_LINE)
        {
            // While rendering, reads expose whatever the sprite pipeline is
            // currently accessing on the internal OAM bus (raw, unmasked)
            return self.oam_bus;
        }

        let mut data = self.oam[self.oam_addr as usize];
        if (self.oam_addr & 0b11) == 0b10 {
            data &= 0b11100011;
        }
        data
    }

    // 0x2004: OAMDATA - W
    fn write_oam_data(&mut self, data: u8) {
        let rendering = self
            .ppu_mask
            .intersects(PPUMASK::SHOW_BACKGROUND | PPUMASK::SHOW_SPRITES);
        if rendering
            && (self.scanline < 240 || self.scanline == PRE_RENDER_LINE)
        {
            // While rendering, writes don't reach OAM; they glitchily bump
            // the high 6 bits of OAMADDR instead
            self.oam_addr = self.oam_addr.wrapping_add(4);
            return;
        }

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
            // the finalized temporary address is copied into the vram
            // address ~3 dots after the write, not instantly
            self.v_pending = self.t;
            self.v_delay = 3;
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
            // access palette directly; the buffer is refilled from the
            // nametable byte "underneath" the palette
            self.read_buffer = self.mem_read(self.v - 0x1000);
            // bits 7-6 come from the PPU open bus, and greyscale mode
            // masks the palette value
            let mask = if self.ppu_mask.contains(PPUMASK::GRAYSCALE) {
                0x30
            } else {
                0x3F
            };
            data = (self.decayed_open_bus() & 0xC0) | (data & mask);
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
        match reg {
            2 => {
                // PPUSTATUS only drives bits 7-5 of the data bus
                let data = self.read_ppu_status();
                self.refresh_open_bus(data, 0xE0);
                data
            }
            4 => {
                let data = self.read_oam_data();
                self.refresh_open_bus(data, 0xFF);
                data
            }
            7 => {
                // palette reads leave bits 7-6 of the data bus undriven
                let palette_read = (self.v & 0x3FFF) >= 0x3F00;
                let data = self.read_ppu_data();
                let mask = if palette_read { 0x3F } else { 0xFF };
                self.refresh_open_bus(data, mask);
                data
            }
            // reading a write-only register returns the open bus contents
            _ => self.decayed_open_bus(),
        }
    }

    pub fn reg_write(&mut self, reg: u8, data: u8) {
        // Writes to PPUCTRL, PPUMASK, PPUSCROLL, and PPUADDR are ignored for
        // ~29658 CPU clocks (88974 PPU dots) after power/reset, but they
        // still drive the PPU data bus
        if self.total_cycles < 88974 && matches!(reg, 0 | 1 | 5 | 6) {
            self.refresh_open_bus(data, 0xFF);
            return;
        }

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
        self.refresh_open_bus(data, 0xFF);
    }

    // Helpers
    // For each bit set in mask, copy that bit of value onto the open bus
    // and restart its decay timer
    fn refresh_open_bus(&mut self, value: u8, mask: u8) {
        for bit in 0..8 {
            if mask & (1 << bit) != 0 {
                self.open_bus =
                    (self.open_bus & !(1 << bit)) | (value & (1 << bit));
                self.open_bus_decay[bit] = self.total_cycles;
            }
        }
    }

    // The open bus latch decays to 0 in ~600 ms (~3.2 million dots),
    // tracked per bit
    fn decayed_open_bus(&mut self) -> u8 {
        for bit in 0..8 {
            if self.total_cycles - self.open_bus_decay[bit] > 3_200_000 {
                self.open_bus &= !(1 << bit);
            }
        }
        self.open_bus
    }

    fn increment_v(&mut self) {
        if self.ppu_ctrl.contains(PPUCTRL::VRAM_INCREMENT) {
            self.v = self.v.wrapping_add(32);
        } else {
            self.v = self.v.wrapping_add(1);
        }
    }
}
