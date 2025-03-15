use std::collections::HashMap;

use crate::nnes::{NNES, Register, Flag, AddressingMode};

pub struct OpCode {
    code: u8,
    instruction: String,
    bytes: u8,
    cycles: u8,
    mode: AddressingMode,
}

impl OpCode {
    pub fn new(
        code: u8,
        instruction: String,
        bytes: u8,
        cycles: u8,
        mode: AddressingMode,
    ) -> Self {
        OpCode {
            code: code,
            instruction: instruction,
            bytes: bytes,
            cycles: cycles,
            mode: mode,
        }
    }

    pub fn get_addressing_mode(&self) -> AddressingMode {
        self.mode
    }
}

lazy_static! {
    pub static ref opcodes_list: Vec<OpCode> = vec![
        OpCode::new(0x00, "BRK".to_string(), 1, 7, AddressingMode::Implied),
        OpCode::new(0xaa, "TAX".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0xe8, "INX".to_string(), 1, 2, AddressingMode::Implied),

        OpCode::new(0xa9, "LDA".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa5, "LDA".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb5, "LDA".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xad, "LDA".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbd, "LDA".to_string(), 3, 4 /* + 1 if page crossed */, AddressingMode::AbsoluteX),
        OpCode::new(0xb9, "LDA".to_string(), 3, 4 /* + 1 if page crossed */, AddressingMode::AbsoluteY),
        OpCode::new(0xa1, "LDA".to_string(), 2, 6, AddressingMode::AbsoluteX),
        OpCode::new(0xb1, "LDA".to_string(), 2, 5 /* + 1 if page crossed */, AddressingMode::AbsoluteX),
        
        OpCode::new(0x85, "STA".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x8d, "STA".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9d, "STA".to_string(), 3, 5, AddressingMode::AbsoluteX),
        OpCode::new(0x99, "STA".to_string(), 3, 5, AddressingMode::AbsoluteY),
        OpCode::new(0x81, "STA".to_string(), 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x91, "STA".to_string(), 2, 6, AddressingMode::IndirectY),
    ];

    pub static ref opcodes_map: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for op in &*opcodes_list {
            map.insert(op.code, op);
        };
        map
    };
}

impl NNES {
    pub fn handle_brk(&mut self) {
        self.set_flag(Flag::Break, true);
    }

    pub fn handle_lda(&mut self, mode: AddressingMode) {
        let op: u16 = self.get_operand(mode);
        let mut data: u16 = op;
        if mode != AddressingMode::Immediate { 
            data = self.memory_read(op) as u16; 
        }
        self.set_register(Register::ACCUMULATOR, data as u8);
        self.update_op_flags(data as u8);
    }

    pub fn handle_tax(&mut self) {
        let reg_acc: u8 = self.get_register(Register::ACCUMULATOR);
        self.set_register(Register::XIndex, reg_acc);
        self.update_op_flags(reg_acc);
    }

    pub fn handle_inx(&mut self) {
        let reg_x: u8 = self.get_register(Register::XIndex);
        if reg_x == 0xff {
            self.set_register(Register::XIndex, 0);
            self.update_op_flags(0);
        } else {
            self.set_register(Register::XIndex, reg_x + 1);
            self.update_op_flags(reg_x + 1);
        }
    }

    pub fn handle_sta(&mut self) {
        
    }
}
