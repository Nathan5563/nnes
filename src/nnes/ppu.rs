use crate::{
    loader::{Cartridge, Mirroring},
    utils::{bit_0, bit_1},
};

bitflags! {
    struct PPUCTRL: u8 {
        const NAMETABLE1 = 0b0000_0001;
        const NAMETABLE2 = 0b0000_0010;
        const VRAM_INCREMENT = 0b0000_0100;
        const SPRITE_TABLE = 0b0000_1000;
        const BACKGROUND_TABLE = 0b0001_0000;
        const SPRITE_SIZE = 0b0010_0000;
        const MASTER_SLAVE = 0b0100_0000;
        const VBLANK_NMI = 0b1000_0000;
    }

    struct PPUMASK: u8 {
        const GRAYSCALE = 0b0000_0001;
        const SHOW_BACKGROUND = 0b0000_0010;
        const SHOW_SPRITES = 0b0000_0100;
        const BACKGROUND_RENDERING = 0b0000_1000;
        const SPRITE_RENDERING = 0b0001_0000;
        const EMPH_RED = 0b0010_0000;
        const EMPH_GREEN = 0b0100_0000;
        const EMPH_BLUE = 0b1000_0000;
    }

    struct PPUSTATUS: u8 {
        // get bits [0, 4] from ppu open bus
        const SPRITE_OVERFLOW = 0b0010_0000;
        const SPRITE0_HIT = 0b0100_0000;
        const VBLANK_FLAG = 0b1000_0000;
    }
}

impl PPUCTRL {
    pub fn write(&mut self, data: u8) {
        unimplemented!()
    }
}

impl PPUMASK {
    pub fn write(&mut self, data: u8) {
        unimplemented!()
    }
}

impl PPUSTATUS {
    pub fn read(&mut self) -> u8 {
        unimplemented!()
    }
}

pub struct PPU {
    // Architectural state
    v: u16,
    t: u16,
    x: u8,
    w: u8,
    chr_rom: Vec<u8>,
    vram: [u8; 0x800],
    palette: [u8; 0x20],
    oam: [u8; 0x100],

    // Background fetch & shift pipeline
    next_tile_id: u8,
    next_tile_attr: u8,
    next_tile_lobyte: u8,
    next_tile_hibyte: u8,
    bg_shifter_lo: u16,
    bg_shifter_hi: u16,
    attr_shifter_lo: u8,
    attr_shifter_hi: u8,

    // Sprite evaluation & shifters
    secondary_oam: [u8; 32],
    sprite_count: u8,
    sprite_shifter_lo: [u8; 8],
    sprite_shifter_hi: [u8; 8],
    sprite_x: [u8; 8],
    sprite_attr: [u8; 8],

    // Buffers
    read_buffer: u8,
    pub output_buffer: [u8; 256 * 240],

    // Open bus
    open_bus: u8,

    // I/O registers
    ppu_ctrl: PPUCTRL,
    ppu_mask: PPUMASK,
    ppu_status: PPUSTATUS,
    oam_addr: u8,

    // PPU metadata
    mirroring: Mirroring,
    odd_frame: bool,
    dot: u16,
    scanline: i16,

    // Debugging tools
    total_dots: u64,
    total_scanlines: u64,
    total_frames: u64,
}

impl PPU {
    pub fn new(cartridge: &Cartridge) -> Self {
        PPU {
            v: 0,
            t: 0,
            x: 0,
            w: 0,
            chr_rom: cartridge.chr_rom.clone(),
            vram: [0; 0x800],
            palette: [0; 0x20],
            oam: [0; 64 * 4],

            next_tile_id: 0,
            next_tile_attr: 0,
            next_tile_lobyte: 0,
            next_tile_hibyte: 0,
            bg_shifter_lo: 0,
            bg_shifter_hi: 0,
            attr_shifter_lo: 0,
            attr_shifter_hi: 0,

            secondary_oam: [0; 32],
            sprite_count: 0,
            sprite_shifter_lo: [0; 8],
            sprite_shifter_hi: [0; 8],
            sprite_x: [0; 8],
            sprite_attr: [0; 8],

            read_buffer: 0,
            output_buffer: [0; 256 * 240],

            open_bus: 0,

            ppu_ctrl: PPUCTRL::empty(),
            ppu_mask: PPUMASK::empty(),
            ppu_status: PPUSTATUS::VBLANK_FLAG.union(PPUSTATUS::SPRITE_OVERFLOW),
            oam_addr: 0,

            mirroring: cartridge.mirroring,
            odd_frame: false,
            dot: 0,
            scanline: 0,

            total_dots: 0,
            total_scanlines: 0,
            total_frames: 0,
        }
    }

    pub fn reset(&mut self) {}

    fn read_oam_data(&mut self) -> u8 {
        let data = self.oam[self.oam_addr as usize];
        self.oam_addr = self.oam_addr.wrapping_add(1);
        data
    }

    fn read_ppu_data(&mut self) -> u8 {
        unimplemented!()
    }

    fn write_oam_addr(&mut self, data: u8) {
        unimplemented!()
    }

