use std::{fs, iter};

const NES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

// Currently only supports iNES file format, mapper 0.
// Validation is not rigorous yet, so be careful with rom selection.
pub fn validate(arg: &String) -> Result<Vec<u8>, String> {
    // No file
    let rom = match fs::read(arg) {
        Ok(rom) => rom,
        Err(_) => return Err("error: invalid path to rom".to_string()),
    };

    // No magic number
    for i in 0..4 {
        if rom[i] != NES_MAGIC[i] {
            return Err("error: not a nes rom".to_string());
        }
    }

    // Not iNES file format
    if rom[7] & 0xc != 0 {
        return Err("error: unsupported file format".to_string());
    }

    // Not mapper 0
    let lo = (rom[6] & 0xf0) >> 4;
    let hi = rom[7] & 0xf0;
    let mapper = hi | lo;
    if mapper != 0 {
        return Err("error: unsupported mapper".to_string());
    }

    Ok(rom)
}

pub struct Cartridge {
    pub has_trainer: bool,
    pub has_sram: bool,

    pub sram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
}

impl Cartridge {
    pub fn new(rom: &Vec<u8>) -> Self {
        /*  iNES file sections, in order:
            - Header,               16 B
            - Trainer,              0 or 512 B
            - PRG ROM Data,         16 * x kB, min 16 kB
            - CHR ROM Data,         8 * y kB, min 8 kB
            - PlayChoice INST-ROM,  0 or 8 kB
            - PlayChoice PROM,      0 or 32 B

            Header bytes:
            - [0,3]    = 0x4E 0x45 0x53 0x1A
            - 4         = x, non-zero
            - 5         = y, game uses CHR RAM if 0
            - 6         = see Flags 6 below
            - 7         = see Flags 7 below
            - 8         = see Flags 8 below
            - 9         = see Flags 9 below
            - 10        = see Flags 10 below
            - [11,15]  = unused padding

            Flags 6 bits:
            - 0         = 0b0: vertical arrangement ("horizontal mirrored"), vice versa
            - 1         = 0b1: contains SRAM at [0x6000, 0x8000)
            - 2         = 0b1: contains trainer at [0x7000, 0x7200)
            - 3         = 0b1: alternative nametable layout
            - [7,4]    = lower nibble of mapper number

            Flags 7 bits:
            - 0         = 0b1: VS Unisystem
            - 1         = 0b1: PlayChoice-10
            - [3,2]     = 0b10: Flags [8, 15] are in NES 2.0 format
            - [7,4]     = upper nibble of mapper number

            Flags 8 bits:
            - [7,0] * 8 kB  = size of SRAM

            Flags 9 bits:
            - 0         = TV system (0: NTSC, 1: PAL)
            - [7,1]     = reserved, set to 0

            Flags 10 bits:
            - [1,0]     = TV system (0: NTSC, 2: PAL, 1/3: dual compatible)
            - 4         = 0b1: contains SRAM at [0x6000, 0x8000)
            - 5         = 0b1: has bus conflicts
        */

        let prg_rom_size = 0x4000 * usize::max(1, rom[4] as usize);
        let chr_rom_size = 0x2000 * usize::max(1, rom[5] as usize);
        let sram_size = 0x2000;
        let trainer_size = if rom[6] & 0b100 != 0 { 512 } else { 0 };
        let prg_start = 16 + trainer_size;
        let chr_start = prg_start + prg_rom_size;
        let mapper_lo = (rom[6] & 0xf0) >> 4;
        let mapper_hi = rom[7] & 0xf0;

        let sram: Vec<u8> = iter::repeat(0u8)
            .take(0x1000)
            .chain(rom[16..16 + trainer_size].iter().cloned())
            .chain(iter::repeat(0u8).take(sram_size - 0x1000 - trainer_size))
            .collect();
        let prg_rom = rom[prg_start..prg_start + prg_rom_size].to_vec();
        let chr_rom = rom[chr_start..chr_start + chr_rom_size].to_vec();
        let mapper = mapper_hi | mapper_lo;

        Cartridge {
            has_trainer: trainer_size != 0,
            has_sram: rom[6] & 0b10 != 0,

            sram,
            prg_rom,
            chr_rom,
            mapper,
        }
    }
}
