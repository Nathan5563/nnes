mod devices;

use std::{rc::Rc, cell::RefCell};
use super::super::{Cartridge, PPU};
use devices::memory_map;

pub trait BusDevice {
    fn contains(&self, addr: u16) -> bool;
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
    fn peek(&self, addr: u16) -> u8;
}

pub struct Bus {
    ppu: Rc<RefCell<PPU>>,
    memory_handlers: Vec<Box<dyn BusDevice>>,
    open_bus: u8,
}

impl Bus {
    pub fn new(ppu: Rc<RefCell<PPU>>, cartridge: &Cartridge) -> Self {
        let mut memory_handlers: Vec<Box<dyn BusDevice>> = Vec::new();
        memory_map(ppu.clone(), &mut memory_handlers, cartridge);

        Bus {
            ppu,
            memory_handlers,
            open_bus: 0,
        }
    }

    pub fn mem_read(&mut self, addr: u16) -> u8 {
        for handler in &mut self.memory_handlers {
            if handler.contains(addr) {
                let v = handler.mem_read(addr);
                self.open_bus = v;
                return v;
            }
        }
        self.open_bus
        // handle dummys somehow (new func or simple param to indicate)
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        for handler in &mut self.memory_handlers {
            if handler.contains(addr) {
                handler.mem_write(addr, data);
                self.open_bus = data;
                return;
            }
        }
    }

    pub fn peek(&self, addr: u16) -> u8 {
        for handler in &self.memory_handlers {
            if handler.contains(addr) {
                return handler.peek(addr);
            }
        }
        self.open_bus
        // handle dummys somehow (new func or simple param to indicate)
    }
}
