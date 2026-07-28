use crate::utils::{bit_0, bit_1, bit_3, byte_from_nibbles, hi_nibble};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    ALTERNATIVE,
}

const NES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

// Currently only supports mapper 0 (iNES, archaic iNES, or NES 2.0 headers).
pub fn validate_rom(rom: &[u8]) -> Result<u8, String> {
    // The header alone is 16 bytes
    if rom.len() < 16 {
        return Err("error: file too small".to_string());
    }

    // No magic number
    for i in 0..4 {
        if rom[i] != NES_MAGIC[i] {
            return Err("error: not a nes rom".to_string());
        }
    }

    // Header variant: 0b10 in bits [3,2] of byte 7 marks NES 2.0, 0b01 marks
    // archaic iNES; both are loadable for mapper 0. Reject only 0b11.
    let nes2 = rom[7] & 0x0C == 0x08;
    let archaic = rom[7] & 0x0C == 0x04;
    if rom[7] & 0x0C == 0x0C {
        return Err("error: unsupported file format".to_string());
    }

    // Not mapper 0. Archaic iNES headers have garbage in byte 7, so the
    // mapper high nibble is only meaningful in the other variants; NES 2.0
    // additionally stores mapper bits [11,8] in the low nibble of byte 8.
    let lo = hi_nibble(rom[6]);
    let hi = if archaic { 0 } else { hi_nibble(rom[7]) };
    let mapper = byte_from_nibbles(lo, hi);
    if mapper != 0 {
        return Err("error: unsupported mapper".to_string());
    }
    if nes2 && rom[8] & 0x0F != 0 {
        return Err("error: unsupported mapper".to_string());
    }
    // NES 2.0 byte 9 extends the PRG/CHR sizes past what NROM can hold
    if nes2 && rom[9] != 0 {
        return Err("error: unsupported ROM size".to_string());
    }

    // NROM has 16 or 32 kB of PRG ROM
    if rom[4] == 0 || rom[4] > 2 {
        return Err("error: unsupported PRG ROM size".to_string());
    }

    // Four-screen mirroring panics in the PPU
    if rom[6] & 0b1000 != 0 {
        return Err(
            "error: four-screen mirroring is not supported".to_string()
        );
    }

    // Every declared section must actually be present in the file
    let trainer = if rom[6] & 0b100 != 0 { 512 } else { 0 };
    let prg = rom[4] as usize * 16384;
    let chr = rom[5] as usize * 8192; // 0 means CHR RAM, contributes no data
    if rom.len() < 16 + trainer + prg + chr {
        return Err("error: file truncated".to_string());
    }

    Ok(0)
}

pub struct Cartridge {
    pub has_trainer: bool,
    pub has_sram: bool,
    pub has_chr_ram: bool,

    pub sram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,

    pub mirroring: Mirroring,
}

impl Cartridge {
    pub fn new(rom: &[u8]) -> Self {
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
            - 0         = 0b0: horizontal mirroring, 0b1: vertical mirroring
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
        let has_chr_ram = rom[5] == 0;
        let chr_rom_size = 0x2000 * usize::max(1, rom[5] as usize);
        let sram_size = 0x2000;
        let trainer_size = if rom[6] & 0b100 != 0 { 512 } else { 0 };
        let prg_start = 16 + trainer_size;
        let chr_start = prg_start + prg_rom_size;

        // The trainer, when present, loads at $7000 (offset 0x1000 in SRAM)
        let mut sram = vec![0u8; sram_size];
        sram[0x1000..0x1000 + trainer_size]
            .copy_from_slice(&rom[16..16 + trainer_size]);
        let prg_rom = rom[prg_start..prg_start + prg_rom_size].to_vec();
        let chr_rom = if has_chr_ram {
            vec![0u8; 0x2000]
        } else {
            rom[chr_start..chr_start + chr_rom_size].to_vec()
        };

        let mirroring = if bit_3(rom[6]) == 1 {
            Mirroring::ALTERNATIVE
        } else if bit_0(rom[6]) == 0 {
            Mirroring::HORIZONTAL
        } else {
            Mirroring::VERTICAL
        };

        Cartridge {
            has_trainer: trainer_size != 0,
            has_sram: bit_1(rom[6]) != 0,
            has_chr_ram,

            sram,
            prg_rom,
            chr_rom,

            mirroring,
        }
    }
}
