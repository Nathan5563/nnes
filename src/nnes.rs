mod cpu;
mod ppu;

use std::{cell::RefCell, rc::Rc};

use super::Cartridge;
use cpu::{bus::Bus, CPU};
use ppu::PPU;

pub struct NNES {
    pub master_clock: u64,
    pub cpu: Rc<RefCell<CPU>>,
    pub ppu: Rc<RefCell<PPU>>,
    // pub apu: Rc<RefCell<APU>>,
}

impl NNES {
    pub fn new(cartridge: Cartridge) -> Self {
        let ppu = Rc::new(RefCell::new(PPU::new(&cartridge)));
        let bus = Bus::new(ppu.clone(), &cartridge);
        let cpu = Rc::new(RefCell::new(CPU::new(bus)));
        // let apu = APU::new();

        // Give PPU NMI callback access to CPU NMI field
        let cpu_ref_for_nmi = cpu.clone();
        ppu.borrow_mut().on_nmi = Box::new(move || {
            cpu_ref_for_nmi.borrow_mut().nmi_pending = true;
        });
        // Give PPU OAM DMA callback access to CPU OAM DMA field
        let cpu_ref_for_oam_dma = cpu.clone();
        ppu.borrow_mut().on_oam_dma = Box::new(move || {
            // TODO: set CPU field to indicate OAMDMA
        });

        NNES {
            master_clock: 0,
            cpu,
            ppu,
            // apu,
        }
    }

    pub fn reset(&mut self) {
        self.ppu.borrow_mut().reset();
        self.cpu.borrow_mut().reset();
    }

    pub fn tick(&mut self) {
        // CPU runs at master/12
        if self.master_clock % 12 == 0 {
            // if self.cpu.borrow_mut().ins.is_none() {
            //     self.cpu.borrow_mut().trace();
            // }
            self.cpu.borrow_mut().tick();
        }

        // PPU runs at master/4
        if self.master_clock % 4 == 0 {
            self.ppu.borrow_mut().tick();
        }

        // // APU runs at master/24
        // if self.master_clock % 24 == 0 {
        //     self.apu.borrow_mut().tick();
        // }

        self.master_clock = self.master_clock.wrapping_add(1);
    }
}
