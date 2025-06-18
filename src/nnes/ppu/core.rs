use super::{NAMETABLE_START, PPU, PPUCTRL, PPUMASK, PPUSTATUS};

impl PPU {
    pub fn draw_pixel(&mut self) {
        // TODO
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
        // TODO: vert(v) = vert(t)
    }

    pub fn copy_x(&mut self) {
        // TODO: hori(v) = hori(t)
    }

    pub fn increment_x(&mut self) {
        // TODO: inc(hori(v))
    }

    pub fn increment_y(&mut self) {
        // TODO: inc(vert(v))
    }
}
