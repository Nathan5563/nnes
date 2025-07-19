mod devices;

use super::super::{Cartridge, PPU};
use crate::controller::Joypad;
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
    fn ppu_debug_cycle(&self) -> Option<u16> {
        None
    }
    fn ppu_debug_scanline(&self) -> Option<u16> {
        None
    }
    fn get_joypad_ref(&mut self) -> Option<&mut Joypad> {
        None
    }
}

pub struct Bus {
    pub memory_handlers: Vec<Box<dyn BusDevice>>,
    open_bus: u8,
}

impl Bus {
    pub fn new(ppu: Rc<RefCell<PPU>>, cartridge: &Cartridge) -> Self {
        let mut memory_handlers: Vec<Box<dyn BusDevice>> = Vec::new();
        memory_map(ppu, cartridge, &mut memory_handlers);
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
        self.open_bus
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
    }

    pub fn get_joypad_ref(&mut self) -> Option<&mut Joypad> {
        for handler in &mut self.memory_handlers {
            if handler.contains(0x4016) {
                return handler.get_joypad_ref();
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
