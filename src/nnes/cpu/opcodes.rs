use super::CPU;

impl CPU {
    // Official addressing functions
    pub fn addr_imp(&mut self, subcycle: u8) -> bool {    // Implied
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_acc(&mut self, subcycle: u8) -> bool {    // Accumulator
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_imm(&mut self, subcycle: u8) -> bool {    // Immediate
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_zpg(&mut self, subcycle: u8) -> bool {    // Zero page
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_zpx(&mut self, subcycle: u8) -> bool {    // Zero page + X
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_zpy(&mut self, subcycle: u8) -> bool {    // Zero page + Y
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_abs(&mut self, subcycle: u8) -> bool {    // Absolute
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_abx(&mut self, subcycle: u8) -> bool {    // Absolute + X
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_aby(&mut self, subcycle: u8) -> bool {    // Absolute + Y
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_ind(&mut self, subcycle: u8) -> bool {    // Indirect
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_inx(&mut self, subcycle: u8) -> bool {    // Indirect + X
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_iny(&mut self, subcycle: u8) -> bool {    // Indirect + Y
        match subcycle {
            _ => {}
        }
        true
    }
    pub fn addr_rel(&mut self, subcycle: u8) -> bool {    // Relative
        match subcycle {
            _ => {}
        }
        true
    }

    // Illegal addressing functions

    // Official execute functions

    // Illegal execute functions
    
}

pub struct OpCode {
    code: u8,
    name: String,
    pub has_decode: bool,
    pub decode_fn: fn(&mut CPU, subcycle: u8) -> bool,
    pub execute_fn: fn(&mut CPU, subcycle: u8) -> bool,
    cross: bool,  // page crossing affects number of cycles
    branch: bool, // branching affects number of cycles
    penalty: u8,  // number of dummy cycles to insert
}

impl OpCode {
    pub fn new(
        code: u8,
        name: String,
        has_decode: bool,
        decode_fn: fn(&mut CPU, subcycle: u8) -> bool,
        execute_fn: fn(&mut CPU, subcycle: u8) -> bool,
        cross: bool,
        branch: bool,
        penalty: u8,
    ) -> Self {
        OpCode {
            code: code,
            name: name,
            has_decode: has_decode,
            decode_fn: decode_fn,
            execute_fn: execute_fn,
            cross: cross,
            branch: branch,
            penalty: penalty,
        }
    }
}

lazy_static! {
    pub static ref opcodes_list: [Option<OpCode>; 256] = [
        Some(OpCode::new(0x00, "BRK".to_string(), CPU::addr_imp, CPU::brk, false, false, 0)),
        Some(OpCode::new(0x01, "ORA".to_string(), CPU::addr_inx, CPU::ora, false, false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x05, "ORA".to_string(), CPU::addr_zpg, CPU::ora, false, false, 0)),
        Some(OpCode::new(0x06, "ASL".to_string(), CPU::addr_zpg, CPU::asl, false, false, 0)),
        None,
        Some(OpCode::new(0x08, "PHP".to_string(), CPU::addr_imp, CPU::php, false, false, 0)),
        Some(OpCode::new(0x09, "ORA".to_string(), CPU::addr_imm, CPU::ora, false, false, 0)),
        Some(OpCode::new(0x0A, "ASL".to_string(), CPU::addr_acc, CPU::asl, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x0D, "ORA".to_string(), CPU::addr_abs, CPU::ora, false, false, 0)),
        Some(OpCode::new(0x0E, "ASL".to_string(), CPU::addr_abs, CPU::asl, false, false, 0)),
        None,
        Some(OpCode::new(0x10, "BPL".to_string(), CPU::addr_rel, CPU::bpl, false, true,  0)),
        Some(OpCode::new(0x11, "ORA".to_string(), CPU::addr_iny, CPU::ora, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x15, "ORA".to_string(), CPU::addr_zpx, CPU::ora, false, false, 0)),
        Some(OpCode::new(0x16, "ASL".to_string(), CPU::addr_zpx, CPU::asl, false, false, 0)),
        None,
        Some(OpCode::new(0x18, "CLC".to_string(), CPU::addr_imp, CPU::clc, false, false, 0)),
        Some(OpCode::new(0x19, "ORA".to_string(), CPU::addr_aby, CPU::ora, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x1D, "ORA".to_string(), CPU::addr_abx, CPU::ora, true,  false, 0)),
        Some(OpCode::new(0x1E, "ASL".to_string(), CPU::addr_abx, CPU::asl, false, false, 0)),
        None,
        Some(OpCode::new(0x20, "JSR".to_string(), CPU::addr_abs, CPU::jsr, false, false, 0)),
        Some(OpCode::new(0x21, "AND".to_string(), CPU::addr_inx, CPU::and, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x24, "BIT".to_string(), CPU::addr_zpg, CPU::bit, false, false, 0)),
        Some(OpCode::new(0x25, "AND".to_string(), CPU::addr_zpg, CPU::and, false, false, 0)),
        Some(OpCode::new(0x26, "ROL".to_string(), CPU::addr_zpg, CPU::rol, false, false, 0)),
        None,
        Some(OpCode::new(0x28, "PLP".to_string(), CPU::addr_imp, CPU::plp, false, false, 0)),
        Some(OpCode::new(0x29, "AND".to_string(), CPU::addr_imm, CPU::and, false, false, 0)),
        Some(OpCode::new(0x2A, "ROL".to_string(), CPU::addr_acc, CPU::rol, false, false, 0)),
        None,
        Some(OpCode::new(0x2C, "BIT".to_string(), CPU::addr_abs, CPU::bit, false, false, 0)),
        Some(OpCode::new(0x2D, "AND".to_string(), CPU::addr_abs, CPU::and, false, false, 0)),
        Some(OpCode::new(0x2E, "ROL".to_string(), CPU::addr_abs, CPU::rol, false, false, 0)),
        None,
        Some(OpCode::new(0x30, "BMI".to_string(), CPU::addr_rel, CPU::bmi, false, true,  0)),
        Some(OpCode::new(0x31, "AND".to_string(), CPU::addr_iny, CPU::and, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x35, "AND".to_string(), CPU::addr_zpx, CPU::and, false, false, 0)),
        Some(OpCode::new(0x36, "ROL".to_string(), CPU::addr_zpx, CPU::rol, false, false, 0)),
        None,
        Some(OpCode::new(0x38, "SEC".to_string(), CPU::addr_imp, CPU::sec, false, false, 0)),
        Some(OpCode::new(0x39, "AND".to_string(), CPU::addr_aby, CPU::and, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x3D, "AND".to_string(), CPU::addr_abx, CPU::and, true,  false, 0)),
        Some(OpCode::new(0x3E, "ROL".to_string(), CPU::addr_abx, CPU::rol, false, false, 0)),
        None,
        Some(OpCode::new(0x40, "RTI".to_string(), CPU::addr_imp, CPU::rti, false, false, 0)),
        Some(OpCode::new(0x41, "EOR".to_string(), CPU::addr_inx, CPU::eor, false, false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x45, "EOR".to_string(), CPU::addr_zpg, CPU::eor, false, false, 0)),
        Some(OpCode::new(0x46, "LSR".to_string(), CPU::addr_zpg, CPU::lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x48, "PHA".to_string(), CPU::addr_imp, CPU::pha, false, false, 0)),
        Some(OpCode::new(0x49, "EOR".to_string(), CPU::addr_imm, CPU::eor, false, false, 0)),
        Some(OpCode::new(0x4A, "LSR".to_string(), CPU::addr_acc, CPU::lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x4C, "JMP".to_string(), CPU::addr_abs, CPU::jmp, false, false, 0)),
        Some(OpCode::new(0x4D, "EOR".to_string(), CPU::addr_abs, CPU::eor, false, false, 0)),
        Some(OpCode::new(0x4E, "LSR".to_string(), CPU::addr_abs, CPU::lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x50, "BVC".to_string(), CPU::addr_rel, CPU::bvc, false, true,  0)),
        Some(OpCode::new(0x51, "EOR".to_string(), CPU::addr_iny, CPU::eor, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x55, "EOR".to_string(), CPU::addr_zpx, CPU::eor, false, false, 0)),
        Some(OpCode::new(0x56, "LSR".to_string(), CPU::addr_zpx, CPU::lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x58, "CLI".to_string(), CPU::addr_imp, CPU::cli, false, false, 0)),
        Some(OpCode::new(0x59, "EOR".to_string(), CPU::addr_aby, CPU::eor, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x5D, "EOR".to_string(), CPU::addr_abx, CPU::eor, true,  false, 0)),
        Some(OpCode::new(0x5E, "LSR".to_string(), CPU::addr_abx, CPU::lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x60, "RTS".to_string(), CPU::addr_imp, CPU::rts, false, false, 0)),
        Some(OpCode::new(0x61, "ADC".to_string(), CPU::addr_inx, CPU::adc, false, false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x65, "ADC".to_string(), CPU::addr_zpg, CPU::adc, false, false, 0)),
        Some(OpCode::new(0x66, "ROR".to_string(), CPU::addr_zpg, CPU::ror, false, false, 0)),
        None,
        Some(OpCode::new(0x68, "PLA".to_string(), CPU::addr_imp, CPU::pla, false, false, 0)),
        Some(OpCode::new(0x69, "ADC".to_string(), CPU::addr_imm, CPU::adc, false, false, 0)),
        Some(OpCode::new(0x6A, "ROR".to_string(), CPU::addr_acc, CPU::ror, false, false, 0)),
        None,
        Some(OpCode::new(0x6C, "JMP".to_string(), CPU::addr_ind, CPU::jmp, false, false, 0)), // indirect bug
        Some(OpCode::new(0x6D, "ADC".to_string(), CPU::addr_abs, CPU::adc, false, false, 0)),
        Some(OpCode::new(0x6E, "ROR".to_string(), CPU::addr_abs, CPU::ror, false, false, 0)),
        None,
        Some(OpCode::new(0x70, "BVS".to_string(), CPU::addr_rel, CPU::bvs, false, true,  0)),
        Some(OpCode::new(0x71, "ADC".to_string(), CPU::addr_iny, CPU::adc, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x75, "ADC".to_string(), CPU::addr_zpx, CPU::adc, false, false, 0)),
        Some(OpCode::new(0x76, "ROR".to_string(), CPU::addr_zpx, CPU::ror, false, false, 0)),
        None,
        Some(OpCode::new(0x78, "SEI".to_string(), CPU::addr_imp, CPU::sei, false, false, 0)),
        Some(OpCode::new(0x79, "ADC".to_string(), CPU::addr_aby, CPU::adc, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x7D, "ADC".to_string(), CPU::addr_abx, CPU::adc, true,  false, 0)),
        Some(OpCode::new(0x7E, "ROR".to_string(), CPU::addr_abx, CPU::ror, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x81, "STA".to_string(), CPU::addr_inx, CPU::sta, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x84, "STY".to_string(), CPU::addr_zpg, CPU::sty, false, false, 0)),
        Some(OpCode::new(0x85, "STA".to_string(), CPU::addr_zpg, CPU::sta, false, false, 0)),
        Some(OpCode::new(0x86, "STX".to_string(), CPU::addr_zpg, CPU::stx, false, false, 0)),
        None,
        Some(OpCode::new(0x88, "DEY".to_string(), CPU::addr_imp, CPU::dey, false, false, 0)),
        None,
        Some(OpCode::new(0x8A, "TXA".to_string(), CPU::addr_imp, CPU::txa, false, false, 0)),
        None,
        Some(OpCode::new(0x8C, "STY".to_string(), CPU::addr_abs, CPU::sty, false, false, 0)),
        Some(OpCode::new(0x8D, "STA".to_string(), CPU::addr_abs, CPU::sta, false, false, 0)),
        Some(OpCode::new(0x8E, "STX".to_string(), CPU::addr_abs, CPU::stx, false, false, 0)),
        None,
        Some(OpCode::new(0x90, "BCC".to_string(), CPU::addr_rel, CPU::bcc, false, true,  0)),
        Some(OpCode::new(0x91, "STA".to_string(), CPU::addr_iny, CPU::sta, false, false, 1)),
        None,
        None,
        Some(OpCode::new(0x94, "STY".to_string(), CPU::addr_zpx, CPU::sty, false, false, 0)),
        Some(OpCode::new(0x95, "STA".to_string(), CPU::addr_zpx, CPU::sta, false, false, 0)),
        Some(OpCode::new(0x96, "STX".to_string(), CPU::addr_zpy, CPU::stx, false, false, 0)),
        None,
        Some(OpCode::new(0x98, "TYA".to_string(), CPU::addr_imp, CPU::tya, false, false, 0)),
        Some(OpCode::new(0x99, "STA".to_string(), CPU::addr_aby, CPU::sta, false, false, 1)),
        Some(OpCode::new(0x9A, "TXS".to_string(), CPU::addr_imp, CPU::txs, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x9D, "STA".to_string(), CPU::addr_abx, CPU::sta, false, false, 1)),
        None,
        None,
        Some(OpCode::new(0xA0, "LDY".to_string(), CPU::addr_imm, CPU::ldy, false, false, 0)),
        Some(OpCode::new(0xA1, "LDA".to_string(), CPU::addr_inx, CPU::lda, false, false, 0)),
        Some(OpCode::new(0xA2, "LDX".to_string(), CPU::addr_imm, CPU::ldx, false, false, 0)),
        None,
        Some(OpCode::new(0xA4, "LDY".to_string(), CPU::addr_zpg, CPU::ldy, false, false, 0)),
        Some(OpCode::new(0xA5, "LDA".to_string(), CPU::addr_zpg, CPU::lda, false, false, 0)),
        Some(OpCode::new(0xA6, "LDX".to_string(), CPU::addr_zpg, CPU::ldx, false, false, 0)),
        None,
        Some(OpCode::new(0xA8, "TAY".to_string(), CPU::addr_imp, CPU::tay, false, false, 0)),
        Some(OpCode::new(0xA9, "LDA".to_string(), CPU::addr_imm, CPU::lda, false, false, 0)),
        Some(OpCode::new(0xAA, "TAX".to_string(), CPU::addr_imp, CPU::tax, false, false, 0)),
        None,
        Some(OpCode::new(0xAC, "LDY".to_string(), CPU::addr_abs, CPU::ldy, false, false, 0)),
        Some(OpCode::new(0xAD, "LDA".to_string(), CPU::addr_abs, CPU::lda, false, false, 0)),
        Some(OpCode::new(0xAE, "LDX".to_string(), CPU::addr_abs, CPU::ldx, false, false, 0)),
        None,
        Some(OpCode::new(0xB0, "BCS".to_string(), CPU::addr_rel, CPU::bcs, false, true,  0)),
        Some(OpCode::new(0xB1, "LDA".to_string(), CPU::addr_iny, CPU::lda, true,  false, 0)),
        None,
        None,
        Some(OpCode::new(0xB4, "LDY".to_string(), CPU::addr_zpx, CPU::ldy, false, false, 0)),
        Some(OpCode::new(0xB5, "LDA".to_string(), CPU::addr_zpx, CPU::lda, false, false, 0)),
        Some(OpCode::new(0xB6, "LDX".to_string(), CPU::addr_zpy, CPU::ldx, false, false, 0)),
        None,
        Some(OpCode::new(0xB8, "CLV".to_string(), CPU::addr_imp, CPU::clv, false, false, 0)),
        Some(OpCode::new(0xB9, "LDA".to_string(), CPU::addr_aby, CPU::lda, true,  false, 0)),
        Some(OpCode::new(0xBA, "TSX".to_string(), CPU::addr_imp, CPU::tsx, false, false, 0)),
        None,
        Some(OpCode::new(0xBC, "LDY".to_string(), CPU::addr_abx, CPU::ldy, true,  false, 0)),
        Some(OpCode::new(0xBD, "LDA".to_string(), CPU::addr_abx, CPU::lda, true,  false, 0)),
        Some(OpCode::new(0xBE, "LDX".to_string(), CPU::addr_aby, CPU::ldx, true,  false, 0)),
        None,
        Some(OpCode::new(0xC0, "CPY".to_string(), CPU::addr_imm, CPU::cpy, false, false, 0)),
        Some(OpCode::new(0xC1, "CMP".to_string(), CPU::addr_inx, CPU::cmp, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0xC4, "CPY".to_string(), CPU::addr_zpg, CPU::cpy, false, false, 0)),
        Some(OpCode::new(0xC5, "CMP".to_string(), CPU::addr_zpg, CPU::cmp, false, false, 0)),
        Some(OpCode::new(0xC6, "DEC".to_string(), CPU::addr_zpg, CPU::dec, false, false, 0)),
        None,
        Some(OpCode::new(0xC8, "INY".to_string(), CPU::addr_imp, CPU::iny, false, false, 0)),
        Some(OpCode::new(0xC9, "CMP".to_string(), CPU::addr_imm, CPU::cmp, false, false, 0)),
        Some(OpCode::new(0xCA, "DEX".to_string(), CPU::addr_imp, CPU::dex, false, false, 0)),
        None,
        Some(OpCode::new(0xCC, "CPY".to_string(), CPU::addr_abs, CPU::cpy, false, false, 0)),
        Some(OpCode::new(0xCD, "CMP".to_string(), CPU::addr_abs, CPU::cmp, false, false, 0)),
        Some(OpCode::new(0xCE, "DEC".to_string(), CPU::addr_abs, CPU::dec, false, false, 0)),
        None,
        Some(OpCode::new(0xD0, "BNE".to_string(), CPU::addr_rel, CPU::bne, false, true,  0)),
        Some(OpCode::new(0xD1, "CMP".to_string(), CPU::addr_iny, CPU::cmp, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0xD5, "CMP".to_string(), CPU::addr_zpx, CPU::cmp, false, false, 0)),
        Some(OpCode::new(0xD6, "DEC".to_string(), CPU::addr_zpx, CPU::dec, false, false, 0)),
        None,
        Some(OpCode::new(0xD8, "CLD".to_string(), CPU::addr_imp, CPU::cld, false, false, 0)),
        Some(OpCode::new(0xD9, "CMP".to_string(), CPU::addr_aby, CPU::cmp, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0xDD, "CMP".to_string(), CPU::addr_abx, CPU::cmp, true,  false, 0)),
        Some(OpCode::new(0xDE, "DEC".to_string(), CPU::addr_abx, CPU::dec, false, false, 0)),
        None,
        Some(OpCode::new(0xE0, "CPX".to_string(), CPU::addr_imm, CPU::cpx, false, false, 0)),
        Some(OpCode::new(0xE1, "SBC".to_string(), CPU::addr_inx, CPU::sbc, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0xE4, "CPX".to_string(), CPU::addr_zpg, CPU::cpx, false, false, 0)),
        Some(OpCode::new(0xE5, "SBC".to_string(), CPU::addr_zpg, CPU::sbc, false, false, 0)),
        Some(OpCode::new(0xE6, "INC".to_string(), CPU::addr_zpg, CPU::inc, false, false, 0)),
        None,
        Some(OpCode::new(0xE8, "INX".to_string(), CPU::addr_imp, CPU::inx, false, false, 0)),
        Some(OpCode::new(0xE9, "SBC".to_string(), CPU::addr_imm, CPU::sbc, false, false, 0)),
        Some(OpCode::new(0xEA, "NOP".to_string(), CPU::addr_imp, CPU::nop, false, false, 0)),
        None,
        Some(OpCode::new(0xEC, "CPX".to_string(), CPU::addr_abs, CPU::cpx, false, false, 0)),
        Some(OpCode::new(0xED, "SBC".to_string(), CPU::addr_abs, CPU::sbc, false, false, 0)),
        Some(OpCode::new(0xEE, "INC".to_string(), CPU::addr_abs, CPU::inc, false, false, 0)),
        None,
        Some(OpCode::new(0xF0, "BEQ".to_string(), CPU::addr_rel, CPU::beq, false, true,  0)),
        Some(OpCode::new(0xF1, "SBC".to_string(), CPU::addr_iny, CPU::sbc, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0xF5, "SBC".to_string(), CPU::addr_zpx, CPU::sbc, false, false, 0)),
        Some(OpCode::new(0xF6, "INC".to_string(), CPU::addr_zpx, CPU::inc, false, false, 0)),
        None,
        Some(OpCode::new(0xF8, "SED".to_string(), CPU::addr_imp, CPU::sed, false, false, 0)),
        Some(OpCode::new(0xF9, "SBC".to_string(), CPU::addr_aby, CPU::sbc, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0xFD, "SBC".to_string(), CPU::addr_abx, CPU::sbc, true,  false, 0)),
        Some(OpCode::new(0xFE, "INC".to_string(), CPU::addr_abx, CPU::inc, false, false, 0)),
        None,
    ];
}
