use std::collections::HashMap;

use crate::nnes::{IRQ_VECTOR, NNES};
use crate::nnes::memory::{AddressingMode, Mem};
use crate::nnes::cpu::flags::Flag;
use crate::nnes::cpu::registers::Register;
use crate::types::{BIT_0, BIT_6, BIT_7, LOWER_BYTE, UPPER_BYTE};

pub struct OpCode {
    code: u8,
    instruction: String,
    bytes: u8,
    cycles: u8,
    mode: AddressingMode,
}

impl OpCode {
    pub fn new(code: u8, instruction: String, bytes: u8, cycles: u8, mode: AddressingMode) -> Self {
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
    // CPU
        // Transfer
        OpCode::new(0xaa, "TAX".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0xa8, "TAY".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0xba, "TSX".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0x8a, "TXA".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0x9a, "TXS".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0x98, "TYA".to_string(), 1, 2, AddressingMode::Implied),
        // Flags
        OpCode::new(0x18, "CLC".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0xd8, "CLD".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0x58, "CLI".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0xb8, "CLV".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0x38, "SEC".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0xf8, "SED".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0x78, "SEI".to_string(), 1, 2, AddressingMode::Implied),

    // Memory
        // Load
        OpCode::new(0xa9, "LDA".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa5, "LDA".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb5, "LDA".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xad, "LDA".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbd, "LDA".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0xb9, "LDA".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0xa1, "LDA".to_string(), 2, 6, AddressingMode::IndirectX),
        OpCode::new(0xb1, "LDA".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY),
        OpCode::new(0xa2, "LDX".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa6, "LDX".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb6, "LDX".to_string(), 2, 4, AddressingMode::ZeroPageY),
        OpCode::new(0xae, "LDX".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbe, "LDX".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0xa0, "LDY".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa4, "LDY".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb4, "LDY".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xac, "LDY".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbc, "LDY".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX),
        // Store
        OpCode::new(0x85, "STA".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x8d, "STA".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9d, "STA".to_string(), 3, 5, AddressingMode::AbsoluteX),
        OpCode::new(0x99, "STA".to_string(), 3, 5, AddressingMode::AbsoluteY),
        OpCode::new(0x81, "STA".to_string(), 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x91, "STA".to_string(), 2, 6, AddressingMode::IndirectY),
        OpCode::new(0x86, "STX".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x96, "STX".to_string(), 2, 4, AddressingMode::ZeroPageY),
        OpCode::new(0x8e, "STX".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0x84, "STY".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x94, "STY".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x8c, "STY".to_string(), 3, 4, AddressingMode::Absolute),
        // Stack
        OpCode::new(0x48, "PHA".to_string(), 1, 3, AddressingMode::Implied),
        OpCode::new(0x08, "PHP".to_string(), 1, 3, AddressingMode::Implied),
        OpCode::new(0x68, "PLA".to_string(), 1, 4, AddressingMode::Implied),
        OpCode::new(0x28, "PLP".to_string(), 1, 4, AddressingMode::Implied),

    // Binary
        // AND
        OpCode::new(0x29, "AND".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, "AND".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, "AND".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x2d, "AND".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3d, "AND".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x39, "AND".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0x21, "AND".to_string(), 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x31, "AND".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY),
        // OR
        OpCode::new(0x09, "ORA".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0x05, "ORA".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x15, "ORA".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x0d, "ORA".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1d, "ORA".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x19, "ORA".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0x01, "ORA".to_string(), 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x11, "ORA".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY),
        // XOR
        OpCode::new(0x49, "EOR".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, "EOR".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, "EOR".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x4d, "EOR".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5d, "EOR".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x59, "EOR".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0x41, "EOR".to_string(), 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x51, "EOR".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY),
        // SAL
        OpCode::new(0x0a, "ASL".to_string(), 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x06, "ASL".to_string(), 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, "ASL".to_string(), 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x0e, "ASL".to_string(), 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1e, "ASL".to_string(), 3, 7, AddressingMode::AbsoluteX),
        // SHR
        OpCode::new(0x4a, "LSR".to_string(), 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x46, "LSR".to_string(), 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x56, "LSR".to_string(), 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x4e, "LSR".to_string(), 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5e, "LSR".to_string(), 3, 7, AddressingMode::AbsoluteX),
        // RCL
        OpCode::new(0x2a, "ROL".to_string(), 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x26, "ROL".to_string(), 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x36, "ROL".to_string(), 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x2e, "ROL".to_string(), 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3e, "ROL".to_string(), 3, 7, AddressingMode::AbsoluteX),
        // RCR
        OpCode::new(0x6a, "ROR".to_string(), 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x66, "ROR".to_string(), 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x76, "ROR".to_string(), 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x6e, "ROR".to_string(), 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7e, "ROR".to_string(), 3, 7, AddressingMode::AbsoluteX),

    // Arithmetic
        // ADC
        OpCode::new(0x69, "ADC".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, "ADC".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, "ADC".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x6d, "ADC".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7d, "ADC".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x79, "ADC".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0x61, "ADC".to_string(), 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x71, "ADC".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY),
        // SBC
        OpCode::new(0xe9, "SBC".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0xe5, "SBC".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xf5, "SBC".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xed, "SBC".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0xfd, "SBC".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0xf9, "SBC".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0xe1, "SBC".to_string(), 2, 6, AddressingMode::IndirectX),
        OpCode::new(0xf1, "SBC".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY),
        // INC
        OpCode::new(0xe6, "INC".to_string(), 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xf6, "INC".to_string(), 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0xee, "INC".to_string(), 3, 6, AddressingMode::Absolute),
        OpCode::new(0xfe, "INC".to_string(), 3, 7, AddressingMode::AbsoluteX),
        OpCode::new(0xe8, "INX".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0xc8, "INY".to_string(), 1, 2, AddressingMode::Implied),
        // DEC
        OpCode::new(0xc6, "DEC".to_string(), 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xd6, "DEC".to_string(), 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0xce, "DEC".to_string(), 3, 6, AddressingMode::Absolute),
        OpCode::new(0xde, "DEC".to_string(), 3, 7, AddressingMode::AbsoluteX),
        OpCode::new(0xca, "DEX".to_string(), 1, 2, AddressingMode::Implied),
        OpCode::new(0x88, "DEY".to_string(), 1, 2, AddressingMode::Implied),

    // Control flow
        // Stop
        OpCode::new(0x00, "BRK".to_string(), 1, 7, AddressingMode::Implied),
        OpCode::new(0xea, "NOP".to_string(), 1, 2, AddressingMode::Implied),
        // Compare
        OpCode::new(0xc9, "CMP".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc5, "CMP".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xd5, "CMP".to_string(), 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xcd, "CMP".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0xdd, "CMP".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0xd9, "CMP".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0xc1, "CMP".to_string(), 2, 6, AddressingMode::IndirectX),
        OpCode::new(0xd1, "CMP".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY),
        OpCode::new(0xe0, "CPX".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0xe4, "CPX".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xec, "CPX".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0xc0, "CPY".to_string(), 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc4, "CPY".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xcc, "CPY".to_string(), 3, 4, AddressingMode::Absolute),
        // Jump
        OpCode::new(0x4c, "JMP".to_string(), 3, 3, AddressingMode::Absolute),
        OpCode::new(0x6c, "JMP".to_string(), 3, 5, AddressingMode::Indirect), // 6502 bug with 0xXXFF
        OpCode::new(0x20, "JSR".to_string(), 3, 6, AddressingMode::Absolute),
        OpCode::new(0x40, "RTI".to_string(), 1, 6, AddressingMode::Implied),
        OpCode::new(0x60, "RTS".to_string(), 1, 6, AddressingMode::Implied),
        // Conditionals
        OpCode::new(0x90, "BCC".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0xb0, "BCS".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0xf0, "BEQ".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0x24, "BIT".to_string(), 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2c, "BIT".to_string(), 3, 4, AddressingMode::Absolute),
        OpCode::new(0x30, "BMI".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0xd0, "BNE".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0x10, "BPL".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0x50, "BVC".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0x70, "BVS".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
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
    pub fn handle_tax(&mut self) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        self.set_register_with_flags(Register::XIndex, reg_acc);
    }

    pub fn handle_tay(&mut self) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        self.set_register_with_flags(Register::YIndex, reg_acc);
    }

