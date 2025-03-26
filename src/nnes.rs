pub mod cpu;
pub mod memory;
pub mod bus;
pub mod rom;
pub mod ppu;

use bus::Bus;
use memory::Mem;

pub static RESET_VECTOR: u16 = 0xfffc;
pub static IRQ_VECTOR: u16 = 0xfffe;
pub static PROGRAM_START_PC: u16 = 0x8000;
pub static PROGRAM_START_STATUS: u8 = 0b00100100;

pub struct NNES {
    program_counter: u16,
    stack_pointer: u8,
    reg_accumulator: u8,
    reg_xindex: u8,
    reg_yindex: u8,
    flags: u8,
    bus: Bus,
}

impl NNES {
    pub fn new(bus: Bus) -> Self {
        NNES {
            program_counter: PROGRAM_START_PC,
            stack_pointer: 0xfd,
            reg_accumulator: 0,
            reg_xindex: 0,
            reg_yindex: 0,
            flags: PROGRAM_START_STATUS,
            bus: bus,
        }
    }

    pub fn reset_state(&mut self) {
        self.reset_registers();
        self.reset_flags();
        self.set_program_counter(self.memory_read_u16(RESET_VECTOR));
        self.set_stack_pointer(0xfd);
    }

    pub fn run_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut NNES),
    {
        let mut exit: bool = false;
        let mut cpu_cycle: u8 = 0;
        let mut ppu_cycle: u8 = 0;
        while !exit {
            callback(self);
            self.cpu_tick(&mut cpu_cycle, &mut exit);
            self.ppu_tick(&mut ppu_cycle);
            self.ppu_tick(&mut ppu_cycle);
            self.ppu_tick(&mut ppu_cycle);
        }
    }
}
