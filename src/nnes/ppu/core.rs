use super::{NAMETABLE_START, PPU, PPUCTRL, PPUMASK, PPUSTATUS};

impl PPU {
    pub fn draw_pixel(&mut self) {
        let x = self.cycle - 1; // this is called during cycles [1,256]
        let y = self.scanline;
        let idx = (y * 256 + x) as usize;

        // Background rendering
        let mut background = 0u8;
        if self.ppu_mask.contains(PPUMASK::SHOW_BACKGROUND) {
            if x >= 8 || self.ppu_mask.contains(PPUMASK::NO_CLIP_BACKGROUND) {
                // Select pixel from pattern table shift registers using fine x scroll
                let bit_mux = 0x8000 >> self.x;
                let p1 = ((self.pattern_lo & bit_mux) > 0) as u8;
                let p2 = ((self.pattern_hi & bit_mux) > 0) as u8;
                // Select palette from attribute table shift registers
                let a1 = ((self.attribute_lo & bit_mux) > 0) as u8;
                let a2 = ((self.attribute_hi & bit_mux) > 0) as u8;
                let pattern = (p2 << 1) | p1;
                let attribute = (a2 << 1) | a1;
                // A pattern of 0 means the background color is used
                if pattern != 0 {
                    background = (attribute << 2) | pattern;
                }
            }
        }

        // Sprite rendering
        let (sprite_index, mut sprite_color) = self.get_sprite_pixel(x);

        // Left edge clipping for sprites
        if x < 8 && !self.ppu_mask.contains(PPUMASK::NO_CLIP_SPRITES) {
            sprite_color = 0;
        }

        // Pixel multiplexing logic - optimized
        let background_opaque = background & 3 != 0;
        let sprite_opaque = sprite_color & 3 != 0;

        let final_color = if !background_opaque && !sprite_opaque {
            // Both transparent - use universal background color
            0
        } else if !background_opaque && sprite_opaque {
            // Background transparent, sprite opaque - use sprite
            sprite_color | 0x10 // Sprite palette base
        } else if background_opaque && !sprite_opaque {
            // Background opaque, sprite transparent - use background
            background
        } else {
            // Both opaque - priority and sprite 0 hit logic

            // Sprite 0 hit detection (sprite_index represents the original sprite slot)
            if sprite_index == 0 && x < 255 && x != 0 {
                self.ppu_status.insert(PPUSTATUS::SPRITE0_HIT);
            }

            // Check sprite priority (bit 5 of sprite attributes)
            let sprite_priority =
                (self.sprite_attributes[sprite_index as usize] >> 5) & 1;
            if sprite_priority == 0 {
                // Sprite has priority - use sprite
                sprite_color | 0x10
            } else {
                // Background has priority - use background
                background
            }
        };

        // Get final palette color
        let palette_addr = self.get_palette_addr(final_color as u16);
        self.back[idx] = self.palette[palette_addr as usize] % 64;
    }

    fn get_sprite_pixel(&self, x: u16) -> (u8, u8) {
        if !self.ppu_mask.contains(PPUMASK::SHOW_SPRITES) {
            return (0, 0);
        }
        for i in 0..8 {
            let sprite_y = self.sprite_y_coords[i];
            if sprite_y == 0xFF {
                continue;
            }
            let sprite_x = self.sprite_x_coords[i] as u16;
            let offset = x.wrapping_sub(sprite_x);
            if offset > 7 {
                continue;
            }
            let pattern = self.sprite_patterns[i];
            // No flip logic here! Always extract left-to-right
            let color = ((pattern >> (offset * 4)) & 0x0F) as u8;
            if color == 0 {
                continue;
            }
            let attributes = self.sprite_attributes[i];
            let palette = attributes & 0x03;
            let final_color = (palette << 2) | color;
            return (i as u8, final_color);
        }
        (0, 0)
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
        self.store.attribute_byte = (attr_byte >> shift) & 0b11;
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
        self.pattern_lo =
            (self.pattern_lo & 0xFF00) | self.store.tile_lo_byte as u16;
        self.pattern_hi =
            (self.pattern_hi & 0xFF00) | self.store.tile_hi_byte as u16;
        self.attribute_lo = (self.attribute_lo & 0xFF00)
            | if self.store.attribute_byte & 0b01 != 0 {
                0xFF
            } else {
                0x00
            };
        self.attribute_hi = (self.attribute_hi & 0xFF00)
            | if self.store.attribute_byte & 0b10 != 0 {
                0xFF
            } else {
                0x00
            };
    }
}
