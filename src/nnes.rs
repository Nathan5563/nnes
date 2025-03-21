pub mod cpu;
pub mod memory;
pub mod bus;
pub mod rom;

use bus::Bus;
use memory::Mem;

pub static RESET_VECTOR: u16 = 0xfffc;
pub static IRQ_VECTOR: u16 = 0xfffe;
pub static PROGRAM_START_PC: u16 = 0x8000;
pub static PROGRAM_START_STATUS: u8 = 0b00100100;
pub static SNAKE_6502_PC: u16 = 0x0600; // new for game code

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

    // New method for game code expecting start at 0x0600
    pub fn load_snake(&mut self, program: Vec<u8>) {
        let mut idx = 0;
        for data in program {
            self.memory_write_u8(SNAKE_6502_PC + idx, data);
            idx += 1;
        }
        self.memory_write_u16(RESET_VECTOR, SNAKE_6502_PC);
    }

    pub fn reset_state_snake(&mut self) {
        self.reset_registers();
        self.reset_flags();
        self.set_program_counter(PROGRAM_START_PC + SNAKE_6502_PC);
        self.set_stack_pointer(0xfd);
    }

    pub fn reset_state(&mut self) {
        self.reset_registers();
        self.reset_flags();
        self.set_program_counter(self.memory_read_u16(RESET_VECTOR));
        self.set_stack_pointer(0xfd);
    }

    pub fn run(&mut self) {
        self.run_callback(|_| {});
    }

    pub fn run_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut NNES),
    {
        let mut exit: bool = false;
        while !exit {
            callback(self);
            self.step(&mut exit);
        }
    }

    pub fn play(&mut self) {
        self.reset_state();
        self.run();
    }
}