    pub fn handle_tsx(&mut self) {
        let stk_ptr: u8 = self.get_stack_pointer();
        self.set_register_with_flags(Register::XIndex, stk_ptr);
    }

    pub fn handle_txa(&mut self) {
        let reg_x: u8 = self.get_register(Register::XIndex);
        self.set_register_with_flags(Register::Accumulator, reg_x);
    }

    pub fn handle_txs(&mut self) {
        let reg_x: u8 = self.get_register(Register::XIndex);
        self.set_stack_pointer_with_flags(reg_x);
    }

    pub fn handle_tya(&mut self) {
        let reg_y: u8 = self.get_register(Register::YIndex);
        self.set_register_with_flags(Register::Accumulator, reg_y);
    }

    pub fn handle_clc(&mut self) {
        self.set_flag(Flag::Carry, false);
    }

    pub fn handle_cld(&mut self) {
        self.set_flag(Flag::DecimalMode, false);
    }

    pub fn handle_cli(&mut self) {
        self.set_flag(Flag::InterruptDisable, false);
    }

    pub fn handle_clv(&mut self) {
        self.set_flag(Flag::Overflow, false);
    }

    pub fn handle_sec(&mut self) {
        self.set_flag(Flag::Carry, true);
    }

    pub fn handle_sed(&mut self) {
        self.set_flag(Flag::DecimalMode, true);
    }

