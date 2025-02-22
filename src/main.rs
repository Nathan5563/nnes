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

struct NNES
{
    program_counter: u16,
    stack_pointer: u8,
    reg_accumulator: u8,
    reg_xindex: u8,
    reg_yindex: u8,
    flags: u8,
    memory: [u8; 0xffff],
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

    fn memory_read(&self, addr: u16) -> u8
    {
        self.memory[addr as usize]
    }

    fn memory_write(&mut self, addr: u16, data: u8)
    {
        self.memory[addr as usize] = data;
    }

    pub fn load(&mut self, program: Vec<u8>)
    {
        self.memory[0x8000 .. (0x8000 + program.len())]
            .copy_from_slice(&program[..]);
        self.program_counter = 0x8000;
    }

    fn update_nz_flags(&mut self, res: u8)
    {
        if res == 0 { self.set_flag(Flag::Zero, true); }
        else { self.set_flag(Flag::Zero, false); }

        if res & NEGATIVE_BIT != 0 { self.set_flag(Flag::Negative, true); }
        else { self.set_flag(Flag::Negative, false); }
    }

    fn handle_brk(&mut self)
    {
        self.set_flag(Flag::Break, true);
    }
    fn handle_lda(&mut self)
    {
        let pc: u16 = self.get_program_counter();
        let param: u8 = self.memory_read(pc);
        self.set_program_counter(pc + 1);
        self.set_register(Register::ACCUMULATOR, param);
        self.update_nz_flags(param);        
    }
    fn handle_tax(&mut self)
    {
        let reg_acc: u8 = self.get_register(Register::ACCUMULATOR);
        self.set_register(Register::XIndex, reg_acc);
        self.update_nz_flags(reg_acc);
    }
    fn handle_inx(&mut self)
    {
        let reg_x: u8 = self.get_register(Register::XIndex);
        if reg_x == 0xff
        { 
            self.set_register(Register::XIndex, 0); 
            self.update_nz_flags(0);
        }
        else 
        { 
            self.set_register(Register::XIndex, reg_x + 1); 
            self.update_nz_flags(reg_x + 1);
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
                0x00 => { self.handle_brk(); }
                0xa9 => { self.handle_lda(); }
                0xaa => { self.handle_tax(); }
                0xe8 => { self.handle_inx(); }
                _ => {}
            }
        }
    }

    pub fn reset(&mut self)
    {
        self.program_counter = 0;
        self.stack_pointer = 0;
        self.reg_accumulator = 0;
        self.reg_xindex = 0;
        self.reg_yindex = 0;
        self.flags = 0;
        self.memory = [0; 0xffff];
    }

    pub fn print_nnes(&self)
    {
        println!("Program Counter: {:04X}", self.program_counter);
        println!("Stack Pointer: {:02X}", self.stack_pointer);
        println!("Accumulator: {:02X}", self.reg_accumulator);
        println!("X Index: {:02X}", self.reg_xindex);
        println!("Y Index: {:02X}", self.reg_yindex);
        println!("Flags: {:08b}", self.flags);
    }
}

fn main()
{

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn t0x00_brk() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Break) == true);
    }

    #[test]
    fn t0xa9_lda_immediate() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0xa9, 0x05, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::ACCUMULATOR) == 0x05);
        assert!(nnes.get_flag(Flag::Zero) == false);
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0xa9_lda_zero_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0xa9, 0x00, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Zero) == true);
        nnes.reset();
        nnes.load(vec![0xa9, 0x80, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Zero) == false);
    }

    #[test]
    fn t0xa9_lda_negative_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0xa9, 0x80, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == true);
        nnes.reset();
        nnes.load(vec![0xa9, 0x7F, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0xaa_tax_implied() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 10);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 10);
    }

    #[test]
    fn t0xaa_tax_zero_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 0);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Zero) == true);
        nnes.reset();
        nnes.set_register(Register::ACCUMULATOR, 128);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Zero) == false);
    }

    #[test]
    fn t0xaa_tax_negative_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 128);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == true);
        nnes.reset();
        nnes.set_register(Register::ACCUMULATOR, 127);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0xe8_inx_implied() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::XIndex, 10);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 11);
    }

    #[test]
    fn t0xe8_inx_overflow() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::XIndex, 0xfe);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == true);
        nnes.reset();
        nnes.set_register(Register::XIndex, 0xff);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 0);
        assert!(nnes.get_flag(Flag::Zero) == true);
        assert!(nnes.get_flag(Flag::Negative) == false);
        nnes.reset();
        nnes.set_register(Register::XIndex, 0xff);
        nnes.load(vec![0xe8, 0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 1);
        assert!(nnes.get_flag(Flag::Zero) == false);
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0xe8_inx_negative_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::XIndex, 127);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == true);
        nnes.reset();
        nnes.set_register(Register::XIndex, 126);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0xa9_immediate_t0xaa_implied_t0xe8_implied_t0x00() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 0xc1);
        assert!(nnes.get_flag(Flag::Zero) == false);
        assert!(nnes.get_flag(Flag::Negative) == true);
    }
}