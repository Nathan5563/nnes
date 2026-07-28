use super::{SpriteEvalState, NAMETABLE_START, PPU, PPUCTRL, PPUSTATUS};

impl PPU {
    pub fn reset_sprite_state(&mut self) {
        self.store.curr_sprite = 0;
        self.store.accepted_sprite = 0;
        self.store.curr_sprite_byte = 0;
        self.store.read_sprite = 0;
        self.store.read_sprite_byte = 0;
        self.store.found_empty = false;
        self.store.sprite_0_in_secondary = false;
        self.store.sprite_eval_state = SpriteEvalState::YCoordinate;
    }

    fn sprite_height(&self) -> u8 {
        // PPUCTRL bit 5 is sampled live wherever the height is used, not
        // latched once per scanline
        if self.ppu_ctrl.contains(PPUCTRL::SPRITE_SIZE) {
            16
        } else {
            8
        }
    }

    pub fn tick_sprite_evaluation(&mut self) {
        // Sprite evaluation algorithm (cycles 65-256):
        // Odd cycles: Read from primary OAM[n][m]
        // Even cycles: Write to secondary OAM (if not full) and advance state
        //
        // For each sprite n (0-63):
        //   1. Read Y coordinate, check if sprite is on next scanline
        //   2. If in range, copy all 4 bytes to secondary OAM
        //   3. If 8 sprites found, set overflow flag on additional in-range sprites
        //   4. The overflow detection has a hardware bug where m also increments

        // Evaluation begins at the current OAMADDR, not at OAM[0]: latch the
        // base on the first evaluation tick. The entry at the base acts as
        // "sprite 0" for sprite-0-hit purposes (OAMADDR misalignment quirk).
        if self.cycle == 65 {
            self.oam_addr_base = self.oam_addr;
        }

        let oam_idx = self.oam_addr_base as usize
            + 4 * self.store.curr_sprite as usize
            + self.store.curr_sprite_byte as usize;
        let secondary_idx = 4 * self.store.accepted_sprite as usize
            + self.store.curr_sprite_byte as usize;

        // No more sprites are found once the end of OAM is reached, which
        // hides any sprites before the starting OAMADDR
        if oam_idx > 0xFF {
            self.store.sprite_eval_state = SpriteEvalState::Done;
            return;
        }

        if self.cycle % 2 == 1 {
            if self.store.sprite_eval_state == SpriteEvalState::Done {
                return;
            }
            // Odd cycle: Read from primary OAM
            let data = self.oam[oam_idx];
            self.oam_bus = data;
            match self.store.sprite_eval_state {
                SpriteEvalState::YCoordinate => self.store.y_coordinate = data,
                SpriteEvalState::TileNumber => self.store.tile_number = data,
                SpriteEvalState::Attributes => self.store.attributes = data,
                SpriteEvalState::XCoordinate => self.store.x_coordinate = data,
                SpriteEvalState::Done => {}
            }
        } else {
            // Even cycle: Write to secondary OAM and advance
            if self.store.sprite_eval_state == SpriteEvalState::Done {
                return;
            }

            // Write to secondary OAM if not full
            if self.store.accepted_sprite < 8 {
                self.secondary_oam[secondary_idx] = match self
                    .store
                    .sprite_eval_state
                {
                    SpriteEvalState::YCoordinate => self.store.y_coordinate,
                    SpriteEvalState::TileNumber => self.store.tile_number,
                    SpriteEvalState::Attributes => self.store.attributes,
                    SpriteEvalState::XCoordinate => self.store.x_coordinate,
                    SpriteEvalState::Done => unreachable!(),
                };
            }

            // Check if sprite is in range: Y is compared against the current
            // scanline, and the fetched sprite is rendered on the next one
            let y = self.store.y_coordinate as u16;
            let in_range = self.scanline >= y
                && self.scanline < y + self.sprite_height() as u16;

            if in_range {
                // Overflow: with 8 sprites already accepted, the flag is set
                // the moment the 9th in-range Y coordinate is found, before
                // the byte walk over the remaining 3 bytes continues
                if self.store.accepted_sprite >= 8
                    && self.store.sprite_eval_state
                        == SpriteEvalState::YCoordinate
                {
                    self.ppu_status.insert(PPUSTATUS::SPRITE_OVERFLOW);
                }

                // Sprite is visible on next scanline - copy remaining bytes
                let (next_byte, next_state) = match self.store.curr_sprite_byte
                {
                    0 => (1, SpriteEvalState::TileNumber),
                    1 => (2, SpriteEvalState::Attributes),
                    2 => (3, SpriteEvalState::XCoordinate),
                    3 => (0, SpriteEvalState::YCoordinate),
                    _ => unreachable!(),
                };
                self.store.curr_sprite_byte = next_byte;
                self.store.sprite_eval_state = next_state;

                // Finished copying this sprite (byte 3 -> 0)
                if self.store.curr_sprite_byte == 0 {
                    self.store.curr_sprite =
                        self.store.curr_sprite.wrapping_add(1);
                    if self.store.curr_sprite == 64 {
                        self.store.sprite_eval_state = SpriteEvalState::Done;
                    }

                    if self.store.accepted_sprite < 8 {
                        // Track sprite 0 presence (curr_sprite just incremented to 1)
                        if self.store.curr_sprite == 1
                            && self.store.accepted_sprite == 0
                        {
                            self.store.sprite_0_in_secondary = true;
                        }
                        self.store.accepted_sprite += 1;
                    }
                }
            } else {
                // Sprite not in range - move to next sprite
                self.store.curr_sprite =
                    self.store.curr_sprite.wrapping_add(1);
                if self.store.curr_sprite == 64 {
                    self.store.sprite_eval_state = SpriteEvalState::Done;
                }

                // Hardware bug: when checking for overflow, m also increments
                if self.store.accepted_sprite >= 8 {
                    self.store.curr_sprite_byte =
                        (self.store.curr_sprite_byte + 1) & 3;
                }
            }
        }
    }

