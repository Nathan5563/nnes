mod devices;

use super::super::{Cartridge, APU, PPU};
use crate::controller::Controllers;
use devices::memory_map;
use std::{cell::RefCell, rc::Rc};

pub trait BusDevice {
    fn contains(&self, addr: u16) -> bool;
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
    fn peek(&self, addr: u16) -> u8;
    fn oam_dma_pending(&self) -> bool {
        false
    }
    fn oam_dma_start(&mut self) {}
    fn oam_dma_running(&self) -> bool {
        false
    }
    fn oam_dma_finish(&mut self) {}
    #[allow(dead_code)]
    fn ppu_debug_cycle(&self) -> Option<u16> {
        None
    }
    #[allow(dead_code)]
    fn ppu_debug_scanline(&self) -> Option<u16> {
        None
    }
    fn get_controllers_ref(&mut self) -> Option<&mut Controllers> {
        None
    }
}

pub struct Bus {
    pub memory_handlers: Vec<Box<dyn BusDevice>>,
    open_bus: u8,
}

impl Bus {
    pub fn new(
        ppu: Rc<RefCell<PPU>>,
        apu: Rc<RefCell<APU>>,
        cartridge: &Cartridge,
    ) -> Self {
        let mut memory_handlers: Vec<Box<dyn BusDevice>> = Vec::new();
        memory_map(ppu, apu, cartridge, &mut memory_handlers);
        Bus {
            memory_handlers,
            open_bus: 0,
        }
    }

    pub fn mem_read(&mut self, addr: u16) -> u8 {
        // $4000-$4014 are write-only; reads return open bus
        if (0x4000..=0x4014).contains(&addr) {
            return self.open_bus;
        }
        for handler in &mut self.memory_handlers {
            if handler.contains(addr) {
                let mut v = handler.mem_read(addr);
                if addr == 0x4015 {
                    // $4015 bit 5 is open bus
                    v |= self.open_bus & 0x20;
                }
                self.open_bus = v;
                return v;
            }
        }
        self.open_bus
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.open_bus = data;
        // $4017 reads are controller 2, but writes go to the APU frame
        // counter, which shares a handler with the other APU registers
        let route = if addr == 0x4017 { 0x4015 } else { addr };
        for handler in &mut self.memory_handlers {
            if handler.contains(route) {
                handler.mem_write(addr, data);
                return;
            }
        }
    }

    pub fn peek(&self, addr: u16) -> u8 {
        for handler in &self.memory_handlers {
            if handler.contains(addr) {
                let mut v = handler.peek(addr);
                if addr == 0x4015 {
                    // Match mem_read: $4015 bit 5 is open bus
                    v |= self.open_bus & 0x20;
                }
                return v;
            }
        }
        self.open_bus
    }

    pub fn get_controllers_ref(&mut self) -> Option<&mut Controllers> {
        for handler in &mut self.memory_handlers {
            if handler.contains(0x4016) {
                return handler.get_controllers_ref();
            }
        }
        None
    }

    pub fn oam_dma_pending(&self) -> bool {
        for handler in &self.memory_handlers {
            if handler.contains(0x4014) {
                return handler.oam_dma_pending();
            }
        }
        false
    }

    pub fn oam_dma_start(&mut self) {
        for handler in &mut self.memory_handlers {
            if handler.contains(0x4014) {
                return handler.oam_dma_start();
            }
        }
    }

    pub fn oam_dma_running(&self) -> bool {
        for handler in &self.memory_handlers {
            if handler.contains(0x4014) {
                return handler.oam_dma_running();
            }
        }
        false
    }

    pub fn oam_dma_finish(&mut self) {
        for handler in &mut self.memory_handlers {
            if handler.contains(0x4014) {
                handler.oam_dma_finish();
                return;
            }
        }
    }
}
