use super::{BusDevice, Cartridge};

// Internal RAM
pub struct RAM {
    ram: [u8; 0x0800],
}

impl BusDevice for RAM {
    fn contains(&self, addr: u16) -> bool {
        if addr < 0x2000 {
            true
        } else {
            false
        }
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        self.ram[(addr & 0x07FF) as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.ram[(addr & 0x07FF) as usize] = data;
    }
}

// PPU IO Registers
pub struct PPU_Regs {
    ppu_regs: [u8; 0x0008],
}

impl BusDevice for PPU_Regs {
    fn contains(&self, addr: u16) -> bool {
        unimplemented!()
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }
}

// APU IO Registers
pub struct APU_Regs {
    apu_regs: [u8; 0x0018],
}

impl BusDevice for APU_Regs {
    fn contains(&self, addr: u16) -> bool {
        unimplemented!()
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }
}

// Save RAM
pub struct SRAM {
    sram: Vec<u8>,
}

impl BusDevice for SRAM {
    fn contains(&self, addr: u16) -> bool {
        unimplemented!()
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }
}

// Program ROM
pub struct PRG_ROM {
    prg_rom: Vec<u8>,
}

impl BusDevice for PRG_ROM {
    fn contains(&self, addr: u16) -> bool {
        unimplemented!()
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }
}

pub fn memory_map(memory_handlers: &mut Vec<Box<dyn BusDevice>>, cartridge: Cartridge) {
    // attach RAM, PPU_Regs, APU_Regs, SRAM as needed, PRG_ROM, and other memory objects
    memory_handlers.push(Box::new( RAM { ram: [0; 0x0800] } ));
    memory_handlers.push(Box::new( PPU_Regs { ppu_regs: [0; 0x0008] } ));
    memory_handlers.push(Box::new( APU_Regs { apu_regs: [0; 0x0018] } ));
    if cartridge.has_trainer || cartridge.has_sram {
        memory_handlers.push(Box::new( SRAM { sram: cartridge.sram } ));
    }
    memory_handlers.push(Box::new( PRG_ROM { prg_rom: cartridge.prg_rom } ));
}
