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
        // PPU runs at master/4
        if self.master_clock % 4 == 0 {
            let mut ppu_ref = self.ppu.borrow_mut();
            ppu_ref.tick();
            if ppu_ref.nmi_signal {
                ppu_ref.nmi_signal = false;
                drop(ppu_ref);
                self.cpu.borrow_mut().nmi_pending = true;
            }
        }

        // CPU runs at master/12
        if self.master_clock % 12 == 0 {
            // if self.cpu.borrow_mut().ins.is_none() {
            //     self.cpu.borrow_mut().trace();
            // }
            let mut cpu_ref = self.cpu.borrow_mut();
            cpu_ref.tick();
            if cpu_ref.store.oam_dma_data < 0x100 {
                self.ppu
                    .borrow_mut()
                    .reg_write(4, cpu_ref.store.oam_dma_data as u8);
                cpu_ref.store.oam_dma_data = 0x200;
            }
        }

        // // APU runs at master/24
        // if self.master_clock % 24 == 0 {
        //     self.apu.borrow_mut().tick();
        // }

        self.master_clock = self.master_clock.wrapping_add(1);
    }
}
