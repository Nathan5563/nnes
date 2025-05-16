mod bus;
mod cpu;

use bus::Bus;
use cpu::CPU;

pub struct NNES {
    cpu: CPU,
    // ppu: PPU,
    // apu: APU,
    // mapper? cartridge? something?
    bus: Bus,
}

impl NNES {
    pub fn new(/* cartridge: Cartridge */) -> Self {
        let cpu = CPU::new();
        // let ppu = PPU::new();
        // let apu = APU::new();

        let bus = Bus::new();
        // bus.attach(...)

        NNES {
            cpu: cpu,
            // ppu: ppu,
            // apu: apu,
            bus: bus,
        }
    }

    pub fn step(&mut self) {
        // figure out master clock, timings, etc

        self.cpu.tick(&mut self.bus);
        // self.ppu.tick();
        // self.ppu.tick();
        // self.ppu.tick();
        // self.apu.tick();
    }

    pub fn trace(&mut self) {
        // for test roms
    }
}
