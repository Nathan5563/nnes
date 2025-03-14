// Make only the enums and utilities public to main.rs
pub use registers::Register;
pub use flags::Flag;
pub use interrupts::Interrupt;
// pub use opcodes::OpCode;
pub mod utils;

mod registers;
mod flags;
mod interrupts;
mod memory;
mod opcodes;

use registers::*;
use flags::*;
use interrupts::*;
use memory::*;
use opcodes::*;

pub struct NNES {
    program_counter: u16,
    stack_pointer: u8,
    reg_accumulator: u8,
    reg_xindex: u8,
    reg_yindex: u8,
    flags: u8,
    memory: [u8; 0xffff],
}

impl NNES {
    pub fn new() -> Self {
        NNES {
            program_counter: 0,
            stack_pointer: 0,
            reg_accumulator: 0,
            reg_xindex: 0,
            reg_yindex: 0,
            flags: 0,
            memory: [0; 0xffff],
        }
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.reset_memory();
        let mut idx = 0;
        for data in program {
            self.memory_write(0x8000 + idx, data);
            idx += 1;
        }
        self.set_program_counter(0x8000);
    }

    pub fn reset(&mut self) {
        self.set_program_counter(self.memory_read(0xfffc) as u16);
        self.reset_registers();
        self.reset_flags();
    }

    pub fn run(&mut self) {
        while !self.get_flag(Flag::Break) {
            let pc: u16 = self.get_program_counter();
            let code: u8 = self.memory_read(pc);
            let ins = opcodes_map
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized", code));
            
            match code {
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    self.handle_lda();
                }
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.handle_sta();
                }
                0xAA => {
                    self.handle_tax();
                }
                0xe8 => {
                    self.handle_inx();
                }
                0x00 => {
                    self.handle_brk();
                }
                _ => {
                    return;
                }
            }            
        }
    }

    pub fn play(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn play_test(&mut self, program: Vec<u8>) {
        self.load(program);
        self.run();
    }
}
