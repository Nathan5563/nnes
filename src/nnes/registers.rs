use crate::nnes::*;

impl NNES
{
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
}
