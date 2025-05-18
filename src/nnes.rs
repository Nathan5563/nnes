mod cpu;

use super::Cartridge;
use cpu::{bus::Bus, CPU};

pub struct NNES {
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
            cpu,
            // ppu,
            // apu,
        }
    }

    pub fn step(&mut self) {
        // figure out master clock, timings, etc

        self.cpu.tick();
        // self.ppu.tick();
        // self.ppu.tick();
        // self.ppu.tick();
        // self.apu.tick();
    }

    pub fn trace(&mut self) {
        // for test roms
    }
}
