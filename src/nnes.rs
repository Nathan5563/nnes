mod cpu;

use super::Cartridge;
use cpu::{bus::Bus, CPU};

pub struct NNES {
    master_clock: u64,
    cpu: CPU,
    // ppu: PPU,
    // apu: APU,
}

impl NNES {
    pub fn new(cartridge: Cartridge) -> Self {
        let bus = Bus::new(cartridge);
        let cpu = CPU::new(bus);
        // let ppu = PPU::new();
        // let apu = APU::new();

        NNES {
            master_clock: 0,
            cpu,
            // ppu,
            // apu,
        }
    }

    pub fn tick(&mut self) {  
        // // PPU runs at master/4
        // if self.master_clock % 4 == 0 {
        //     self.ppu.tick(&mut self.bus);
        // }

        // CPU runs at master/12
        if self.master_clock % 12 == 0 {
            self.cpu.tick();
        }

        // // APU runs at master/24
        // if self.master_clock % 24 == 0 {
        //     self.apu.tick(&mut self.bus);
        // }

        self.master_clock = self.master_clock.wrapping_add(1);
    }
}