    pub fn handle_sei(&mut self) {
        self.set_flag(Flag::InterruptDisable, true);
    }

    pub fn handle_lda(&mut self, mode: AddressingMode) {
        let op: u16 = self.get_operand(mode);
        let mut data: u16 = op;
        if mode != AddressingMode::Immediate {
            data = self.memory_read_u8(op) as u16;
        }
        self.set_register_with_flags(Register::Accumulator, data as u8);
    }

    pub fn handle_ldx(&mut self, mode: AddressingMode) {
        let op: u16 = self.get_operand(mode);
        let mut data: u16 = op;
        if mode != AddressingMode::Immediate {
            data = self.memory_read_u8(op) as u16;
        }
        self.set_register_with_flags(Register::XIndex, data as u8);
    }

    pub fn handle_ldy(&mut self, mode: AddressingMode) {
        let op: u16 = self.get_operand(mode);
        let mut data: u16 = op;
        if mode != AddressingMode::Immediate {
            data = self.memory_read_u8(op) as u16;
        }
        self.set_register_with_flags(Register::YIndex, data as u8);
    }

    pub fn handle_sta(&mut self, mode: AddressingMode) {
        let data: u8 = self.get_register(Register::Accumulator);
        let addr: u16 = self.get_operand(mode);
        self.memory_write_u8(addr, data);
    }

    pub fn handle_stx(&mut self, mode: AddressingMode) {
        let data: u8 = self.get_register(Register::XIndex);
        let addr: u16 = self.get_operand(mode);
        self.memory_write_u8(addr, data);
    }

    pub fn handle_sty(&mut self, mode: AddressingMode) {
        let data: u8 = self.get_register(Register::YIndex);
        let addr: u16 = self.get_operand(mode);
        self.memory_write_u8(addr, data);
    }

    pub fn handle_pha(&mut self) {
        self.stack_push_u8(self.get_register(Register::Accumulator));
    }

    pub fn handle_php(&mut self) {
        self.set_flag(Flag::Break, true);
        self.stack_push_u8(self.get_flags());
    }

    pub fn handle_pla(&mut self) {
        let data: u8 = self.stack_pop_u8();
        self.set_register_with_flags(Register::Accumulator, data);
    }

    pub fn handle_plp(&mut self) {
        let data: u8 = self.stack_pop_u8();
        self.set_flags(data);
        self.set_flag(Flag::Break, false);
    }

    pub fn handle_and(&mut self, mode: AddressingMode) {
        let data: u8 = self.get_data(mode);
        let res: u8 = self.get_register(Register::Accumulator) & data as u8;
        self.set_register_with_flags(Register::Accumulator, res);
    }

    pub fn handle_ora(&mut self, mode: AddressingMode) {
        let data: u8 = self.get_data(mode);
        let res: u8 = self.get_register(Register::Accumulator) | data;
        self.set_register_with_flags(Register::Accumulator, res);
    }

    pub fn handle_eor(&mut self, mode: AddressingMode) {
        let data: u8 = self.get_data(mode);
        let res: u8 = self.get_register(Register::Accumulator) ^ data;
        self.set_register_with_flags(Register::Accumulator, res);
    }

    fn shift(&mut self, mode: AddressingMode, left: bool, with_carry: bool) {
        let cf: bool = self.get_flag(Flag::Carry);
        if mode == AddressingMode::Accumulator {
            let mut reg_acc: u8 = self.get_register(Register::Accumulator);
            if left {
                self.set_flag(Flag::Carry, (reg_acc & BIT_7) != 0);
                reg_acc <<= 1;
                if with_carry && cf {
                    reg_acc |= BIT_0;
                }
            } else {
                self.set_flag(Flag::Carry, (reg_acc & BIT_0) != 0);
                reg_acc >>= 1;
                if with_carry && cf {
                    reg_acc |= BIT_7;
                }
            }
            self.set_register_with_flags(Register::Accumulator, reg_acc);
        } else {
            let op: u16 = self.get_operand(mode);
            let mut data: u8 = self.memory_read_u8(op);
            if left {
                self.set_flag(Flag::Carry, (data & BIT_7) != 0);
                data <<= 1;
                if with_carry && cf {
                    data |= BIT_0
                }
            } else {
                self.set_flag(Flag::Carry, (data & BIT_0) != 0);
                data >>= 1;
                if with_carry && cf {
                    data |= BIT_7
                }
            }
            self.memory_write_u8(op, data);
            self.update_op_flags(data);
        }
    }

