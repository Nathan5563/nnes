use std::collections::HashMap;

use crate::nnes::cpu::flags::{Flag, BF, CF, DF, IF, NF, VF, ZF};
use crate::nnes::cpu::registers::Register;
use crate::nnes::memory::{AddressingMode, Mem};
use crate::nnes::{IRQ_VECTOR, NNES};
use crate::types::{BIT_0, BIT_6, BIT_7, LOWER_BYTE, UPPER_BYTE};
use crate::utils::{add_mod_16bit, add_mod_8bit};

pub struct OpCode {
    code: u8,
    instruction: String,
    bytes: u8,
    cycles: u8,
    mode: AddressingMode,
    handler: fn(&mut NNES, mode: AddressingMode, cycle: &mut u8),
}

impl OpCode {
    pub fn new(
        code: u8,
        instruction: String,
        bytes: u8,
        cycles: u8,
        mode: AddressingMode,
        handler: fn(&mut NNES, mode: AddressingMode, cycle: &mut u8),
    ) -> Self {
        OpCode {
            code: code,
            instruction: instruction,
            bytes: bytes,
            cycles: cycles,
            mode: mode,
            handler: handler,
        }
    }

    pub fn get_instruction(&self) -> &String {
        &self.instruction
    }

    pub fn get_bytes(&self) -> u8 {
        self.bytes
    }

    pub fn get_cycles(&self) -> u8 {
        self.cycles
    }

    pub fn get_addressing_mode(&self) -> AddressingMode {
        self.mode
    }

    pub fn get_handler(&self) -> &fn(&mut NNES, mode: AddressingMode, cycle: &mut u8) {
        &self.handler
    }
}

