use crate::nnes::*;

impl NNES
{
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