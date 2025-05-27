mod cpu;
mod ppu;

use super::Cartridge;
use cpu::{bus::Bus, CPU};
use ppu::PPU;

pub struct NNES {
    pub master_clock: u64,
    pub cpu: CPU,
    pub ppu: PPU,
    // pub apu: APU,
}

impl NNES {
    pub fn new(cartridge: Cartridge) -> Self {
        let bus = Bus::new(&cartridge);
        let mut cpu = CPU::new(bus);
        let ppu = PPU::new(
            &cartridge,
            Box::new(move || {
                cpu.nmi_pending = true;
            }),
        );
        // let apu = APU::new();

        NNES {
            master_clock: 0,
            cpu,
            ppu,
            // apu,
        }
    }

    pub fn tick(&mut self) {
        // PPU runs at master/4
        if self.master_clock % 4 == 0 {
            self.ppu.tick();
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
