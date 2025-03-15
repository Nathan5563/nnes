use crate::nnes::{Register, NNES};

#[derive(Copy, Clone, PartialEq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

enum RegisterOffset {
    None,
    XIndex,
    YIndex,
}

impl NNES {
    pub fn memory_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn memory_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn reset_memory(&mut self) {
        self.memory = [0; 0xffff];
    }

    fn handle_immediate(&mut self) -> u16 {
        let pc: u16 = self.get_program_counter();
        let op: u8 = self.memory_read(pc);
        self.set_program_counter(pc + 1);
        op as u16
    }

    fn handle_zero_page(&mut self, index: RegisterOffset) -> u16 {
        let pc: u16 = self.get_program_counter();
        let addr: u8 = self.memory_read(pc);
        self.set_program_counter(pc + 1);
        match index {
            RegisterOffset::None => addr as u16,
            RegisterOffset::XIndex => (addr + self.get_register(Register::XIndex)) as u16,
            RegisterOffset::YIndex => (addr + self.get_register(Register::YIndex)) as u16,
        }
    }

    fn handle_relative(&mut self) -> u16 {
        let pc: u16 = self.get_program_counter();
        let offset: u8 = self.memory_read(pc);
        self.set_program_counter(pc + 1);
        offset as u16
    }

    fn handle_absolute(&mut self, index: RegisterOffset) -> u16 {
        let pc: u16 = self.get_program_counter();
        let addr_lower: u8 = self.memory_read(pc);
        let addr_higher: u8 = self.memory_read(pc + 1);
        self.set_program_counter(pc + 2);
        let addr: u16 = ((addr_higher as u16) << 8) | (addr_lower as u16);
        match index {
            RegisterOffset::None => addr,
            RegisterOffset::XIndex => addr + self.get_register(Register::XIndex) as u16,
            RegisterOffset::YIndex => addr + self.get_register(Register::YIndex) as u16,
        }
    }

    fn handle_indirect(&mut self, index: RegisterOffset) -> u16 {
        let pc: u16 = self.get_program_counter();
        let indirect_lower: u8 = self.memory_read(pc);
        let indirect_higher: u8 = self.memory_read(pc + 1);
        self.set_program_counter(pc + 2);
        let indirect: u16 = ((indirect_higher as u16) << 8) | (indirect_lower as u16);
        let addr_lower: u8;
        let addr_higher: u8;
        let addr: u16;
        match index {
            RegisterOffset::None => {
                addr_lower = self.memory_read(indirect);
                addr_higher = self.memory_read(indirect + 1);
                addr = ((addr_higher as u16) << 8) | (addr_lower as u16);
                addr
            }
            RegisterOffset::XIndex => {
                let offset: u8 = self.get_register(Register::XIndex);
                addr_lower = self.memory_read(indirect + offset as u16);
                addr_higher = self.memory_read(indirect + offset as u16 + 1);
                addr = ((addr_higher as u16) << 8) | (addr_lower as u16);
                addr
            }
            RegisterOffset::YIndex => {
                let offset: u8 = self.get_register(Register::YIndex);
                addr_lower = self.memory_read(indirect);
                addr_higher = self.memory_read(indirect + 1);
                addr = ((addr_higher as u16) << 8) | (addr_lower as u16);
                addr + offset as u16
            }
        }
    }

    pub fn get_operand(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.handle_immediate(),
            AddressingMode::Relative => self.handle_relative(),
            AddressingMode::ZeroPage => self.handle_zero_page(RegisterOffset::None),
            AddressingMode::ZeroPageX => self.handle_zero_page(RegisterOffset::XIndex),
            AddressingMode::ZeroPageY => self.handle_zero_page(RegisterOffset::YIndex),
            AddressingMode::Absolute => self.handle_absolute(RegisterOffset::None),
            AddressingMode::AbsoluteX => self.handle_absolute(RegisterOffset::XIndex),
            AddressingMode::AbsoluteY => self.handle_absolute(RegisterOffset::YIndex),
            AddressingMode::Indirect => self.handle_indirect(RegisterOffset::None),
            AddressingMode::IndirectX => self.handle_indirect(RegisterOffset::XIndex),
            AddressingMode::IndirectY => self.handle_indirect(RegisterOffset::YIndex),
            _ => 0,
        }
    }
}
