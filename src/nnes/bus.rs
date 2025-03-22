use crate::nnes::memory::Mem;
use crate::nnes::rom::{Rom, PRG_ROM_PAGE_SIZE};

pub struct Bus {
    ram: [u8; 0x0800],
    pub rom: Rom,
}

static PRG_ROM_MEM_START: u16 = 0x8000;

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            ram: [0; 0x0800],
            rom: rom,
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= PRG_ROM_MEM_START;
        if (self.rom.prg_rom.len() == PRG_ROM_PAGE_SIZE as usize) && (addr >= PRG_ROM_PAGE_SIZE as u16)
        {
            addr %= PRG_ROM_PAGE_SIZE as u16;
        }
        self.rom.prg_rom[addr as usize]
    }
}

impl Mem for Bus {
    fn memory_read_u8(&self, mut addr: u16) -> u8 {
        let mirrored: u16;
        match addr {
            0x0000..0x2000 => {
                mirrored = addr & 0b111_1111_1111;
                self.ram[mirrored as usize]
            }
            0x2000..0x4000 => {
                mirrored = addr & 0b100000_00000111;
                todo!("PPU not yet implemented!");
                self.ram[mirrored as usize]
            }
            0x8000..=0xffff => self.read_prg_rom(addr),
            _ => todo!("Memory reads not yet implemented at {addr}!"),
        }
    }

    fn memory_write_u8(&mut self, mut addr: u16, data: u8) {
        let mirrored: u16;
        match addr {
            0x0000..0x2000 => {
                mirrored = addr & 0b111_1111_1111;
                self.ram[mirrored as usize] = data;
            }
            0x2000..0x4000 => {
                mirrored = addr & 0b100000_00000111;
                todo!("PPU not yet implemented!");
                self.ram[mirrored as usize] = data;
            }
            0x8000..=0xffff => panic!("Attempt to write to PRG ROM"),
            _ => todo!("Memory writes not yet implemented at {addr}!"),
        }
    }
}
