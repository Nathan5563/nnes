mod apu;
mod cpu;
mod ppu;

use std::{cell::RefCell, rc::Rc};

use super::Cartridge;
use apu::APU;
use cpu::{bus::Bus, CPU};
use ppu::PPU;

pub struct NNES {
    pub master_clock: u64,
    pub cpu: Rc<RefCell<CPU>>,
    pub ppu: Rc<RefCell<PPU>>,
    pub apu: Rc<RefCell<APU>>,
}

impl NNES {
    pub fn new(cartridge: Cartridge) -> Self {
        let ppu = Rc::new(RefCell::new(PPU::new(&cartridge)));
        let apu = Rc::new(RefCell::new(APU::new()));
        let bus = Bus::new(ppu.clone(), apu.clone(), &cartridge);
        let cpu = Rc::new(RefCell::new(CPU::new(bus)));

        NNES {
            master_clock: 0,
            cpu,
            ppu,
            apu,
        }
    }

    pub fn reset(&mut self) {
        self.ppu.borrow_mut().reset();
        self.apu.borrow_mut().reset();
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

        // CPU runs at master/12; the APU shares the CPU clock (its triangle
        // timer and frame counter run at CPU rate) and ticks first, so that
        // register writes made by the CPU settle at the end of the cycle
        if self.master_clock % 12 == 0 {
            // if self.cpu.borrow_mut().ins.is_none() {
            //     self.cpu.borrow_mut().trace();
            // }
            let mut cpu_ref = self.cpu.borrow_mut();
            {
                let mut apu_ref = self.apu.borrow_mut();
                apu_ref.tick();
                // The APU can't reach the CPU bus, so DMC sample fetches
                // are serviced here, mirroring the OAM DMA forwarding
                if let Some(addr) = apu_ref.take_dmc_request() {
                    let data = cpu_ref.bus.mem_read(addr);
                    apu_ref.dmc_load(data);
                    cpu_ref.begin_dmc_stall();
                }
                cpu_ref.set_irq_level(apu_ref.irq_line());
            }
            cpu_ref.tick();
            if cpu_ref.store.oam_dma_data < 0x100 {
                self.ppu
                    .borrow_mut()
                    .reg_write(4, cpu_ref.store.oam_dma_data as u8);
                cpu_ref.store.oam_dma_data = 0x200;
            }
            let mut ppu_ref = self.ppu.borrow_mut();
            if ppu_ref.nmi_suppress {
                ppu_ref.nmi_suppress = false;
                cpu_ref.suppress_nmi();
            }
        }

        self.master_clock = self.master_clock.wrapping_add(1);
    }
}
