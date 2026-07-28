use super::{BusDevice, Cartridge, PPU};
use crate::{controller::{Controllers, Joypad}, utils::bit_7};
use std::{cell::RefCell, rc::Rc};

pub struct RAM {
    ram: [u8; 0x0802],
    oam_dma_running: bool,
}

impl BusDevice for RAM {
    fn contains(&self, addr: u16) -> bool {
        (0x0000..0x2000).contains(&addr) || addr == 0x4014
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        if addr == 0x4014 {
            panic!("OAM DMA read");
        } else {
            self.ram[addr as usize & 0x07FF]
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        if addr == 0x4014 {
            self.ram[0x0800] = 42;
            self.ram[0x0801] = data;
        } else {
            self.ram[addr as usize & 0x07FF] = data;
        }
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr == 0x4014 {
            self.ram[0x0801]
        } else {
            self.ram[addr as usize & 0x07FF]
        }
    }

    fn oam_dma_pending(&self) -> bool {
        if self.ram[0x0800] == 42 {
            true
        } else {
            false
        }
    }

    fn oam_dma_start(&mut self) {
        self.ram[0x0800] = 0;
        self.ram[0x0801] = 0;
        self.oam_dma_running = true;
    }

    fn oam_dma_running(&self) -> bool {
        self.oam_dma_running
    }

    fn oam_dma_finish(&mut self) {
        self.oam_dma_running = false;
    }
}

pub struct PpuRegs {
    ppu: Rc<RefCell<PPU>>,
}

impl BusDevice for PpuRegs {
    fn contains(&self, addr: u16) -> bool {
        (0x2000..0x4000).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        let reg = (addr % 8) as u8;
        self.ppu.borrow_mut().reg_read(reg)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        let reg = (addr % 8) as u8;
        self.ppu.borrow_mut().reg_write(reg, data);
    }

    fn peek(&self, addr: u16) -> u8 {
        // TODO: reg_peek()? return last written byte? what is important
        0
    }

    fn ppu_debug_cycle(&self) -> Option<u16> {
        Some(self.ppu.borrow_mut().cycle)
    }

    fn ppu_debug_scanline(&self) -> Option<u16> {
        Some(self.ppu.borrow_mut().scanline)
    }
}

pub struct ApuRegs {
    // apu: Rc<RefCell<APU>>,
    apu_regs: [u8; 0x0020],
}

impl BusDevice for ApuRegs {
    fn contains(&self, addr: u16) -> bool {
        (0x4000..=0x4015).contains(&addr) || (0x4018..=0x4019).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        0
    }

    fn mem_write(&mut self, addr: u16, data: u8) {}

    fn peek(&self, addr: u16) -> u8 {
        0
    }
}

impl BusDevice for Controllers {
    fn contains(&self, addr: u16) -> bool {
        (0x4016..=0x4017).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        let pad = if addr == 0x4016 {
            &mut self.pad1
        } else {
            &mut self.pad2
        };
        let data = bit_7(pad.state);
        pad.state <<= 1;
        data
    }

    fn mem_write(&mut self, addr: u16, _data: u8) {
        // $4017 writes go to the APU frame counter, not the controllers
        if addr == 0x4016 {
            self.pad1.state = self.pad1.active;
            self.pad2.state = self.pad2.active;
        }
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr == 0x4016 {
            self.pad1.state
        } else {
            self.pad2.state
        }
    }

    fn get_controllers_ref(&mut self) -> Option<&mut Controllers> {
        Some(self)
    }
}

pub struct ExpansionRom {
    _expansion_rom: [u8; 0x1FE0],
}

impl BusDevice for ExpansionRom {
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

pub struct Sram {
    sram: Vec<u8>,
}

impl BusDevice for Sram {
    fn contains(&self, addr: u16) -> bool {
        (0x6000..0x8000).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        self.sram[(addr - 0x6000) as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.sram[(addr - 0x6000) as usize] = data;
    }

    fn peek(&self, addr: u16) -> u8 {
        self.sram[(addr - 0x6000) as usize]
    }
}

pub struct PrgRom {
    num_banks: u8,
    prg_rom: Vec<u8>,
}

impl BusDevice for PrgRom {
    fn contains(&self, addr: u16) -> bool {
        (0x8000..=0xFFFF).contains(&addr)
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        match self.num_banks {
            1 => self.prg_rom[(addr as usize - 0x8000) & 0x3FFF],
            2 => self.prg_rom[addr as usize - 0x8000],
            _ => unimplemented!(),
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {}

    fn peek(&self, addr: u16) -> u8 {
        match self.num_banks {
            1 => self.prg_rom[(addr as usize - 0x8000) & 0x3FFF],
            2 => self.prg_rom[addr as usize - 0x8000],
            _ => unimplemented!(),
        }
    }
}

/**
 * Map various memory objects into the CPU's address space based on the
 * inserted cartridge.
 * @param   ppu                 pointer to a shared PPU object
 * @param   cartridge           reference to inserted cartridge
 * @param   memory_handlers     reference to vector of pointers to available
 *                              memory objects
 */
pub fn memory_map(
    ppu: Rc<RefCell<PPU>>,
    cartridge: &Cartridge,
    memory_handlers: &mut Vec<Box<dyn BusDevice>>,
) {
    memory_handlers.push(Box::new(RAM {
        ram: [0; 0x0802],
        oam_dma_running: false,
    }));
    memory_handlers.push(Box::new(PpuRegs { ppu }));
    memory_handlers.push(Box::new(ApuRegs {
        apu_regs: [0; 0x0020],
    }));
    memory_handlers.push(Box::new(Controllers {
        pad1: Joypad { active: 0, state: 0 },
        pad2: Joypad { active: 0, state: 0 },
    }));
    if cartridge.has_trainer || cartridge.has_sram {
        memory_handlers.push(Box::new(Sram {
            sram: cartridge.sram.clone(),
        }));
    }
    let prg_rom = cartridge.prg_rom.clone();
    let num_banks = (prg_rom.len() / 0x4000) as u8;
    memory_handlers.push(Box::new(PrgRom { prg_rom, num_banks }));
}
