use super::{NAMETABLE_START, PPU, PPUCTRL, PPUMASK, PPUSTATUS};

impl PPU {
    pub fn draw_pixel(&mut self) {
        let x = self.cycle - 1; // this is called during cycles [1,256]
        let y = self.scanline;
        let idx = (y * 256 + x) as usize;

        // --- Background pixel ---
        let mut bg_pattern = 0u8;
        let mut bg_palette = 0u8;
        if self.ppu_mask.contains(PPUMASK::SHOW_BACKGROUND) {
            if x >= 8 || self.ppu_mask.contains(PPUMASK::NO_CLIP_BACKGROUND) {
                // Select pixel from pattern table shift registers using fine x scroll
                let bit_mux = 0x8000 >> self.x;
                let p1 = ((self.pattern_lo & bit_mux) > 0) as u8;
                let p2 = ((self.pattern_hi & bit_mux) > 0) as u8;

                // Select palette from attribute table shift registers
                let a1 = ((self.attribute_lo & bit_mux) > 0) as u8;
                let a2 = ((self.attribute_hi & bit_mux) > 0) as u8;

                bg_pattern = (p2 << 1) | p1;
                bg_palette = (a2 << 1) | a1;
            }
        }

        // --- Sprite pixel ---
        let mut sprite_pattern = 0u8;
        let mut sprite_palette = 0u8;
        let mut sprite_priority = false; // false = in front of BG, true = behind BG
        let mut sprite_0_visible = false;

        if self.ppu_mask.contains(PPUMASK::SHOW_SPRITES) {
            if x >= 8 || self.ppu_mask.contains(PPUMASK::NO_CLIP_SPRITES) {
                // Check each sprite (priority is by index - lower index = higher priority)
                for i in 0..8 {
                    let sprite = &self.sprite_render[i];
                    if sprite.x_counter == 0 {
                        // This sprite is active at this X position
                        // Get the leftmost pixel (bit 7)
                        let p1 = (sprite.pattern_lo >> 7) & 1;
                        let p2 = (sprite.pattern_hi >> 7) & 1;
                        let pattern = (p2 << 1) | p1;

                        if pattern != 0 && sprite_pattern == 0 {
                            // First non-transparent sprite pixel wins
                            sprite_pattern = pattern;
                            sprite_palette = sprite.attributes & 0x03;
                            sprite_priority = (sprite.attributes & 0x20) != 0;
                            if sprite.is_sprite_0 && i == 0 {
                                sprite_0_visible = true;
                            }
                        }
                    }
                }
            }
        }

        // --- Sprite 0 hit detection ---
        // Sprite 0 hit occurs when both sprite 0 and background are opaque
        // and we're not at x=255 (due to timing quirk)
        if sprite_0_visible && bg_pattern != 0 && x < 255 {
            if self.ppu_mask.contains(PPUMASK::SHOW_BACKGROUND)
                && self.ppu_mask.contains(PPUMASK::SHOW_SPRITES)
            {
                self.ppu_status.insert(PPUSTATUS::SPRITE0_HIT);
            }
        }

        // --- Multiplexing: determine final pixel color ---
        let color = if sprite_pattern != 0 {
            if bg_pattern == 0 || !sprite_priority {
                // Sprite wins: either BG is transparent, or sprite is in front
                0x10 | (sprite_palette << 2) | sprite_pattern
            } else {
                // BG wins: sprite is behind and BG is opaque
                (bg_palette << 2) | bg_pattern
            }
        } else if bg_pattern != 0 {
            (bg_palette << 2) | bg_pattern
        } else {
            0 // Universal background color
        };

        let palette_addr = self.get_palette_addr(color as u16);
        let mask = if self.ppu_mask.contains(PPUMASK::GRAYSCALE) {
            0x30
        } else {
            0x3F
        };
        let emphasis = (self.ppu_mask.bits() >> 5) as u16;
        self.back[idx] = (emphasis << 6)
            | (self.palette[palette_addr as usize] as u16 & mask);

        // --- Update sprite shift registers ---
        self.update_sprite_shifters();
    }

    fn update_sprite_shifters(&mut self) {
        for sprite in &mut self.sprite_render {
            if sprite.x_counter > 0 {
                sprite.x_counter -= 1;
            } else {
                // Sprite is active, shift pattern registers
                sprite.pattern_lo <<= 1;
                sprite.pattern_hi <<= 1;
            }
        }
    }

    pub fn draw_backdrop(&mut self) {
        let x = self.cycle - 1; // this is called during cycles [1,256]
        let y = self.scanline;
        let idx = (y * 256 + x) as usize;

        // Backdrop color, or the palette entry at v if v points into
        // palette space (the background palette hack)
        let palette_addr = if self.v & 0x3F00 == 0x3F00 {
            self.get_palette_addr(self.v)
        } else {
            0
        };
        let mask = if self.ppu_mask.contains(PPUMASK::GRAYSCALE) {
            0x30
        } else {
            0x3F
        };
        let emphasis = (self.ppu_mask.bits() >> 5) as u16;
        self.back[idx] = (emphasis << 6)
            | (self.palette[palette_addr as usize] as u16 & mask);
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
        // Copy vertical scroll bits from t to v: fine Y, nametable Y bit, coarse Y
        self.v &= 0b000_01_00000_11111;
        self.v |= self.t & 0b111_10_11111_00000;
    }

    pub fn copy_x(&mut self) {
        // Copy horizontal scroll bits from t to v: nametable X bit, coarse X
        self.v &= 0b111_10_11111_00000;
        self.v |= self.t & 0b000_01_00000_11111;
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
