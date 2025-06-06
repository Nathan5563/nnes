use super::{PPU, PPUCTRL, PPUMASK, PPUSTATUS};

impl PPU {
    fn fetch_tile_data(&self) -> u32 {
        (self.tiles >> 32) as u32
    }

    fn background_pixel(&self) -> u8 {
        if !self.ppu_mask.contains(PPUMASK::BACKGROUND_RENDERING) {
            return 0;
        }
        let data = self.fetch_tile_data() >> ((7 - self.x) * 4);
        return (data & 0x0F) as u8;
    }

    fn sprite_pixel(&self) -> (u8, u8) {
        if !self.ppu_mask.contains(PPUMASK::SPRITE_RENDERING) {
            return (0, 0);
        }
        for i in 0..self.sprite_count {
            let mut offset = (self.cycle - self.sprite_positions[i as usize] as u16) as i16;
            if offset < 0 || offset > 7 {
                continue;
            }
            offset = 7 - offset;
            let color = ((self.sprite_patterns[i as usize] >> ((offset * 4) as u8)) & 0x0F) as u8;
            if color % 4 == 0 {
                continue;
            }
            return (i as u8, color);
        }
        (0, 0)
    }

    pub fn draw_pixel(&mut self) {
        let x = self.cycle;
        let y = self.scanline as u16;
        let mut background = self.background_pixel();
        let (i, mut sprite) = self.sprite_pixel();

        if x < 8 && !self.ppu_mask.contains(PPUMASK::SHOW_BACKGROUND) {
            background = 0
        }
        if x < 8 && !self.ppu_mask.contains(PPUMASK::SHOW_SPRITES) {
            sprite = 0
        }

        let b = background % 4 != 0;
        let s = sprite % 4 != 0;
        let color;
        if !b && !s {
            color = 0
        } else if !b && s {
            color = sprite | 0x10
        } else if b && !s {
            color = background
        } else {
            if self.sprite_indices[i as usize] == 0 && x < 255 {
                self.ppu_status.insert(PPUSTATUS::SPRITE0_HIT);
            }
            if self.sprite_priorities[i as usize] == 0 {
                color = sprite | 0x10
            } else {
                color = background
            }
        }
        let idx = (y * 256 + x) as usize;
        let data = self.palette[self.palette_addr(color as u16) as usize];
        self.output_buffer[idx] = data % 64;
    }

    pub fn copy_x(&mut self) {
        self.v = (self.v & 0xFBE0) | (self.t & 0x041F);
    }

    pub fn copy_y(&mut self) {
        self.v = (self.v & 0x841F) | (self.t & 0x7BE0);
    }

    pub fn increment_x(&mut self) {
        // if coarse X == 31
        if self.v & 0x001F == 31 {
            // coarse X = 0
            self.v &= 0xFFE0;
            // switch horizontal nametable
            self.v ^= 0x0400;
        } else {
            // increment coarse X
            self.v += 1;
        }
    }

    pub fn increment_y(&mut self) {
        // if fine Y < 7
        if self.v & 0x7000 != 0x7000 {
            // increment fine Y
            self.v += 0x1000;
        } else {
            // fine Y = 0
            self.v &= 0x8FFF;
            // let y = coarse Y
            let mut y = (self.v & 0x03E0) >> 5;
            if y == 29 {
                // coarse Y = 0
                y = 0;
                // switch vertical nametable
                self.v ^= 0x0800;
            } else if y == 31 {
                // coarse Y = 0, nametable not switched
                y = 0;
            } else {
                // increment coarse Y
                y += 1;
            }
            // put coarse Y back into v
            self.v = (self.v & 0xFC1F) | (y << 5)
        }
    }

    pub fn fetch_nametable(&mut self) {
        let addr = 0x2000 | (self.v & 0x0FFF);
        self.nametable = self.mem_read(addr);
    }