    pub fn tick_sprite_fetch(&mut self) {
        // Cycles 257-320: Fetch sprite tile data for next scanline
        // 8 sprites × 8 cycles each:
        //   Cycles 0-1: Garbage nametable fetch; Y read from secondary OAM
        //   Cycles 2-3: Garbage nametable fetch; tile, attributes, X read
        //   Cycles 4-5: Fetch pattern table low byte
        //   Cycles 6-7: Fetch pattern table high byte, finalize sprite

        let sprite_idx = ((self.cycle - 257) / 8) as u8;
        let cycle_in_sprite = (self.cycle - 257) % 8;
        let secondary_idx = (sprite_idx as usize) * 4;
        let is_empty = sprite_idx >= self.store.accepted_sprite;

        match cycle_in_sprite {
            0 => {
                // Garbage nametable fetch (bus activity only)
                let _ = self.mem_read(NAMETABLE_START | (self.v & 0x0FFF));

                // Read Y coordinate (or special values for empty slots)
                self.store.y_coordinate = if is_empty {
                    if !self.store.found_empty {
                        self.store.found_empty = true;
                        self.oam[4 * 63] // First empty uses sprite 63's Y
                    } else {
                        0xFF // Subsequent empty slots use $FF
                    }
                } else {
                    self.secondary_oam[secondary_idx]
                };
                self.oam_bus = self.store.y_coordinate;
            }
            1 => {
                self.store.tile_number = if is_empty {
                    0xFF
                } else {
                    self.secondary_oam[secondary_idx + 1]
                };
                self.oam_bus = self.store.tile_number;
            }
            2 => {
                // Garbage nametable fetch (bus activity only)
                let _ = self.mem_read(NAMETABLE_START | (self.v & 0x0FFF));

                self.store.attributes = if is_empty {
                    0xFF
                } else {
                    self.secondary_oam[secondary_idx + 2]
                };
                self.oam_bus = self.store.attributes;
            }
            3 => {
                self.store.x_coordinate = if is_empty {
                    0xFF
                } else {
                    self.secondary_oam[secondary_idx + 3]
                };
                self.oam_bus = self.store.x_coordinate;
            }
            5 => {
                // Compute the pattern address and fetch the low plane
                self.fetch_sprite_pattern_lo(is_empty);
            }
            7 => {
                // Fetch the high plane and finalize the sprite slot
                self.fetch_sprite_pattern_hi(sprite_idx, is_empty);
            }
            4 | 6 => {} // Address setup cycles
            _ => unreachable!(),
        }
    }

