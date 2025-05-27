mod cpu;
mod ppu;

use std::{rc::Rc, cell::RefCell};

use super::Cartridge;
use cpu::{bus::Bus, CPU};
use ppu::PPU;

pub struct NNES {
    pub master_clock: u64,
    pub cpu: CPU,
    pub ppu: Rc<RefCell<PPU>>,
    // pub apu: APU,
}

impl NNES {
    pub fn new(cartridge: Cartridge) -> Self {
        let ppu = Rc::new(RefCell::new(PPU::new(&cartridge)));
        let bus = Bus::new(ppu.clone(), &cartridge);
        let cpu = CPU::new(bus);
        // let apu = APU::new();

        NNES {
            master_clock: 0,
            cpu,
            ppu,
            // apu,
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn tick(&mut self) {
        // PPU runs at master/4
        if self.master_clock % 4 == 0 {
            self.ppu.borrow_mut().tick();
        }

        // CPU runs at master/12
        if self.master_clock % 12 == 0 {
            self.cpu.tick();
        }

        // // APU runs at master/24
        // if self.master_clock % 24 == 0 {
        //     self.apu.tick();
        // }

        self.master_clock = self.master_clock.wrapping_add(1);
    }
}
