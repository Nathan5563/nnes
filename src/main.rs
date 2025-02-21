#![allow(dead_code)]
#![allow(unused_variables)]

static CARRY_BIT: u8 = 0b00000001;
static NEG_CARRY_BIT: u8 = 0b11111110;
static ZERO_BIT: u8 = 0b00000010;
static NEG_ZERO_BIT: u8 = 0b11111101;
static INTERRUPTDISABLE_BIT: u8 = 0b00000100;
static NEG_INTERRUPTDISABLE_BIT: u8 = 0b11111011;
static DECIMALMODE_BIT: u8 = 0b00001000;
static NEG_DECIMALMODE_BIT: u8 = 0b11110111;
static BREAK_BIT: u8 = 0b00010000;
static NEG_BREAK_BIT: u8 = 0b11101111;
static OVERFLOW_BIT: u8 = 0b01000000;
static NEG_OVERFLOW_BIT: u8 = 0b10111111;
static NEGATIVE_BIT: u8 = 0b10000000;
static NEG_NEGATIVE_BIT: u8 = 0b01111111;

struct CPU
{
    program_counter: u16,
    stack_pointer: u8,
    reg_accumulator: u8,
    reg_xindex: u8,
    reg_yindex: u8,
    flags: u8,
}

enum Register
{
    ACCUMULATOR,
    XIndex,
    YIndex,
}

enum Flag
{
    Carry,
    Zero,
    InterruptDisable,
    DecimalMode,
    Break,
    Overflow,
    Negative,
}

enum Interrupt
{
    Reset,
    NMI,
    IRQ,
}

impl CPU
{
    pub fn new() -> Self
    {
        CPU {
            program_counter: 0,
            stack_pointer: 0,
            reg_accumulator: 0,
            reg_xindex: 0,
            reg_yindex: 0,
            flags: 0,
        }
    }

    pub fn get_program_counter(&self) -> u16
    {
        self.program_counter
    }

    pub fn set_program_counter(&mut self, value: u16)
    {
        self.program_counter = value;
    }

    pub fn get_stack_pointer(&self) -> u8
    {
        self.stack_pointer
    }

    pub fn set_stack_pointer(&mut self, value: u8)
    {
        self.stack_pointer = value;
    }

    pub fn get_register(&self, register: Register) -> u8
    {
        match register
        {
            Register::ACCUMULATOR => { self.reg_accumulator }
            Register::XIndex => { self.reg_xindex }
            Register::YIndex => { self.reg_yindex }
        }
    }

    pub fn set_register(&mut self, register: Register, value: u8)
    {
        match register
        {
            Register::ACCUMULATOR => { self.reg_accumulator = value; }
            Register::XIndex => { self.reg_xindex = value; }
            Register::YIndex => { self.reg_yindex = value; }
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool
    {
        match flag
        {
            Flag::Carry => { (self.flags & CARRY_BIT) != 0 }
            Flag::Zero => { (self.flags & ZERO_BIT) != 0 }
            Flag::InterruptDisable => { (self.flags & INTERRUPTDISABLE_BIT) != 0 }
            Flag::DecimalMode => { (self.flags & DECIMALMODE_BIT) != 0 }
            Flag::Break => { (self.flags & BREAK_BIT) != 0 }
            Flag::Overflow => { (self.flags & OVERFLOW_BIT) != 0 }
            Flag::Negative => { (self.flags & NEGATIVE_BIT) != 0 }
        }
    }

    pub fn set_flag(&mut self, flag: Flag, status: bool)
    {
        match flag
        {
            Flag::Carry => { 
                if status { self.flags |= CARRY_BIT; }
                else { self.flags &= NEG_CARRY_BIT; }
            }
            Flag::Zero => { 
                if status { self.flags |= ZERO_BIT; }
                else { self.flags &= NEG_ZERO_BIT; }
            }
            Flag::InterruptDisable => { 
                if status { self.flags |= INTERRUPTDISABLE_BIT; }
                else { self.flags &= NEG_INTERRUPTDISABLE_BIT; }
            }
            Flag::DecimalMode => { 
                if status { self.flags |= DECIMALMODE_BIT; }
                else { self.flags &= NEG_DECIMALMODE_BIT; }
            }
            Flag::Break => { 
                if status { self.flags |= BREAK_BIT; }
                else { self.flags &= NEG_BREAK_BIT; }
            }
            Flag::Overflow => { 
                if status { self.flags |= OVERFLOW_BIT; }
                else { self.flags &= NEG_OVERFLOW_BIT; }
            }
            Flag::Negative => { 
                if status { self.flags |= NEGATIVE_BIT; }
                else { self.flags &= NEG_NEGATIVE_BIT; }
            }
        }
    }

    pub fn print_cpu(&self)
    {
        println!("Program Counter: {:04X}", self.program_counter);
        println!("Stack Pointer: {:02X}", self.stack_pointer);
        println!("Accumulator: {:02X}", self.reg_accumulator);
        println!("X Index: {:02X}", self.reg_xindex);
        println!("Y Index: {:02X}", self.reg_yindex);
        println!("Flags: {:08b}", self.flags);
    }

    pub fn interpret(&mut self, program: Vec<u8>)
    {
        loop 
        {
            let opcode: u8 = program[self.program_counter as usize];
            self.program_counter += 1;

            match opcode
            {
                0xA9 => {
                    let param: u8 = program[self.program_counter as usize];
                    self.program_counter += 1;
                    self.reg_accumulator = param;

                    if self.reg_accumulator == 0 { self.set_flag(Flag::Zero, true); }
                    else { self.set_flag(Flag::Zero, false); }

                    if self.reg_accumulator & NEGATIVE_BIT != 0 { self.set_flag(Flag::Negative, true); }
                    else { self.set_flag(Flag::Negative, false); }
                }
                0x00 => {
                    self.set_flag(Flag::Break, true);
                    break;
                }
                _ => {}
            }
        }
    }
}

fn main()
{

}

#[cfg(test)]
mod test {
   use super::*;
 
   #[test]
   fn t0xa9_lda_load_accumulator() {
       let mut cpu = CPU::new();
       cpu.interpret(vec![0xa9, 0x05, 0x00]);
       assert!(cpu.get_register(Register::ACCUMULATOR) == 0x05);
       assert!(cpu.get_flag(Flag::Zero) == false);
       assert!(cpu.get_flag(Flag::Negative) == false);
   }

    #[test]
    fn t0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.get_flag(Flag::Zero) == true);
    }
    #[test]
    fn t0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x80, 0x00]);
        assert!(cpu.get_flag(Flag::Negative) == true);
    }
}