    fn fetch_sprite_pattern_lo(&mut self, is_empty: bool) {
        let y = self.store.y_coordinate;
        let tile = self.store.tile_number;
        let attr = self.store.attributes;
        let flip_v = (attr & 0x80) != 0;
        let sprite_height = self.sprite_height();

        // Row within the sprite: fetches on scanline N target scanline N+1,
        // and row 0 of a sprite with coordinate Y lands on scanline Y+1.
        // Empty slots fetch row 0 of tile $FF so the bus pattern matches
        // hardware even though the fetched data is discarded.
        let row = if is_empty {
            0
        } else {
            // Only the low row bits address the tile: masking keeps the row
            // in bounds if the sprite height changed after evaluation
            let raw_row = self.scanline.wrapping_sub(y as u16) as u8;
            if sprite_height == 16 {
                let raw_row = raw_row & 15;
                if flip_v {
                    15 - raw_row
                } else {
                    raw_row
                }
            } else {
                let raw_row = raw_row & 7;
                if flip_v {
                    7 - raw_row
                } else {
                    raw_row
                }
            }
        };

        // Calculate pattern table address
        let pattern_addr = if sprite_height == 16 {
            // 8x16 sprites: tile bit 0 selects bank, bits 1-7 are top tile index
            let bank = (tile & 1) as u16 * 0x1000;
            let top_tile = (tile & 0xFE) as u16;
            let (tile_idx, row_in_tile) = if row >= 8 {
                (top_tile + 1, row - 8)
            } else {
                (top_tile, row)
            };
            bank + tile_idx * 16 + row_in_tile as u16
        } else {
            // 8x8 sprites: PPUCTRL bit 3 selects pattern table
            let bank = if self.ppu_ctrl.contains(PPUCTRL::SPRITE_PATTERN_TABLE)
            {
                0x1000
            } else {
                0x0000
            };
            bank + (tile as u16) * 16 + row as u16
        };
        self.store.sprite_pattern_addr = pattern_addr;

        // Fetch the low plane; empty slots store zeroes (transparent)
        let lo = self.mem_read(pattern_addr);
        self.store.sprite_pattern_lo = if is_empty { 0 } else { lo };
    }

    fn fetch_sprite_pattern_hi(&mut self, sprite_idx: u8, is_empty: bool) {
        let attr = self.store.attributes;
        let x = self.store.x_coordinate;
        let flip_h = (attr & 0x40) != 0;

        // Fetch the high plane; empty slots store zeroes (transparent)
        let hi_read = self.mem_read(self.store.sprite_pattern_addr + 8);
        let mut lo = self.store.sprite_pattern_lo;
        let mut hi = if is_empty { 0 } else { hi_read };
        if flip_h {
            lo = lo.reverse_bits();
            hi = hi.reverse_bits();
        }

        // Store in sprite render buffer
        let render = &mut self.sprite_render[sprite_idx as usize];
        render.pattern_lo = lo;
        render.pattern_hi = hi;
        render.attributes = attr;
        render.x_counter = x;
        render.is_sprite_0 =
            sprite_idx == 0 && self.store.sprite_0_in_secondary;

        // Finalize sprite state after last sprite
        if sprite_idx == 7 {
            self.sprite_count = self.store.accepted_sprite;
            self.sprite_0_on_line = self.store.sprite_0_in_secondary;
        }
    }
}
