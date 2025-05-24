use super::{BusDevice, Cartridge};

// Internal RAM
pub struct RAM {
    ram: [u8; 0x0800],
}

impl BusDevice for RAM {
    fn contains(&self, addr: u16) -> bool {
        (0x0000..0x2000).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize & 0x07FF]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize & 0x07FF] = data;
    }

    fn peek(&self, addr: u16) -> u8 {
        self.ram[addr as usize & 0x07FF]
    }
}

// PPU IO Registers
pub struct PPU_Regs {
    ppu_regs: [u8; 0x0008],
}

impl BusDevice for PPU_Regs {
    fn contains(&self, addr: u16) -> bool {
        (0x2000..0x4000).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }

    fn peek(&self, addr: u16) -> u8 {
        self.ppu_regs[(addr as usize - 0x2000) & 0x7]
    }
}

// APU IO Registers
pub struct APU_Regs {
    apu_regs: [u8; 0x0020],
}

impl BusDevice for APU_Regs {
    fn contains(&self, addr: u16) -> bool {
        (0x4000..0x4020).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }

    fn peek(&self, addr: u16) -> u8 {
        unimplemented!()
    }
}

pub struct Expansion_ROM {
    expansion_rom: [u8; 0x1FE0],
}

impl BusDevice for Expansion_ROM {
    fn contains(&self, addr: u16) -> bool {
        (0x4020..0x6000).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }

    fn peek(&self, addr: u16) -> u8 {
        unimplemented!()
    }
}

// Save RAM
pub struct SRAM {
    sram: Vec<u8>,
}

impl BusDevice for SRAM {
    fn contains(&self, addr: u16) -> bool {
        (0x6000..0x8000).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }

    fn peek(&self, addr: u16) -> u8 {
        unimplemented!()
    }
}

// Program ROM
pub struct PRG_ROM {
    num_banks: u8,
    prg_rom: Vec<u8>,
}

impl BusDevice for PRG_ROM {
    fn contains(&self, addr: u16) -> bool {
        (0x8000..=0xFFFF).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        match self.num_banks {
            1 => {
                self.prg_rom[(addr as usize - 0x8000) & 0x3FFF]
            }
            2 => {
                self.prg_rom[addr as usize - 0x8000]
            }
            _ => unimplemented!()
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        unreachable!()
    }

    fn peek(&self, addr: u16) -> u8 {
        match self.num_banks {
            1 => {
                self.prg_rom[(addr as usize - 0x8000) & 0x3FFF]
            }
            2 => {
                self.prg_rom[addr as usize - 0x8000]
            }
            _ => unimplemented!()
        }
    }
}

pub fn memory_map(memory_handlers: &mut Vec<Box<dyn BusDevice>>, cartridge: Cartridge) {
    // attach RAM, PPU_Regs, APU_Regs, SRAM as needed, PRG_ROM, and other memory objects
    memory_handlers.push(Box::new(RAM { ram: [0; 0x0800] }));
    memory_handlers.push(Box::new(PPU_Regs {
        ppu_regs: [0; 0x0008],
    }));
    memory_handlers.push(Box::new(APU_Regs {
        apu_regs: [0; 0x0020],
    }));
    if cartridge.has_trainer || cartridge.has_sram {
        memory_handlers.push(Box::new(SRAM {
            sram: cartridge.sram,
        }));
    }
    let prg_rom = cartridge.prg_rom;
    let num_banks = (prg_rom.len() / 0x4000) as u8;
    memory_handlers.push(Box::new(PRG_ROM {
        prg_rom,
        num_banks,
    }));
}
