mod registers;
mod flags;
mod interrupts;
mod memory;
mod opcodes;
pub mod utils;

use flags::*;
use registers::*;
use interrupts::*;
use memory::*;
use opcodes::*;

pub enum Register
{
    ACCUMULATOR,
    XIndex,
    YIndex,
}

pub enum Flag
{
    Carry,
    Zero,
    InterruptDisable,
    DecimalMode,
    Break,
    Overflow,
    Negative,
}

pub enum Interrupt
{
    Reset,
    NMI,
    IRQ,
}

pub struct NNES
{
    program_counter: u16,
    stack_pointer: u8,
    reg_accumulator: u8,
    reg_xindex: u8,
    reg_yindex: u8,
    flags: u8,
    memory: [u8; 0xffff],
}

impl NNES
{
    pub fn new() -> Self
    {
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

    pub fn interpret(&mut self)
    {
        while !self.get_flag(Flag::Break)
        {
            let pc: u16 = self.get_program_counter();
            let opcode: u8 = self.memory_read(pc);
            self.set_program_counter(pc + 1);

            match opcode
            {
                0x00 => self.handle_brk(),
                0xa9 => self.handle_lda(),
                0xaa => self.handle_tax(),
                0xe8 => self.handle_inx(),
                _ => ()
            }
        }
    }
}
