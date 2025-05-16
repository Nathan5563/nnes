mod cpu;
mod bus;

use cpu::CPU;
use bus::{Bus, Cartridge};

pub struct NNES {
    cpu: CPU,
    // ppu: PPU,
    // apu: APU,
    bus: Bus,
}

impl NNES {
    pub fn new(rom: Cartridge) -> Self {
        let bus = Bus::new(rom);
        // let ppu = PPU::new();
        // let apu = APU::new();
        let cpu = CPU::new();

        // figure out cpu, ppu, apu <=> bus address space

        NNES { cpu: cpu, /* ppu: ppu, apu: apu, */ bus: bus }
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
        // grab snapshot of current state for testing roms
    }
}