use crate::nnes::*;

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

    pub fn load(&mut self, program: Vec<u8>)
    {
        self.memory[0x8000 .. (0x8000 + program.len())]
            .copy_from_slice(&program[..]);
        self.program_counter = 0x8000;
    }
}