    pub fn handle_asl(&mut self, mode: AddressingMode) {
        self.shift(mode, true, false);
    }

    pub fn handle_lsr(&mut self, mode: AddressingMode) {
        self.shift(mode, false, false);
    }

    pub fn handle_rol(&mut self, mode: AddressingMode) {
        self.shift(mode, true, true);
    }

    pub fn handle_ror(&mut self, mode: AddressingMode) {
        self.shift(mode, false, true);
    }

    fn add(&mut self, num1: u8, mut num2: u8, twos_complement: bool, with_carry: bool) -> u8 {
        let mut cbit: u8 = 0;
        if self.get_flag(Flag::Carry) && with_carry {
            cbit = 1;
            self.set_flag(Flag::Carry, false);
        }
        let mut res: u16;
        if twos_complement {
            num2 = num2.wrapping_neg();
            cbit ^= 1;
            res = num1 as u16 + num2 as u16;
            if with_carry {
                res -= cbit as u16;
            }
        } else {
            res = num1 as u16 + num2 as u16;
            if with_carry {
                res += cbit as u16;
            }
        }
        if with_carry {
            if res & UPPER_BYTE as u16 != 0 {
                self.set_flag(Flag::Carry, true);
            }
            res &= LOWER_BYTE;
            if (num1 ^ res as u8) & (num2 ^ res as u8) & BIT_7 != 0 {
                self.set_flag(Flag::Overflow, true);
            } else {
                self.set_flag(Flag::Overflow, false);
            }
        }
        else {
            res &= LOWER_BYTE;
        }
        res as u8
    }