impl NNES {
    pub fn handle_tax(&mut self, _: AddressingMode, cycle: &mut u8) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        self.set_register_with_flags(Register::XIndex, reg_acc);
    }

    pub fn handle_tay(&mut self, _: AddressingMode, cycle: &mut u8) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        self.set_register_with_flags(Register::YIndex, reg_acc);
    }

    pub fn handle_tsx(&mut self, _: AddressingMode, cycle: &mut u8) {
        let stk_ptr: u8 = self.get_stack_pointer();
        self.set_register_with_flags(Register::XIndex, stk_ptr);
    }

    pub fn handle_txa(&mut self, _: AddressingMode, cycle: &mut u8) {
        let reg_x: u8 = self.get_register(Register::XIndex);
        self.set_register_with_flags(Register::Accumulator, reg_x);
    }

    pub fn handle_txs(&mut self, _: AddressingMode, cycle: &mut u8) {
        let reg_x: u8 = self.get_register(Register::XIndex);
        self.set_stack_pointer(reg_x);
    }

    pub fn handle_tya(&mut self, _: AddressingMode, cycle: &mut u8) {
        let reg_y: u8 = self.get_register(Register::YIndex);
        self.set_register_with_flags(Register::Accumulator, reg_y);
    }

    pub fn handle_clc(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.set_flag(Flag::Carry, false);
    }

    pub fn handle_cld(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.set_flag(Flag::DecimalMode, false);
    }

    pub fn handle_cli(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.set_flag(Flag::InterruptDisable, false);
    }

    pub fn handle_clv(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.set_flag(Flag::Overflow, false);
    }

    pub fn handle_sec(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.set_flag(Flag::Carry, true);
    }

    pub fn handle_sed(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.set_flag(Flag::DecimalMode, true);
    }

    pub fn handle_sei(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.set_flag(Flag::InterruptDisable, true);
    }

    pub fn handle_lda(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let op: u16 = self.get_operand(mode);
        let mut data: u16 = op;
        if mode != AddressingMode::Immediate {
            data = self.memory_read_u8(op) as u16;
        }
        self.set_register_with_flags(Register::Accumulator, data as u8);
    }

    pub fn handle_ldx(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let op: u16 = self.get_operand(mode);
        let mut data: u16 = op;
        if mode != AddressingMode::Immediate {
            data = self.memory_read_u8(op) as u16;
        }
        self.set_register_with_flags(Register::XIndex, data as u8);
    }

    pub fn handle_ldy(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let op: u16 = self.get_operand(mode);
        let mut data: u16 = op;
        if mode != AddressingMode::Immediate {
            data = self.memory_read_u8(op) as u16;
        }
        self.set_register_with_flags(Register::YIndex, data as u8);
    }

    pub fn handle_sta(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let data: u8 = self.get_register(Register::Accumulator);
        let addr: u16 = self.get_operand(mode);
        self.memory_write_u8(addr, data);
    }

    pub fn handle_stx(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let data: u8 = self.get_register(Register::XIndex);
        let addr: u16 = self.get_operand(mode);
        self.memory_write_u8(addr, data);
    }

    pub fn handle_sty(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let data: u8 = self.get_register(Register::YIndex);
        let addr: u16 = self.get_operand(mode);
        self.memory_write_u8(addr, data);
    }

    pub fn handle_pha(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.stack_push_u8(self.get_register(Register::Accumulator));
    }

    pub fn handle_php(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.stack_push_u8(self.get_flags() | BF);
    }

    pub fn handle_pla(&mut self, _: AddressingMode, cycle: &mut u8) {
        let data: u8 = self.stack_pop_u8();
        self.set_register_with_flags(Register::Accumulator, data);
    }

    pub fn handle_plp(&mut self, _: AddressingMode, cycle: &mut u8) {
        let flags: u8 = self.stack_pop_u8();
        self.set_flag(Flag::Carry, flags & CF != 0);
        self.set_flag(Flag::Zero, flags & ZF != 0);
        self.set_flag(Flag::InterruptDisable, flags & IF != 0);
        self.set_flag(Flag::DecimalMode, flags & DF != 0);
        self.set_flag(Flag::Overflow, flags & VF != 0);
        self.set_flag(Flag::Negative, flags & NF != 0);
    }

    pub fn handle_and(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let data: u8 = self.get_data(mode);
        let res: u8 = self.get_register(Register::Accumulator) & data as u8;
        self.set_register_with_flags(Register::Accumulator, res);
    }

    pub fn handle_ora(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let data: u8 = self.get_data(mode);
        let res: u8 = self.get_register(Register::Accumulator) | data;
        self.set_register_with_flags(Register::Accumulator, res);
    }

    pub fn handle_eor(&mut self, mode: AddressingMode, cycle: &mut u8) {
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

    pub fn handle_asl(&mut self, mode: AddressingMode, cycle: &mut u8) {
        self.shift(mode, true, false);
    }

    pub fn handle_lsr(&mut self, mode: AddressingMode, cycle: &mut u8) {
        self.shift(mode, false, false);
    }

    pub fn handle_rol(&mut self, mode: AddressingMode, cycle: &mut u8) {
        self.shift(mode, true, true);
    }

    pub fn handle_ror(&mut self, mode: AddressingMode, cycle: &mut u8) {
        self.shift(mode, false, true);
    }

    fn add(&mut self, num1: u8, mut num2: u8, twos_complement: bool) -> u8 {
        let cbit: u8;
        if self.get_flag(Flag::Carry) {
            cbit = 1;
        } else {
            cbit = 0;
        }

        let mut res: u16;
        if twos_complement {
            num2 = !num2;
            res = num1 as u16 + num2 as u16 + cbit as u16;
            if (res as i8) < 0 {
                self.set_flag(Flag::Carry, false);
            } else {
                self.set_flag(Flag::Carry, true);
            }
        } else {
            res = num1 as u16 + num2 as u16 + cbit as u16;
            if res > 0xff {
                self.set_flag(Flag::Carry, true);
            } else {
                self.set_flag(Flag::Carry, false);
            }
        }

        res &= LOWER_BYTE;
        if (num1 ^ res as u8) & (num2 ^ res as u8) & BIT_7 != 0 {
            self.set_flag(Flag::Overflow, true);
        } else {
            self.set_flag(Flag::Overflow, false);
        }

        res as u8
    }

    pub fn handle_adc(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        let data: u8 = self.get_data(mode);
        let sum: u8 = self.add(reg_acc, data, false);
        self.set_register_with_flags(Register::Accumulator, sum);
    }

    pub fn handle_sbc(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        let data: u8 = self.get_data(mode);
        let difference: u8 = self.add(reg_acc, data, true);
        self.set_register_with_flags(Register::Accumulator, difference);
    }

    pub fn handle_inc(&mut self, mode: AddressingMode, cycle: &mut u8) {
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

    pub fn handle_inx(&mut self, _: AddressingMode, cycle: &mut u8) {
        let mut reg_x: u8 = self.get_register(Register::XIndex);
        if reg_x == 0xff {
            reg_x = 0;
        } else {
            reg_x += 1;
        }
        self.set_register_with_flags(Register::XIndex, reg_x);
    }

    pub fn handle_iny(&mut self, _: AddressingMode, cycle: &mut u8) {
        let mut reg_y: u8 = self.get_register(Register::YIndex);
        if reg_y == 0xff {
            reg_y = 0;
        } else {
            reg_y += 1;
        }
        self.set_register_with_flags(Register::YIndex, reg_y);
    }

    pub fn handle_dec(&mut self, mode: AddressingMode, cycle: &mut u8) {
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

    pub fn handle_dex(&mut self, _: AddressingMode, cycle: &mut u8) {
        let mut reg_x: u8 = self.get_register(Register::XIndex);
        if reg_x == 0 {
            reg_x = 0xff;
        } else {
            reg_x -= 1;
        }
        self.set_register_with_flags(Register::XIndex, reg_x);
    }

    pub fn handle_dey(&mut self, _: AddressingMode, cycle: &mut u8) {
        let mut reg_y: u8 = self.get_register(Register::YIndex);
        if reg_y == 0 {
            reg_y = 0xff;
        } else {
            reg_y -= 1;
        }
        self.set_register_with_flags(Register::YIndex, reg_y);
    }

    pub fn handle_brk(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let pc: u16 = self.get_program_counter();
        self.set_program_counter(add_mod_16bit(pc, 1)); // implied padding byte
        self.stack_push_u16(self.get_program_counter());
        self.handle_php(mode, cycle);
        let irq: u16 = self.memory_read_u16(IRQ_VECTOR);
        self.set_program_counter(irq);
    }

    pub fn handle_nop(&mut self, _: AddressingMode, cycle: &mut u8) {}

    fn cmp(&mut self, num1: u8, num2: u8) {
        let tcnum2: u8 = add_mod_8bit(!num2, 1);
        let diff: u8 = add_mod_8bit(num1, tcnum2);
        self.update_op_flags(diff);
        if num1 >= num2 {
            self.set_flag(Flag::Carry, true);
        } else {
            self.set_flag(Flag::Carry, false);
        }
    }

    pub fn handle_cmp(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        let data: u8 = self.get_data(mode);
        self.cmp(reg_acc, data);
    }

    pub fn handle_cpx(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let reg_x: u8 = self.get_register(Register::XIndex);
        let data: u8 = self.get_data(mode);
        self.cmp(reg_x, data);
    }

    pub fn handle_cpy(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let reg_y: u8 = self.get_register(Register::YIndex);
        let data: u8 = self.get_data(mode);
        self.cmp(reg_y, data);
    }

    pub fn handle_jmp(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let op: u16;
        if mode == AddressingMode::Absolute {
            op = self.get_operand(mode);
            self.set_program_counter(op);
        } else {
            let pc: u16 = self.get_program_counter();
            let indirect: u16 = self.memory_read_u16(pc);
            self.set_program_counter(add_mod_16bit(pc, 2));
            if indirect & LOWER_BYTE == LOWER_BYTE {
                let lower_byte: u8 = self.memory_read_u8(indirect);
                let upper_byte: u8 = self.memory_read_u8(indirect & UPPER_BYTE);
                op = ((upper_byte as u16) << 8) | (lower_byte as u16);
                self.set_program_counter(op);
            } else {
                op = self.memory_read_u16(indirect);
                self.set_program_counter(op);
            }
        }
    }

    pub fn handle_jsr(&mut self, _: AddressingMode, cycle: &mut u8) {
        let op: u16 = self.get_operand(AddressingMode::Absolute);
        self.stack_push_u16(self.get_program_counter() - 1);
        self.set_program_counter(op);
    }

    pub fn handle_rti(&mut self, _: AddressingMode, cycle: &mut u8) {
        let flags: u8 = self.stack_pop_u8();
        self.set_flag(Flag::Carry, flags & CF != 0);
        self.set_flag(Flag::Zero, flags & ZF != 0);
        self.set_flag(Flag::InterruptDisable, flags & IF != 0);
        self.set_flag(Flag::DecimalMode, flags & DF != 0);
        self.set_flag(Flag::Overflow, flags & VF != 0);
        self.set_flag(Flag::Negative, flags & NF != 0);
        let pc: u16 = self.stack_pop_u16();
        self.set_program_counter(pc);
    }

    pub fn handle_rts(&mut self, _: AddressingMode, cycle: &mut u8) {
        let pc: u16 = self.stack_pop_u16();
        self.set_program_counter(add_mod_16bit(pc, 1));
    }

    fn branch(&mut self, jmp: bool) {
        let op: i8 = self.get_operand(AddressingMode::Relative) as i8;
        if jmp {
            let res: i64 = (self.get_program_counter() as i64) + (op as i64);
            self.set_program_counter(res as u16);
        }
    }

    pub fn handle_bcc(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.branch(!self.get_flag(Flag::Carry));
    }

    pub fn handle_bcs(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.branch(self.get_flag(Flag::Carry));
    }

    pub fn handle_beq(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.branch(self.get_flag(Flag::Zero));
    }

    pub fn handle_bit(&mut self, mode: AddressingMode, cycle: &mut u8) {
        let reg_acc: u8 = self.get_register(Register::Accumulator);
        let data: u8 = self.get_data(mode);
        self.set_flag(Flag::Overflow, data & BIT_6 != 0);
        self.set_flag(Flag::Negative, data & BIT_7 != 0);
        self.set_flag(Flag::Zero, data & reg_acc == 0);
    }

    pub fn handle_bmi(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.branch(self.get_flag(Flag::Negative));
    }

    pub fn handle_bne(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.branch(!self.get_flag(Flag::Zero));
    }

    pub fn handle_bpl(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.branch(!self.get_flag(Flag::Negative));
    }

    pub fn handle_bvc(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.branch(!self.get_flag(Flag::Overflow));
    }

    pub fn handle_bvs(&mut self, _: AddressingMode, cycle: &mut u8) {
        self.branch(self.get_flag(Flag::Overflow));
    }
}

lazy_static! {
    pub static ref opcodes_list: Vec<OpCode> = vec![
    // CPU
        // Transfer
        OpCode::new(0xaa, "TAX".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_tax),
        OpCode::new(0xa8, "TAY".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_tay),
        OpCode::new(0xba, "TSX".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_tsx),
        OpCode::new(0x8a, "TXA".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_txa),
        OpCode::new(0x9a, "TXS".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_txs),
        OpCode::new(0x98, "TYA".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_tya),
        // Flags
        OpCode::new(0x18, "CLC".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_clc),
        OpCode::new(0xd8, "CLD".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_cld),
        OpCode::new(0x58, "CLI".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_cli),
        OpCode::new(0xb8, "CLV".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_clv),
        OpCode::new(0x38, "SEC".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_sec),
        OpCode::new(0xf8, "SED".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_sed),
        OpCode::new(0x78, "SEI".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_sei),

    // Memory
        // Load
        OpCode::new(0xa9, "LDA".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_lda),
        OpCode::new(0xa5, "LDA".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_lda),
        OpCode::new(0xb5, "LDA".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_lda),
        OpCode::new(0xad, "LDA".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_lda),
        OpCode::new(0xbd, "LDA".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX, NNES::handle_lda),
        OpCode::new(0xb9, "LDA".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY, NNES::handle_lda),
        OpCode::new(0xa1, "LDA".to_string(), 2, 6, AddressingMode::IndirectX, NNES::handle_lda),
        OpCode::new(0xb1, "LDA".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY, NNES::handle_lda),
        OpCode::new(0xa2, "LDX".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_ldx),
        OpCode::new(0xa6, "LDX".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_ldx),
        OpCode::new(0xb6, "LDX".to_string(), 2, 4, AddressingMode::ZeroPageY, NNES::handle_ldx),
        OpCode::new(0xae, "LDX".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_ldx),
        OpCode::new(0xbe, "LDX".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY, NNES::handle_ldx),
        OpCode::new(0xa0, "LDY".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_ldy),
        OpCode::new(0xa4, "LDY".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_ldy),
        OpCode::new(0xb4, "LDY".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_ldy),
        OpCode::new(0xac, "LDY".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_ldy),
        OpCode::new(0xbc, "LDY".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX, NNES::handle_ldy),
        // Store
        OpCode::new(0x85, "STA".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_sta),
        OpCode::new(0x95, "STA".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_sta),
        OpCode::new(0x8d, "STA".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_sta),
        OpCode::new(0x9d, "STA".to_string(), 3, 5, AddressingMode::AbsoluteX, NNES::handle_sta),
        OpCode::new(0x99, "STA".to_string(), 3, 5, AddressingMode::AbsoluteY, NNES::handle_sta),
        OpCode::new(0x81, "STA".to_string(), 2, 6, AddressingMode::IndirectX, NNES::handle_sta),
        OpCode::new(0x91, "STA".to_string(), 2, 6, AddressingMode::IndirectY, NNES::handle_sta),
        OpCode::new(0x86, "STX".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_stx),
        OpCode::new(0x96, "STX".to_string(), 2, 4, AddressingMode::ZeroPageY, NNES::handle_stx),
        OpCode::new(0x8e, "STX".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_stx),
        OpCode::new(0x84, "STY".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_sty),
        OpCode::new(0x94, "STY".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_sty),
        OpCode::new(0x8c, "STY".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_sty),
        // Stack
        OpCode::new(0x48, "PHA".to_string(), 1, 3, AddressingMode::Implied, NNES::handle_pha),
        OpCode::new(0x08, "PHP".to_string(), 1, 3, AddressingMode::Implied, NNES::handle_php),
        OpCode::new(0x68, "PLA".to_string(), 1, 4, AddressingMode::Implied, NNES::handle_pla),
        OpCode::new(0x28, "PLP".to_string(), 1, 4, AddressingMode::Implied, NNES::handle_plp),

    // Binary
        // AND
        OpCode::new(0x29, "AND".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_and),
        OpCode::new(0x25, "AND".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_and),
        OpCode::new(0x35, "AND".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_and),
        OpCode::new(0x2d, "AND".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_and),
        OpCode::new(0x3d, "AND".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX, NNES::handle_and),
        OpCode::new(0x39, "AND".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY, NNES::handle_and),
        OpCode::new(0x21, "AND".to_string(), 2, 6, AddressingMode::IndirectX, NNES::handle_and),
        OpCode::new(0x31, "AND".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY, NNES::handle_and),
        // OR
        OpCode::new(0x09, "ORA".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_ora),
        OpCode::new(0x05, "ORA".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_ora),
        OpCode::new(0x15, "ORA".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_ora),
        OpCode::new(0x0d, "ORA".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_ora),
        OpCode::new(0x1d, "ORA".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX, NNES::handle_ora),
        OpCode::new(0x19, "ORA".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY, NNES::handle_ora),
        OpCode::new(0x01, "ORA".to_string(), 2, 6, AddressingMode::IndirectX, NNES::handle_ora),
        OpCode::new(0x11, "ORA".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY, NNES::handle_ora),
        // XOR
        OpCode::new(0x49, "EOR".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_eor),
        OpCode::new(0x45, "EOR".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_eor),
        OpCode::new(0x55, "EOR".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_eor),
        OpCode::new(0x4d, "EOR".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_eor),
        OpCode::new(0x5d, "EOR".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX, NNES::handle_eor),
        OpCode::new(0x59, "EOR".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY, NNES::handle_eor),
        OpCode::new(0x41, "EOR".to_string(), 2, 6, AddressingMode::IndirectX, NNES::handle_eor),
        OpCode::new(0x51, "EOR".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY, NNES::handle_eor),
        // SAL
        OpCode::new(0x0a, "ASL".to_string(), 1, 2, AddressingMode::Accumulator, NNES::handle_asl),
        OpCode::new(0x06, "ASL".to_string(), 2, 5, AddressingMode::ZeroPage, NNES::handle_asl),
        OpCode::new(0x16, "ASL".to_string(), 2, 6, AddressingMode::ZeroPageX, NNES::handle_asl),
        OpCode::new(0x0e, "ASL".to_string(), 3, 6, AddressingMode::Absolute, NNES::handle_asl),
        OpCode::new(0x1e, "ASL".to_string(), 3, 7, AddressingMode::AbsoluteX, NNES::handle_asl),
        // SHR
        OpCode::new(0x4a, "LSR".to_string(), 1, 2, AddressingMode::Accumulator, NNES::handle_lsr),
        OpCode::new(0x46, "LSR".to_string(), 2, 5, AddressingMode::ZeroPage, NNES::handle_lsr),
        OpCode::new(0x56, "LSR".to_string(), 2, 6, AddressingMode::ZeroPageX, NNES::handle_lsr),
        OpCode::new(0x4e, "LSR".to_string(), 3, 6, AddressingMode::Absolute, NNES::handle_lsr),
        OpCode::new(0x5e, "LSR".to_string(), 3, 7, AddressingMode::AbsoluteX, NNES::handle_lsr),
        // RCL
        OpCode::new(0x2a, "ROL".to_string(), 1, 2, AddressingMode::Accumulator, NNES::handle_rol),
        OpCode::new(0x26, "ROL".to_string(), 2, 5, AddressingMode::ZeroPage, NNES::handle_rol),
        OpCode::new(0x36, "ROL".to_string(), 2, 6, AddressingMode::ZeroPageX, NNES::handle_rol),
        OpCode::new(0x2e, "ROL".to_string(), 3, 6, AddressingMode::Absolute, NNES::handle_rol),
        OpCode::new(0x3e, "ROL".to_string(), 3, 7, AddressingMode::AbsoluteX, NNES::handle_rol),
        // RCR
        OpCode::new(0x6a, "ROR".to_string(), 1, 2, AddressingMode::Accumulator, NNES::handle_ror),
        OpCode::new(0x66, "ROR".to_string(), 2, 5, AddressingMode::ZeroPage, NNES::handle_ror),
        OpCode::new(0x76, "ROR".to_string(), 2, 6, AddressingMode::ZeroPageX, NNES::handle_ror),
        OpCode::new(0x6e, "ROR".to_string(), 3, 6, AddressingMode::Absolute, NNES::handle_ror),
        OpCode::new(0x7e, "ROR".to_string(), 3, 7, AddressingMode::AbsoluteX, NNES::handle_ror),

    // Arithmetic
        // ADC
        OpCode::new(0x69, "ADC".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_adc),
        OpCode::new(0x65, "ADC".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_adc),
        OpCode::new(0x75, "ADC".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_adc),
        OpCode::new(0x6d, "ADC".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_adc),
        OpCode::new(0x7d, "ADC".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX, NNES::handle_adc),
        OpCode::new(0x79, "ADC".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY, NNES::handle_adc),
        OpCode::new(0x61, "ADC".to_string(), 2, 6, AddressingMode::IndirectX, NNES::handle_adc),
        OpCode::new(0x71, "ADC".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY, NNES::handle_adc),
        // SBC
        OpCode::new(0xe9, "SBC".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_sbc),
        OpCode::new(0xe5, "SBC".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_sbc),
        OpCode::new(0xf5, "SBC".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_sbc),
        OpCode::new(0xed, "SBC".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_sbc),
        OpCode::new(0xfd, "SBC".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX, NNES::handle_sbc),
        OpCode::new(0xf9, "SBC".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY, NNES::handle_sbc),
        OpCode::new(0xe1, "SBC".to_string(), 2, 6, AddressingMode::IndirectX, NNES::handle_sbc),
        OpCode::new(0xf1, "SBC".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY, NNES::handle_sbc),
        // INC
        OpCode::new(0xe6, "INC".to_string(), 2, 5, AddressingMode::ZeroPage, NNES::handle_inc),
        OpCode::new(0xf6, "INC".to_string(), 2, 6, AddressingMode::ZeroPageX, NNES::handle_inc),
        OpCode::new(0xee, "INC".to_string(), 3, 6, AddressingMode::Absolute, NNES::handle_inc),
        OpCode::new(0xfe, "INC".to_string(), 3, 7, AddressingMode::AbsoluteX, NNES::handle_inc),
        OpCode::new(0xe8, "INX".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_inx),
        OpCode::new(0xc8, "INY".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_iny),
        // DEC
        OpCode::new(0xc6, "DEC".to_string(), 2, 5, AddressingMode::ZeroPage, NNES::handle_dec),
        OpCode::new(0xd6, "DEC".to_string(), 2, 6, AddressingMode::ZeroPageX, NNES::handle_dec),
        OpCode::new(0xce, "DEC".to_string(), 3, 6, AddressingMode::Absolute, NNES::handle_dec),
        OpCode::new(0xde, "DEC".to_string(), 3, 7, AddressingMode::AbsoluteX, NNES::handle_dec),
        OpCode::new(0xca, "DEX".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_dex),
        OpCode::new(0x88, "DEY".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_dey),

    // Control flow
        // Stop
        OpCode::new(0x00, "BRK".to_string(), 1, 7, AddressingMode::Implied, NNES::handle_brk),
        OpCode::new(0xea, "NOP".to_string(), 1, 2, AddressingMode::Implied, NNES::handle_nop),
        // Compare
        OpCode::new(0xc9, "CMP".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_cmp),
        OpCode::new(0xc5, "CMP".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_cmp),
        OpCode::new(0xd5, "CMP".to_string(), 2, 4, AddressingMode::ZeroPageX, NNES::handle_cmp),
        OpCode::new(0xcd, "CMP".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_cmp),
        OpCode::new(0xdd, "CMP".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX, NNES::handle_cmp),
        OpCode::new(0xd9, "CMP".to_string(), 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY, NNES::handle_cmp),
        OpCode::new(0xc1, "CMP".to_string(), 2, 6, AddressingMode::IndirectX, NNES::handle_cmp),
        OpCode::new(0xd1, "CMP".to_string(), 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY, NNES::handle_cmp),
        OpCode::new(0xe0, "CPX".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_cpx),
        OpCode::new(0xe4, "CPX".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_cpx),
        OpCode::new(0xec, "CPX".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_cpx),
        OpCode::new(0xc0, "CPY".to_string(), 2, 2, AddressingMode::Immediate, NNES::handle_cpy),
        OpCode::new(0xc4, "CPY".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_cpy),
        OpCode::new(0xcc, "CPY".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_cpy),
        // Jump
        OpCode::new(0x4c, "JMP".to_string(), 3, 3, AddressingMode::Absolute, NNES::handle_jmp),
        OpCode::new(0x6c, "JMP".to_string(), 3, 5, AddressingMode::Indirect, NNES::handle_jmp), // 6502 bug with 0xXXFF
        OpCode::new(0x20, "JSR".to_string(), 3, 6, AddressingMode::Absolute, NNES::handle_jsr),
        OpCode::new(0x40, "RTI".to_string(), 1, 6, AddressingMode::Implied, NNES::handle_rti),
        OpCode::new(0x60, "RTS".to_string(), 1, 6, AddressingMode::Implied, NNES::handle_rts),
        // Conditionals
        OpCode::new(0x90, "BCC".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative, NNES::handle_bcc),
        OpCode::new(0xb0, "BCS".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative, NNES::handle_bcs),
        OpCode::new(0xf0, "BEQ".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative, NNES::handle_beq),
        OpCode::new(0x24, "BIT".to_string(), 2, 3, AddressingMode::ZeroPage, NNES::handle_bit),
        OpCode::new(0x2c, "BIT".to_string(), 3, 4, AddressingMode::Absolute, NNES::handle_bit),
        OpCode::new(0x30, "BMI".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative, NNES::handle_bmi),
        OpCode::new(0xd0, "BNE".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative, NNES::handle_bne),
        OpCode::new(0x10, "BPL".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative, NNES::handle_bpl),
        OpCode::new(0x50, "BVC".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative, NNES::handle_bvc),
        OpCode::new(0x70, "BVS".to_string(), 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative, NNES::handle_bvs),
    ];

    pub static ref opcodes_map: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for op in &*opcodes_list {
            map.insert(op.code, op);
        };
        map
    };
}
