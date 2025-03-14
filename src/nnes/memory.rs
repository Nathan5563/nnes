use crate::nnes::NNES;

pub enum AddressingMode {
    Implied,
    Immediate,
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}

impl NNES
{
    pub fn memory_read(&self, addr: u16) -> u8
    {
        self.memory[addr as usize]
    }

    pub fn memory_write(&mut self, addr: u16, data: u8)
    {
        self.memory[addr as usize] = data;
    }

    pub fn reset_memory(&mut self)
    {
        self.memory = [0; 0xffff];
    }
}