    pub fn handle_adc(&mut self, mode: AddressingMode) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        let data: u8 = self.get_data(mode);
        let sum: u8 = self.add(reg_acc, data, false, true);
        self.set_register_with_flags(Register::Accumulator, sum);
    }

    pub fn handle_sbc(&mut self, mode: AddressingMode) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        let data: u8 = self.get_data(mode);
        let difference: u8 = self.add(reg_acc, data, true, true);
        self.set_register_with_flags(Register::Accumulator, difference);
    }

    pub fn handle_inc(&mut self, mode: AddressingMode) {
        let op: u16 = self.get_operand(mode);
        let mut data: u8 = self.memory_read_u8(op);
        if data == 0xff {
            data = 0;
        } else {
            data += 1;
        }
        self.memory_write_u8(op, data);
        self.update_op_flags(data);
    }

    pub fn handle_inx(&mut self) {
        let mut reg_x: u8 = self.get_register(Register::XIndex);
        if reg_x == 0xff {
            reg_x = 0;
        } else {
            reg_x += 1;
        }
        self.set_register_with_flags(Register::XIndex, reg_x);
    }

    pub fn handle_iny(&mut self) {
        let mut reg_y: u8 = self.get_register(Register::YIndex);
        if reg_y == 0xff {
            reg_y = 0;
        } else {
            reg_y += 1;
        }
        self.set_register_with_flags(Register::YIndex, reg_y);
    }

    pub fn handle_dec(&mut self, mode: AddressingMode) {
        let op: u16 = self.get_operand(mode);
        let mut data: u8 = self.memory_read_u8(op);
        if data == 0 {
            data = 0xff;
        } else {
            data -= 1;
        }
        self.memory_write_u8(op, data);
        self.update_op_flags(data);
    }

    pub fn handle_dex(&mut self) {
        let mut reg_x: u8 = self.get_register(Register::XIndex);
        if reg_x == 0 {
            reg_x = 0xff;
        } else {
            reg_x -= 1;
        }
        self.set_register_with_flags(Register::XIndex, reg_x);
    }

    pub fn handle_dey(&mut self) {
        let mut reg_y: u8 = self.get_register(Register::YIndex);
        if reg_y == 0 {
            reg_y = 0xff;
        } else {
            reg_y -= 1;
        }
        self.set_register_with_flags(Register::YIndex, reg_y);
    }

    pub fn handle_brk(&mut self) {
        self.set_program_counter(self.get_program_counter() + 1); // implied padding byte
        self.stack_push_u16(self.get_program_counter());
        self.handle_php();
        let irq: u16 = self.memory_read_u16(IRQ_VECTOR);
        self.set_program_counter(irq);
    }

    pub fn handle_nop(&mut self) {}

    pub fn handle_cmp(&mut self, mode: AddressingMode) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        let data: u8 = self.get_data(mode);
        let diff = self.add(reg_acc, data, true, false);
        self.update_op_flags(diff);
        if reg_acc >= data {
            self.set_flag(Flag::Carry, true);
        } else {
            self.set_flag(Flag::Carry, false);
        }
    }

    pub fn handle_cmx(&mut self, mode: AddressingMode) {
        let reg_x: u8 = self.get_register(Register::XIndex);
        let data: u8 = self.get_data(mode);
        let diff = self.add(reg_x, data, true, false);
        self.update_op_flags(diff);
        if reg_x >= data {
            self.set_flag(Flag::Carry, true);
        } else {
            self.set_flag(Flag::Carry, false);
        }
    }

    pub fn handle_cmy(&mut self, mode: AddressingMode) {
        let reg_y: u8 = self.get_register(Register::YIndex);
        let data: u8 = self.get_data(mode);
        let diff = self.add(reg_y, data, true, false);
        self.update_op_flags(diff);
        if reg_y >= data {
            self.set_flag(Flag::Carry, true);
        } else {
            self.set_flag(Flag::Carry, false);
        }
    }

    pub fn handle_jmp(&mut self, mode: AddressingMode) {
        let op: u16;
        if mode == AddressingMode::Absolute {
            op = self.get_operand(mode);
            self.set_program_counter(op);
        }
        else {
            let pc: u16 = self.get_program_counter();
            let indirect: u16 = self.memory_read_u16(pc);
            self.set_program_counter(pc + 2);
            if indirect & LOWER_BYTE == LOWER_BYTE {
                let lower_byte: u8 = self.memory_read_u8(indirect);
                let upper_byte: u8 = self.memory_read_u8(indirect & UPPER_BYTE);
                op = ((upper_byte as u16) << 8) | (lower_byte as u16);
                self.set_program_counter(op);
            }
            else {
                op = self.memory_read_u16(indirect);
                self.set_program_counter(op);
            }
        }
    }

    pub fn handle_jsr(&mut self) {
        let op: u16 = self.get_operand(AddressingMode::Absolute);
        self.stack_push_u16(self.get_program_counter() - 1);
        self.set_program_counter(op);
    }

    pub fn handle_rti(&mut self) {
        let flags: u8 = self.stack_pop_u8();
        let pc: u16 = self.stack_pop_u16();
        self.set_flags(flags);
        self.set_flag(Flag::Break, false);
        self.set_program_counter(pc);
    }

    pub fn handle_rts(&mut self) {
        let pc: u16 = self.stack_pop_u16();
        self.set_program_counter(pc + 1);
    }

    fn branch(&mut self, jmp: bool) {
        let op: i8 = self.get_operand(AddressingMode::Relative) as i8;
        if jmp {
            let res: i64 = (self.get_program_counter() as i64) + (op as i64);
            self.set_program_counter(res as u16);
        }
    }

    pub fn handle_bcc(&mut self) {
        self.branch(!self.get_flag(Flag::Carry));
    }

    pub fn handle_bcs(&mut self) {
        self.branch(self.get_flag(Flag::Carry));
    }

    pub fn handle_beq(&mut self) {
        self.branch(self.get_flag(Flag::Zero));
    }

    pub fn handle_bit(&mut self, mode: AddressingMode) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        let data: u8 = self.get_data(mode);
        self.set_flag(Flag::Overflow, data & BIT_6 != 0);
        self.set_flag(Flag::Negative, data & BIT_7 != 0);
        self.set_flag(Flag::Zero, data & reg_acc == 0);
    }

    pub fn handle_bmi(&mut self) {
        self.branch(self.get_flag(Flag::Negative));
    }

    pub fn handle_bne(&mut self) {
        self.branch(!self.get_flag(Flag::Zero));
    }

    pub fn handle_bpl(&mut self) {
        self.branch(!self.get_flag(Flag::Negative));
    }

    pub fn handle_bvc(&mut self) {
        self.branch(!self.get_flag(Flag::Overflow));
    }

    pub fn handle_bvs(&mut self) {
        self.branch(self.get_flag(Flag::Overflow));
    }
}
