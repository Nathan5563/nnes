use super::{NAMETABLE_START, PPU, PPUCTRL, PPUMASK, PPUSTATUS};

impl PPU {
    pub fn draw_pixel(&mut self) {
        let x = self.cycle - 1; // this is called during cycles [1,256]
        let y = self.scanline;
        // get pixel index based on current cycle and scanline
        let idx = (y * 256 + x) as usize;
        // ??? not sure what's going on beyond here
        let background = if x < 8 { 0 } else { 
            let mut data = 0;
            if self.ppu_mask.contains(PPUMASK::SHOW_BACKGROUND) {
                data = (self.tiles >> 32) as u32 >> ((7 - self.x) * 4);
                data &= 0x0F;
            }
            data as u8
         };
        let color = if background % 4 == 0 { 0 } else { background };
        let data = self.palette[self.get_palette_addr(color as u16) as usize];
        self.back[idx] = data % 64;
    }

    pub fn fetch_nametable(&mut self) {
        // get nametable offset from v[12:0]: ... NN YYYYY XXXXX
        let offset = self.v & 0b11_11111_11111;
        // final address: .10 NN YYYYY XXXXX
        let addr = NAMETABLE_START | offset;
        self.store.nametable_byte = self.mem_read(addr);
    }

    pub fn fetch_attribute(&mut self) {
        // get nametable select from v[12:11]: ... NN .... ... ...
        let nametable = self.v & 0b11_0000_000_000;
        // set attribute offset from nametable: ... .. 1111 ... ...
        let offset = 0b1111_000_000;
        // get high 3 bits of coarse y: ... .. .... YYY ...
        let coarse_y = (self.v >> 4) & 0b111_000;
        // get high 3 bits of coarse x: ... .. .... ... XXX
        let coarse_x = (self.v >> 2) & 0b111;
        // final address: .10 NN 1111 YYY XXX
        let addr = NAMETABLE_START | nametable | offset | coarse_y | coarse_x;
        let attr_byte = self.mem_read(addr);

        // Each attribute packs four 2-bit tiles:
        //     attr_byte[1:0]: top left
        //     attr_byte[3:2]: top right
        //     attr_byte[5:4]: bottom left
        //     attr_byte[7:6]: bottom right

        // get x quadrant from v[1]: .X
        let x_quad = (self.v >> 1) & 0b1;
        // get y quadrant from v[6]: Y.
        let y_quad = (self.v >> 5) & 0b10;
        // final quadrant index: YX
        let idx = y_quad | x_quad; // [0,3]

        let shift = idx << 1; // {0,2,4,6}
                              // final attribute: AA ..
        self.store.attribute_byte = ((attr_byte >> shift) & 0b11) << 2;
    }

    pub fn fetch_tile_lo(&mut self) {
        // get fine y from v[15:13]
        let fine_y = (self.v >> 12) & 0b111;
        // get base addr from PPUCTRL::BACKGROUND_PATTERN_TABLE: 0: 0x0000, 1: 0x1000
        let background_pattern_table_start = 0x1000
            * self.ppu_ctrl.contains(PPUCTRL::BACKGROUND_PATTERN_TABLE) as u16;
        // each entry in the nametable is a tile index, tiles are 16 bytes each (8 lo, 8 hi)
        let tile_offset = self.store.nametable_byte as u16 * 16;
        // final address: pattern_table_base + start of current tile data + fine_y for lo byte
        self.store.tile_addr =
            background_pattern_table_start + tile_offset + fine_y;
        self.store.tile_lo_byte = self.mem_read(self.store.tile_addr);
    }

    pub fn fetch_tile_hi(&mut self) {
        // must always be called after self.fetch_tile_lo()
        // final address: prev lo address + 8 for hi byte
        self.store.tile_hi_byte = self.mem_read(self.store.tile_addr + 8);
    }

    pub fn copy_y(&mut self) {
        // set v: ttt t. ttttt ..... from t: ttt t. ttttt .....
        self.v &= 0b000_01_00000_11111;
        self.v |= (self.t & 0b111_10_11111_00000);
    }

    pub fn copy_x(&mut self) {
        // set v: ... .t ..... ttttt from t: ... .t ..... ttttt
        self.v &= 0b111_10_11111_00000;
        self.v |= (self.t & 0b000_01_00000_11111);
    }

    pub fn increment_x(&mut self) {
        // check if all coarse x bits are set
        if (self.v & 0b11111) != 0b11111 {
            // if not, simple increment
            self.v += 1;
        } else {
            // if yes, zero out coarse x
            self.v &= 0b1_111_11_11111_00000;
            // switch horizontal nametable (wrapping add logic)
            self.v ^= 0b1_00000_00000;
        }
    }

    pub fn increment_y(&mut self) {
        // check if all fine y bits are set
        if (self.v & 0b111_00_00000_00000) != 0b111_00_00000_00000 {
            // if not, simple increment
            self.v += 0b1_00_00000_00000;
        } else {
            // if yes, zero out fine y
            self.v &= 0b1_000_11_11111_11111;
            // get coarse y from v[9:5]
            let mut coarse_y = (self.v & 0b11111_00000) >> 5;
            // increment coarse x (wrapping add logic)
            if coarse_y == 0b11101 {
                coarse_y = 0; // PPU uses 30 tiles, so max(coarse_y) = 0b11101
                self.v ^= 0b10_00000_00000; // switch vertical nametable
            } else if coarse_y == 0b11111 {
                coarse_y = 0; // coarse Y reset to 0, no switching
            } else {
                coarse_y += 1; // simple increment
            }
            // OR in coarse y to get final address
            self.v = (self.v & 0b1_111_11_00000_11111) | (coarse_y << 5);
        }
    }

    pub fn store_tiles(&mut self) {
        let mut data = 0;
        for _i in 0..8 {
            let a = self.store.attribute_byte;
            let p1 = (self.store.tile_lo_byte & 0x80) >> 7;
            let p2 = (self.store.tile_hi_byte & 0x80) >> 6;
            self.store.tile_lo_byte <<= 1;
            self.store.tile_hi_byte <<= 1;
            data <<= 4;
            data |= (a | p1 | p2) as u32;
        }
        self.tiles |= data as u64;
    }
}