    fn write_oam_data(&mut self, data: u8) {
        self.oam[self.oam_addr as usize] = data;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn write_ppu_scroll(&mut self, data: u8) {
        if self.w == 0 {
            let coarse_x = data >> 3; // top 5 bits -> tile column
            let fine_x = data & 0x07; // low 3 bits -> pixel offset

            // bits [0,4] = coarse X
            self.t = (self.t & !0x1F) | (coarse_x as u16);
            self.x = fine_x;
        } else {
            let coarse_y = data >> 3; // top 5 bits -> tile row
            let fine_y = data & 0x07; // low 3 bits -> pixel offset

            // bits [12,14] = fine Y, bits [5,9] = coarse Y
            self.t = (self.t & !0x73E0) | ((fine_y as u16) << 12) | ((coarse_y as u16) << 5);
        }
        self.w ^= 1;
    }

    pub fn write_ppu_addr(&mut self, data: u8) {
        if self.w == 0 {
            // set bits [13,8]
            self.t = (self.t & !0x3F00) | (((data as u16) & 0x3F) << 8);
        } else {
            // set bits [7,0]
            self.t = (self.t & !0x00FF) | (data as u16);
            self.v = self.t;
        }
        self.w ^= 1;
    }

    fn write_ppu_data(&mut self, data: u8) {
        unimplemented!()
    }

    pub fn reg_read(&mut self, reg: u8) -> u8 {
        match reg {
            2 => self.ppu_status.read(),
            4 => self.read_oam_data(),
            7 => self.read_ppu_data(),
            _ => unimplemented!(),
        }
    }

    pub fn reg_write(&mut self, reg: u8, data: u8) {
        match reg {
            0 => self.ppu_ctrl.write(data),
            1 => self.ppu_mask.write(data),
            3 => self.write_oam_addr(data),
            4 => self.write_oam_data(data),
            5 => self.write_ppu_scroll(data),
            6 => self.write_ppu_addr(data),
            7 => self.write_ppu_data(data),
            _ => unimplemented!(),
        }
    }

    pub fn peek(&self, reg: u8) -> u8 {
        match reg {
            2 => self.ppu_status.bits(),
            4 => self.oam[self.oam_addr as usize],
            7 => self.read_buffer,
            _ => unimplemented!(),
        }
    }

    pub fn tick(&mut self) {
        // //————————————————————————————————————————————————————————————————
        // //  Work for current dot
        // //————————————————————————————————————————————————————————————————
        // // Pre-render line start: clear flags
        // if self.scanline == 261 && self.dot == 1 {
        //     self.clear_vblank_and_sprite_flags();
        // }

        // // VBlank start: set flag & NMI
        // if self.scanline == 241 && self.dot == 1 {
        //     self.set_vblank_flag();
        //     if self.ppu_ctrl.contains(PPUCTRL::VBLANK_NMI) {
        //         (self.on_nmi)();
        //     }
        // }

        // // Background & sprite pipelines on visible & pre-render fetch ranges
        // if (0..=239).contains(&self.scanline) || self.scanline == 261 {
        //     // Background fetch + shift (dots 1–256, 321–336)
        //     if (1..=256).contains(&self.dot) || (321..=336).contains(&self.dot) {
        //         match self.dot % 8 {
        //             1 => self.fetch_name_table_byte(),
        //             3 => self.fetch_attribute_byte(),
        //             5 => self.fetch_pattern_low_byte(),
        //             7 => self.fetch_pattern_high_byte(),
        //             0 => {
        //                 self.load_bg_shifters();
        //                 self.increment_scroll_x();
        //             }
        //             _ => {}
        //         }
        //     }

        //     // End of scanline horizontal wrap at dot 256
        //     if self.dot == 256 {
        //         self.increment_scroll_y();
        //     }

        //     // Copy scroll bits into v at dot 257
        //     if self.dot == 257 {
        //         self.copy_horizontal_scroll();
        //     }

        //     // On pre-render line only, copy vertical bits at dots 280–304
        //     if self.scanline == 261 && (280..=304).contains(&self.dot) {
        //         self.copy_vertical_scroll();
        //     }
        // }

        // // Sprite evaluation & fetch (visible lines only)
        // if (0..=239).contains(&self.scanline) {
        //     // On dot 1 start secondary OAM
        //     if self.dot == 1 {
        //         self.clear_secondary_oam();
        //     }
        //     // Dots 1–256: scan primary OAM for this scanline
        //     if (1..=256).contains(&self.dot) {
        //         self.evaluate_sprite_for_cycle();
        //     }
        //     // Dots 257–320: fetch pattern bytes & fill sprite shifters
        //     if (257..=320).contains(&self.dot) {
        //         self.fetch_sprite_pattern_byte_for_cycle();
        //     }
        // }

        // // Pixel output & shifter advance (visible area only)
        // if (0..=239).contains(&self.scanline) && (1..=256).contains(&self.dot) {
        //     // Shift background & sprites
        //     self.shift_bg_shifters();
        //     self.shift_sprite_shifters();

        //     // Combine bg + sprite pixels, handle sprite-0 hit, palette lookup
        //     let color = self.pixel_mux_and_palette();
        //     let x = (self.dot - 1) as usize;
        //     let y = self.scanline as usize;
        //     self.output_buffer[y * 256 + x] = color;
        // }

        //————————————————————————————————————————————————————————————————
        //  Timing calculations
        //————————————————————————————————————————————————————————————————
        self.dot += 1;
        self.total_dots += 1;

        // Skip the last cycle of the pre-render line on odd frames
        if (self.odd_frame && self.scanline == 261 && self.dot == 340) || self.dot == 341 {
            self.dot = 0;
            self.scanline += 1;
        }

        // Finalize frame
        if self.scanline == 262 {
            self.scanline = 0;
            self.odd_frame = !self.odd_frame;
            self.total_frames += 1;
        }
    }
}
