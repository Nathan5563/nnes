use crate::types::{BIT_0, BIT_2, BIT_3, UPPER_NIBBLE};

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

static NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
pub static HEADER_SIZE: u8 = 0x10;
pub static TRAINER_SIZE: u16 = 0x200;
pub static PRG_ROM_PAGE_SIZE: u16 = 0x4000;
pub static CHR_ROM_PAGE_SIZE: u16 = 0x2000;

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        if &raw[0..4] != NES_TAG {
            return Err("File not in iNES format".to_string());
        }
        let version: u8 = (raw[7] >> 2) & 0b11;
        if version != 0 {
            return Err("NES2.0 is not supported yet".to_string());
        }

        let prg_rom_size: usize = raw[4] as usize * PRG_ROM_PAGE_SIZE as usize;
        let chr_rom_size: usize = raw[5] as usize * CHR_ROM_PAGE_SIZE as usize;
        let mapper: u8 = (raw[7] & UPPER_NIBBLE) | ((raw[6] & UPPER_NIBBLE) >> 4);
        let four_screen: bool = raw[6] & BIT_3 != 0;
        let vertical_mirroring: bool = raw[6] & BIT_0 != 0;
        let screen_mirroring: Mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let trainer: bool = raw[6] & BIT_2 != 0;
        // Doesn't do anything with the trainer; if it exists, skip it
        let prg_rom_start: usize =
            HEADER_SIZE as usize + if trainer { TRAINER_SIZE as usize } else { 0 };
        let chr_rom_start: usize = prg_rom_start + prg_rom_size;

        Ok({
            let rom: Rom = Rom {
                prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
                chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
                mapper: mapper,
                screen_mirroring: screen_mirroring,
            };
            println!();
            rom
        })
    }

    pub fn get_prg_rom_length(&self) -> usize {
        self.prg_rom.len()
    }

    pub fn read_prg_rom(&self, addr: u16) -> u8 {
        self.prg_rom[addr as usize]
    }
}
