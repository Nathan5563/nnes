mod devices;

use super::super::Cartridge;

pub trait BusDevice {
    fn contains(&self, addr: u16) -> bool;
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
    fn peek(&self, addr: u16) -> u8;
}

pub struct Bus {
    memory_handlers: Vec<Box<dyn BusDevice>>,
    open_bus: u8,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        let mut memory_handlers = Vec::new();
        devices::memory_map(&mut memory_handlers, cartridge);

        Bus {
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
        // self.open_bus
        // unimplemented!(); // get rid of this and uncomment the above once finished
        // handle dummys somehow (new func or simple param to indicate)
        0
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        for handler in &mut self.memory_handlers {
            if handler.contains(addr) {
                handler.mem_write(addr, data);
                self.open_bus = data;
                return;
            }
        }
        panic!("Unmapped write at address ${addr}"); // get rid of this once finished
    }

    pub fn peek(&self, addr: u16) -> u8 {
        for handler in &self.memory_handlers {
            if handler.contains(addr) {
                return handler.peek(addr);
            }
        }
        // self.open_bus
        // unimplemented!(); // get rid of this and uncomment the above once finished
        // handle dummys somehow (new func or simple param to indicate)
        0
    }
}
