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

pub static STACK_OFFSET: u16 = 0x100;

enum RegisterOffset {
    None,
    XIndex,
    YIndex,
}

impl NNES {
    pub fn memory_read_u8(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn memory_write_u8(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn memory_read_u16(&self, addr: u16) -> u16 {
        if addr == 0xffff {
            panic!("Can not read two bytes at one byte location (end of memory reached)");
        }
        let low: u8 = self.memory[addr as usize];
        let high: u8 = self.memory[addr as usize + 1];
        ((high as u16) << 8) | (low as u16)
    }

    pub fn memory_write_u16(&mut self, addr: u16, data: u16) {
        if addr == 0xffff {
            panic!("Can not write two bytes at one byte location (end of memory reached)");
        }
        let low: u16 = data & 0x00ff;
        let high: u16 = (data & 0xff00) >> 8;
        self.memory[addr as usize] = low as u8;
        self.memory[addr as usize + 1] = high as u8;
    }

    pub fn stack_push(&mut self, data: u8) {
        let mut stk_ptr: u8 = self.get_stack_pointer();
        self.memory_write_u8(STACK_OFFSET + stk_ptr as u16, data);
        if stk_ptr == 0 {
            stk_ptr = 0xff;
        }
        else {
            stk_ptr -= 1;
        }
        self.set_stack_pointer(stk_ptr);
    }

    pub fn stack_pop(&mut self) -> u8 {
        let mut stk_ptr: u8 = self.get_stack_pointer();
        if stk_ptr == 0xff {
            stk_ptr = 0;
        }
        else {
            stk_ptr += 1;
        }
        self.set_stack_pointer(stk_ptr);
        self.memory_read_u8(STACK_OFFSET + stk_ptr as u16)
    }

    pub fn reset_memory(&mut self) {
        self.memory = [0; 0xffff];
    }

    fn handle_immediate(&mut self) -> u16 {
        let pc: u16 = self.get_program_counter();
        let op: u8 = self.memory_read_u8(pc);
        self.set_program_counter(pc + 1);
        op as u16
    }

    fn handle_zero_page(&mut self, index: RegisterOffset) -> u16 {
        let pc: u16 = self.get_program_counter();
        let addr: u8 = self.memory_read_u8(pc);
        self.set_program_counter(pc + 1);
        match index {
            RegisterOffset::None => addr as u16,
            RegisterOffset::XIndex => (addr + self.get_register(Register::XIndex)) as u16,
            RegisterOffset::YIndex => (addr + self.get_register(Register::YIndex)) as u16,
        }
    }

    fn handle_relative(&mut self) -> u16 {
        let pc: u16 = self.get_program_counter();
        let offset: u8 = self.memory_read_u8(pc);
        self.set_program_counter(pc + 1);
        offset as u16
    }

    fn handle_absolute(&mut self, index: RegisterOffset) -> u16 {
        let pc: u16 = self.get_program_counter();
        let addr: u16 = self.memory_read_u16(pc);
        self.set_program_counter(pc + 2);
        match index {
            RegisterOffset::None => addr,
            RegisterOffset::XIndex => addr + self.get_register(Register::XIndex) as u16,
            RegisterOffset::YIndex => addr + self.get_register(Register::YIndex) as u16,
        }
    }

    fn handle_indirect(&mut self, index: RegisterOffset) -> u16 {
        let pc: u16 = self.get_program_counter();
        let indirect: u16 = self.memory_read_u16(pc);
        self.set_program_counter(pc + 2);
        match index {
            RegisterOffset::None => {
                self.memory_read_u16(indirect)
            }
            RegisterOffset::XIndex => {
                let offset: u8 = self.get_register(Register::XIndex);
                self.memory_read_u16(indirect + offset as u16)
            }
            RegisterOffset::YIndex => {
                let offset: u8 = self.get_register(Register::YIndex);
                self.memory_read_u16(indirect) + offset as u16
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
