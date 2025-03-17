mod flags;
mod interrupts;
mod memory;
mod opcodes;
mod registers;
pub mod types;
pub mod utils;

// Make only the enums and utilities public to main.rs
pub use flags::Flag;
pub use interrupts::Interrupt;
pub use memory::{AddressingMode, STACK_OFFSET};
pub use opcodes::OpCode;
pub use registers::Register;
pub use types::*;
pub use utils::*;

use flags::*;
use interrupts::*;
use memory::*;
use opcodes::*;
use registers::*;

pub static PROGRAM_START_POINT: u16 = 0x8000;

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
            stack_pointer: 0xff,
            reg_accumulator: 0,
            reg_xindex: 0,
            reg_yindex: 0,
            flags: 0,
            memory: [0; 0xffff],
        }
    }

    pub fn load(&mut self, program: Vec<u8>) {
        let mut idx = 0;
        for data in program {
            self.memory_write_u8(PROGRAM_START_POINT + idx, data);
            idx += 1;
        }
        self.memory_write_u16(0xfffc, PROGRAM_START_POINT);
    }

    pub fn reset_state(&mut self) {
        self.reset_registers();
        self.reset_flags();
        self.set_program_counter(self.memory_read_u16(0xfffc));
        self.set_stack_pointer(0xff);
    }

    pub fn run(&mut self) {
        while !self.get_flag(Flag::Break) {
            let pc: u16 = self.get_program_counter();
            let code: u8 = self.memory_read_u8(pc);
            self.set_program_counter(pc + 1);
            let ins = opcodes_map
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized", code));
            let mode: AddressingMode = ins.get_addressing_mode();

            match code {
                0xaa => self.handle_tax(),
                0xa8 => self.handle_tay(),
                0xba => self.handle_tsx(),
                0x8a => self.handle_txa(),
                0x9a => self.handle_txs(),
                0x98 => self.handle_tya(),
                0x18 => self.handle_clc(),
                0xd8 => self.handle_cld(),
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => self.handle_lda(mode),
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => self.handle_ldx(mode),
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => self.handle_ldy(mode),
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => self.handle_sta(mode),
                0x86 | 0x96 | 0x8e => self.handle_stx(mode),
                0x84 | 0x94 | 0x8c => self.handle_sty(mode),
                0x48 => self.handle_pha(),
                0x08 => self.handle_php(),
                0x68 => self.handle_pla(),
                0x28 => self.handle_plp(),
                0xe8 => self.handle_inx(),
                0x00 => self.handle_brk(),
                _ => return,
            }
        }
    }

    pub fn play(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset_state();
        self.run();
    }

    pub fn play_test(&mut self, program: Vec<u8>) {
        self.load(program);
        self.set_program_counter(self.memory_read_u16(0xfffc));
        self.run();
    }
}
