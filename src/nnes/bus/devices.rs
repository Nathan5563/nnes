use super::BusDevice;

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

// APU IO Registers
pub struct APU_Regs {
    apu_regs: [u8; 0x0018],
}

// for everything below this point, figure out cartridge struct first

// Save RAM
pub struct SRAM {
    s_ram: [u8; 0x2000],
}

// Program ROM
pub struct PRG_ROM {
    prg_rom: [u8; 0x8000],
}