    pub fn fetch_attribute_table(&mut self) {
        let addr = 0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07);
        let shift = ((self.v >> 4) & 4) | (self.v & 2);
        self.attribute_table = ((self.mem_read(addr) >> shift) & 3) << 2;
    }

    pub fn fetch_low_tile(&mut self) {
        let fine_y = (self.v >> 12) & 7;
        let table = self.ppu_ctrl.contains(PPUCTRL::BACKGROUND_TABLE);
        let tile = self.nametable;
        let addr = 0x1000 * table as u16 + tile as u16 * 16 + fine_y;
        self.low_tile = self.mem_read(addr);
    }

    pub fn fetch_high_tile(&mut self) {
        let fine_y = (self.v >> 12) & 7;
        let table = self.ppu_ctrl.contains(PPUCTRL::BACKGROUND_TABLE);
        let tile = self.nametable;
        let addr = 0x1000 * table as u16 + tile as u16 * 16 + fine_y;
        self.high_tile = self.mem_read(addr + 8)
    }

    pub fn store_tiles(&mut self) {
        // if self.scanline < 240 {
        //     println!(
        //         "[PPU] store_tiles @ scanline={}, cycle={}  (v={:04X})",
        //         self.scanline, self.cycle, self.v
        //     );
        // }

        let mut data = 0;
        for _i in 0..8 {
            let a = self.attribute_table;
            let p1 = (self.low_tile & 0x80) >> 7;
            let p2 = (self.high_tile & 0x80) >> 6;
            self.low_tile <<= 1;
            self.high_tile <<= 1;
            data <<= 4;
            data |= (a | p1 | p2) as u32;
        }
        self.tiles |= data as u64;
    }

    fn fetch_sprite_pattern(&mut self, i: usize, mut row: i16) -> u32 {
        let mut tile = self.oam[i * 4 + 1];
        let attributes = self.oam[i * 4 + 2];
        let addr;
        if self.ppu_ctrl.contains(PPUCTRL::SPRITE_SIZE) {
            if attributes & 0x80 == 0x80 {
                row = 15 - row;
            }
            let table = tile & 1;
            tile &= 0xFE;
            if row > 7 {
                tile += 1;
                row -= 8
            }
            addr = 0x1000 * table as u16 + tile as u16 * 16 + row as u16;
        } else {
            if attributes & 0x80 == 0x80 {
                row = 7 - row;
            }
            let table = self.ppu_ctrl.contains(PPUCTRL::SPRITE_TABLE);
            addr = 0x1000 * table as u16 + tile as u16 * 16 + row as u16
        }

        let a = (attributes & 3) << 2;
        let mut low_tile = self.mem_read(addr);
        let mut high_tile = self.mem_read(addr + 8);
        let mut data = 0;
        for _i in 0..8 {
            let p1;
            let p2;
            if attributes & 0x40 == 0x40 {
                p1 = (low_tile & 1) << 0;
                p2 = (high_tile & 1) << 1;
                low_tile >>= 1;
                high_tile >>= 1;
            } else {
                p1 = (low_tile & 0x80) >> 7;
                p2 = (high_tile & 0x80) >> 6;
                low_tile <<= 1;
                high_tile <<= 1;
            }

            data <<= 4;
            data |= (a | p1 | p2) as u32;
        }

        data
    }

    pub fn evaluate_sprites(&mut self) {
        let h;
        if self.ppu_ctrl.contains(PPUCTRL::SPRITE_SIZE) {
            h = 16;
        } else {
            h = 8;
        }

        let mut count = 0;
        for i in 0..64 {
            let y = self.oam[i * 4];
            let a = self.oam[i * 4 + 2];
            let x = self.oam[i * 4 + 3];
            let row = (self.scanline - y as u16) as i16;
            if row < 0 || row >= h {
                continue;
            }

            if count < 8 {
                self.sprite_patterns[count] = self.fetch_sprite_pattern(i, row);
                self.sprite_positions[count] = x;
                self.sprite_priorities[count] = (a >> 5) & 1;
                self.sprite_indices[count] = i as u8;
            }

            count += 1;
        }

        if count > 8 {
            count = 8;
            self.ppu_status.insert(PPUSTATUS::SPRITE_OVERFLOW);
        }

        self.sprite_count = count as i32;
    }
}
