use crate::nnes::cpu::registers::Register;
use crate::nnes::NNES;
use crate::types::{LOWER_BYTE, UPPER_BYTE};
use crate::utils::{add_mod_16bit, add_mod_8bit};

#[derive(Debug, Copy, Clone, PartialEq)]
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

pub trait Mem {
    fn memory_read_u8(&self, addr: u16) -> u8;

    fn memory_write_u8(&mut self, addr: u16, data: u8);

    fn memory_read_u16(&self, addr: u16) -> u16 {
        let low: u8 = self.memory_read_u8(addr);
        let high: u8 = self.memory_read_u8(add_mod_16bit(addr, 1));
        ((high as u16) << 8) | (low as u16)
    }

    fn memory_write_u16(&mut self, addr: u16, data: u16) {
        let low: u16 = data & LOWER_BYTE;
        let high: u16 = (data & UPPER_BYTE) >> 8;
        self.memory_write_u8(addr, low as u8);
        self.memory_write_u8(add_mod_16bit(addr, 1), high as u8);
    }
}

impl Mem for NNES {
    fn memory_read_u8(&self, addr: u16) -> u8 {
        self.bus.memory_read_u8(addr)
    }

    fn memory_write_u8(&mut self, addr: u16, data: u8) {
        self.bus.memory_write_u8(addr, data);
    }

    fn memory_read_u16(&self, addr: u16) -> u16 {
        self.bus.memory_read_u16(addr)
    }

    fn memory_write_u16(&mut self, addr: u16, data: u16) {
        self.bus.memory_write_u16(addr, data);
    }
}

impl NNES {
    pub fn stack_push_u8(&mut self, data: u8) {
        let mut stk_ptr: u8 = self.get_stack_pointer();
        self.memory_write_u8(STACK_OFFSET + stk_ptr as u16, data);
        if stk_ptr == 0 {
            stk_ptr = 0xff;
        } else {
            stk_ptr -= 1;
        }
        self.set_stack_pointer(stk_ptr);
    }

    pub fn stack_pop_u8(&mut self) -> u8 {
        let mut stk_ptr: u8 = self.get_stack_pointer();
        if stk_ptr == 0xff {
            stk_ptr = 0;
        } else {
            stk_ptr += 1;
        }
        self.set_stack_pointer(stk_ptr);
        self.memory_read_u8(STACK_OFFSET + stk_ptr as u16)
    }

    pub fn stack_push_u16(&mut self, data: u16) {
        self.stack_push_u8((data >> 8) as u8);
        self.stack_push_u8((data & LOWER_BYTE) as u8);
    }

    pub fn stack_pop_u16(&mut self) -> u16 {
        let lower_byte: u8 = self.stack_pop_u8();
        let upper_byte: u8 = self.stack_pop_u8();
        ((upper_byte as u16) << 8) | (lower_byte as u16)
    }

    fn handle_immediate(&mut self) -> u16 {
        let pc: u16 = self.get_program_counter();
        let op: u8 = self.memory_read_u8(pc);
        self.set_program_counter(add_mod_16bit(pc, 1));
        op as u16
    }

    fn handle_zero_page(&mut self, index: RegisterOffset) -> u16 {
        let pc: u16 = self.get_program_counter();
        let addr: u8 = self.memory_read_u8(pc);
        self.set_program_counter(add_mod_16bit(pc, 1));
        match index {
            RegisterOffset::None => addr as u16,
            RegisterOffset::XIndex => {
                add_mod_8bit(addr, self.get_register(Register::XIndex)) as u16
            }
            RegisterOffset::YIndex => {
                add_mod_8bit(addr, self.get_register(Register::YIndex)) as u16
            }
        }
    }

    fn handle_relative(&mut self) -> u16 {
        let pc: u16 = self.get_program_counter();
        let offset: u8 = self.memory_read_u8(pc);
        self.set_program_counter(add_mod_16bit(pc, 1));
        offset as u16
    }

    fn handle_absolute(&mut self, index: RegisterOffset) -> u16 {
        let pc: u16 = self.get_program_counter();
        let addr: u16 = self.memory_read_u16(pc);
        self.set_program_counter(add_mod_16bit(pc, 2));
        match index {
            RegisterOffset::None => addr,
            RegisterOffset::XIndex => {
                add_mod_16bit(addr, self.get_register(Register::XIndex) as u16)
            }
            RegisterOffset::YIndex => {
                add_mod_16bit(addr, self.get_register(Register::YIndex) as u16)
            }
        }
    }

    fn handle_indirect_xy(&mut self, index: RegisterOffset) -> u16 {
        let pc: u16 = self.get_program_counter();
        let indirect: u8 = self.memory_read_u8(pc);
        self.set_program_counter(add_mod_16bit(pc, 1));
        match index {
            RegisterOffset::XIndex => {
                let offset: u8 = self.get_register(Register::XIndex);
                let eff_addr: u8 = add_mod_8bit(indirect, offset);
                let low: u8 = self.memory_read_u8(eff_addr as u16);
                let high: u8 = self.memory_read_u8(add_mod_8bit(eff_addr, 1) as u16);
                ((high as u16) << 8) | (low as u16)
            }
            RegisterOffset::YIndex => {
                let offset: u8 = self.get_register(Register::YIndex);
                let eff_addrl: u8 = self.memory_read_u8(indirect as u16);
                let eff_addrh: u8 = self.memory_read_u8(add_mod_8bit(indirect, 1) as u16);
                let eff_addr: u16 = ((eff_addrh as u16) << 8) | (eff_addrl as u16);
                add_mod_16bit(eff_addr, offset as u16)
            }
            _ => 0,
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
            AddressingMode::Indirect => self.handle_indirect_xy(RegisterOffset::None),
            AddressingMode::IndirectX => self.handle_indirect_xy(RegisterOffset::XIndex),
            AddressingMode::IndirectY => self.handle_indirect_xy(RegisterOffset::YIndex),
            _ => 0,
        }
    }

    pub fn get_data(&mut self, mode: AddressingMode) -> u8 {
        let op: u16 = self.get_operand(mode);
        let mut data: u16 = op;
        if mode != AddressingMode::Immediate {
            data = self.memory_read_u8(op) as u16;
        }
        data as u8
    }
}
