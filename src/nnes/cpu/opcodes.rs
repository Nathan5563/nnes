use super::CPU;

impl CPU {
    // Official addressing functions
    pub fn addr_imp(&mut self, subcycle: u8) -> bool {    // Implied
        true
    }
    pub fn addr_acc(&mut self, subcycle: u8) -> bool {    // Accumulator
        true
    }
    pub fn addr_imm(&mut self, subcycle: u8) -> bool {    // Immediate
        true
    }
    pub fn addr_zpg(&mut self, subcycle: u8) -> bool {    // Zero page
        true
    }
    pub fn addr_zpx(&mut self, subcycle: u8) -> bool {    // Zero page + X
        true
    }
    pub fn addr_zpy(&mut self, subcycle: u8) -> bool {    // Zero page + Y
        true
    }
    pub fn addr_abs(&mut self, subcycle: u8) -> bool {    // Absolute
        true
    }
    pub fn addr_abx(&mut self, subcycle: u8) -> bool {    // Absolute + X
        true
    }
    pub fn addr_aby(&mut self, subcycle: u8) -> bool {    // Absolute + Y
        true
    }
    pub fn addr_ind(&mut self, subcycle: u8) -> bool {    // Indirect
        true
    }
    pub fn addr_inx(&mut self, subcycle: u8) -> bool {    // Indirect + X
        true
    }
    pub fn addr_iny(&mut self, subcycle: u8) -> bool {    // Indirect + Y
        true
    }
    pub fn addr_rel(&mut self, subcycle: u8) -> bool {    // Relative
        true
    }
    // Illegal addressing functions

    // Official execute functions

    // Illegal execute functions
    
}

pub struct OpCode {
    code: u8,
    name: String,
    addr_fn: fn(&mut CPU, subcycle: u8) -> bool,
    exec_fn: fn(&mut CPU, subcycle: u8) -> bool,
    cross: bool,  // page crossing affects number of cycles
    branch: bool, // branching affects number of cycles
    penalty: u8,  // number of dummy cycles to insert
}

impl OpCode {
    pub fn new(
        code: u8,
        name: String,
        addr_fn: fn(&mut CPU, subcycle: u8) -> bool,
        exec_fn: fn(&mut CPU, subcycle: u8) -> bool,
        cross: bool,
        branch: bool,
        penalty: u8,
    ) -> Self {
        OpCode {
            code: code,
            name: name,
            addr_fn: addr_fn,
            exec_fn: exec_fn,
            cross: cross,
            branch: branch,
            penalty: penalty,
        }
    }
}

lazy_static! {
    pub static ref opcodes_list: [Option<OpCode>; 256] = [
        Some(OpCode::new(0x00, "BRK".to_string(), CPU::addr_imp, CPU::exec_brk, false, false, 0)),
        Some(OpCode::new(0x01, "ORA".to_string(), CPU::addr_inx, CPU::exec_ora, false, false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x05, "ORA".to_string(), CPU::addr_zpg, CPU::exec_ora, false, false, 0)),
        Some(OpCode::new(0x06, "ASL".to_string(), CPU::addr_zpg, CPU::exec_asl, false, false, 0)),
        None,
        Some(OpCode::new(0x08, "PHP".to_string(), CPU::addr_imp, CPU::exec_php, false, false, 0)),
        Some(OpCode::new(0x09, "ORA".to_string(), CPU::addr_imm, CPU::exec_ora, false, false, 0)),
        Some(OpCode::new(0x0A, "ASL".to_string(), CPU::addr_acc, CPU::exec_asl, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x0D, "ORA".to_string(), CPU::addr_abs, CPU::exec_ora, false, false, 0)),
        Some(OpCode::new(0x0E, "ASL".to_string(), CPU::addr_abs, CPU::exec_asl, false, false, 0)),
        None,
        Some(OpCode::new(0x10, "BPL".to_string(), CPU::addr_rel, CPU::exec_bpl, false, true,  0)),
        Some(OpCode::new(0x11, "ORA".to_string(), CPU::addr_iny, CPU::exec_ora, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x15, "ORA".to_string(), CPU::addr_zpx, CPU::exec_ora, false, false, 0)),
        Some(OpCode::new(0x16, "ASL".to_string(), CPU::addr_zpx, CPU::exec_asl, false, false, 0)),
        None,
        Some(OpCode::new(0x18, "CLC".to_string(), CPU::addr_imp, CPU::exec_clc, false, false, 0)),
        Some(OpCode::new(0x19, "ORA".to_string(), CPU::addr_aby, CPU::exec_ora, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x1D, "ORA".to_string(), CPU::addr_abx, CPU::exec_ora, true,  false, 0)),
        Some(OpCode::new(0x1E, "ASL".to_string(), CPU::addr_abx, CPU::exec_asl, false, false, 0)),
        None,
        Some(OpCode::new(0x20, "JSR".to_string(), CPU::addr_abs, CPU::exec_jsr, false, false, 0)),
        Some(OpCode::new(0x21, "AND".to_string(), CPU::addr_inx, CPU::exec_and, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x24, "BIT".to_string(), CPU::addr_zpg, CPU::exec_bit, false, false, 0)),
        Some(OpCode::new(0x25, "AND".to_string(), CPU::addr_zpg, CPU::exec_and, false, false, 0)),
        Some(OpCode::new(0x26, "ROL".to_string(), CPU::addr_zpg, CPU::exec_rol, false, false, 0)),
        None,
        Some(OpCode::new(0x28, "PLP".to_string(), CPU::addr_imp, CPU::exec_plp, false, false, 0)),
        Some(OpCode::new(0x29, "AND".to_string(), CPU::addr_imm, CPU::exec_and, false, false, 0)),
        Some(OpCode::new(0x2A, "ROL".to_string(), CPU::addr_acc, CPU::exec_rol, false, false, 0)),
        None,
        Some(OpCode::new(0x2C, "BIT".to_string(), CPU::addr_abs, CPU::exec_bit, false, false, 0)),
        Some(OpCode::new(0x2D, "AND".to_string(), CPU::addr_abs, CPU::exec_and, false, false, 0)),
        Some(OpCode::new(0x2E, "ROL".to_string(), CPU::addr_abs, CPU::exec_rol, false, false, 0)),
        None,
        Some(OpCode::new(0x30, "BMI".to_string(), CPU::addr_rel, CPU::exec_bmi, false, true,  0)),
        Some(OpCode::new(0x31, "AND".to_string(), CPU::addr_iny, CPU::exec_and, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x35, "AND".to_string(), CPU::addr_zpx, CPU::exec_and, false, false, 0)),
        Some(OpCode::new(0x36, "ROL".to_string(), CPU::addr_zpx, CPU::exec_rol, false, false, 0)),
        None,
        Some(OpCode::new(0x38, "SEC".to_string(), CPU::addr_imp, CPU::exec_sec, false, false, 0)),
        Some(OpCode::new(0x39, "AND".to_string(), CPU::addr_aby, CPU::exec_and, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x3D, "AND".to_string(), CPU::addr_abx, CPU::exec_and, true,  false, 0)),
        Some(OpCode::new(0x3E, "ROL".to_string(), CPU::addr_abx, CPU::exec_rol, false, false, 0)),
        None,
        Some(OpCode::new(0x40, "RTI".to_string(), CPU::addr_imp, CPU::exec_rti, false, false, 0)),
        Some(OpCode::new(0x41, "EOR".to_string(), CPU::addr_inx, CPU::exec_eor, false, false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x45, "EOR".to_string(), CPU::addr_zpg, CPU::exec_eor, false, false, 0)),
        Some(OpCode::new(0x46, "LSR".to_string(), CPU::addr_zpg, CPU::exec_lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x48, "PHA".to_string(), CPU::addr_imp, CPU::exec_pha, false, false, 0)),
        Some(OpCode::new(0x49, "EOR".to_string(), CPU::addr_imm, CPU::exec_eor, false, false, 0)),
        Some(OpCode::new(0x4A, "LSR".to_string(), CPU::addr_acc, CPU::exec_lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x4C, "JMP".to_string(), CPU::addr_abs, CPU::exec_jmp, false, false, 0)),
        Some(OpCode::new(0x4D, "EOR".to_string(), CPU::addr_abs, CPU::exec_eor, false, false, 0)),
        Some(OpCode::new(0x4E, "LSR".to_string(), CPU::addr_abs, CPU::exec_lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x50, "BVC".to_string(), CPU::addr_rel, CPU::exec_bvc, false, true,  0)),
        Some(OpCode::new(0x51, "EOR".to_string(), CPU::addr_iny, CPU::exec_eor, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x55, "EOR".to_string(), CPU::addr_zpx, CPU::exec_eor, false, false, 0)),
        Some(OpCode::new(0x56, "LSR".to_string(), CPU::addr_zpx, CPU::exec_lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x58, "CLI".to_string(), CPU::addr_imp, CPU::exec_cli, false, false, 0)),
        Some(OpCode::new(0x59, "EOR".to_string(), CPU::addr_aby, CPU::exec_eor, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x5D, "EOR".to_string(), CPU::addr_abx, CPU::exec_eor, true,  false, 0)),
        Some(OpCode::new(0x5E, "LSR".to_string(), CPU::addr_abx, CPU::exec_lsr, false, false, 0)),
        None,
        Some(OpCode::new(0x60, "RTS".to_string(), CPU::addr_imp, CPU::exec_rts, false, false, 0)),
        Some(OpCode::new(0x61, "ADC".to_string(), CPU::addr_inx, CPU::exec_adc, false, false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x65, "ADC".to_string(), CPU::addr_zpg, CPU::exec_adc, false, false, 0)),
        Some(OpCode::new(0x66, "ROR".to_string(), CPU::addr_zpg, CPU::exec_ror, false, false, 0)),
        None,
        Some(OpCode::new(0x68, "PLA".to_string(), CPU::addr_imp, CPU::exec_pla, false, false, 0)),
        Some(OpCode::new(0x69, "ADC".to_string(), CPU::addr_imm, CPU::exec_adc, false, false, 0)),
        Some(OpCode::new(0x6A, "ROR".to_string(), CPU::addr_acc, CPU::exec_ror, false, false, 0)),
        None,
        Some(OpCode::new(0x6C, "JMP".to_string(), CPU::addr_ind, CPU::exec_jmp, false, false, 0)), // indirect bug
        Some(OpCode::new(0x6D, "ADC".to_string(), CPU::addr_abs, CPU::exec_adc, false, false, 0)),
        Some(OpCode::new(0x6E, "ROR".to_string(), CPU::addr_abs, CPU::exec_ror, false, false, 0)),
        None,
        Some(OpCode::new(0x70, "BVS".to_string(), CPU::addr_rel, CPU::exec_bvs, false, true,  0)),
        Some(OpCode::new(0x71, "ADC".to_string(), CPU::addr_iny, CPU::exec_adc, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x75, "ADC".to_string(), CPU::addr_zpx, CPU::exec_adc, false, false, 0)),
        Some(OpCode::new(0x76, "ROR".to_string(), CPU::addr_zpx, CPU::exec_ror, false, false, 0)),
        None,
        Some(OpCode::new(0x78, "SEI".to_string(), CPU::addr_imp, CPU::exec_sei, false, false, 0)),
        Some(OpCode::new(0x79, "ADC".to_string(), CPU::addr_aby, CPU::exec_adc, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0x7D, "ADC".to_string(), CPU::addr_abx, CPU::exec_adc, true,  false, 0)),
        Some(OpCode::new(0x7E, "ROR".to_string(), CPU::addr_abx, CPU::exec_ror, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x81, "STA".to_string(), CPU::addr_inx, CPU::exec_sta, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x84, "STY".to_string(), CPU::addr_zpg, CPU::exec_sty, false, false, 0)),
        Some(OpCode::new(0x85, "STA".to_string(), CPU::addr_zpg, CPU::exec_sta, false, false, 0)),
        Some(OpCode::new(0x86, "STX".to_string(), CPU::addr_zpg, CPU::exec_stx, false, false, 0)),
        None,
        Some(OpCode::new(0x88, "DEY".to_string(), CPU::addr_imp, CPU::exec_dey, false, false, 0)),
        None,
        Some(OpCode::new(0x8A, "TXA".to_string(), CPU::addr_imp, CPU::exec_txa, false, false, 0)),
        None,
        Some(OpCode::new(0x8C, "STY".to_string(), CPU::addr_abs, CPU::exec_sty, false, false, 0)),
        Some(OpCode::new(0x8D, "STA".to_string(), CPU::addr_abs, CPU::exec_sta, false, false, 0)),
        Some(OpCode::new(0x8E, "STX".to_string(), CPU::addr_abs, CPU::exec_stx, false, false, 0)),
        None,
        Some(OpCode::new(0x90, "BCC".to_string(), CPU::addr_rel, CPU::exec_bcc, false, true,  0)),
        Some(OpCode::new(0x91, "STA".to_string(), CPU::addr_iny, CPU::exec_sta, false, false, 1)),
        None,
        None,
        Some(OpCode::new(0x94, "STY".to_string(), CPU::addr_zpx, CPU::exec_sty, false, false, 0)),
        Some(OpCode::new(0x95, "STA".to_string(), CPU::addr_zpx, CPU::exec_sta, false, false, 0)),
        Some(OpCode::new(0x96, "STX".to_string(), CPU::addr_zpy, CPU::exec_stx, false, false, 0)),
        None,
        Some(OpCode::new(0x98, "TYA".to_string(), CPU::addr_imp, CPU::exec_tya, false, false, 0)),
        Some(OpCode::new(0x99, "STA".to_string(), CPU::addr_aby, CPU::exec_sta, false, false, 1)),
        Some(OpCode::new(0x9A, "TXS".to_string(), CPU::addr_imp, CPU::exec_txs, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0x9D, "STA".to_string(), CPU::addr_abx, CPU::exec_sta, false, false, 1)),
        None,
        None,
        Some(OpCode::new(0xA0, "LDY".to_string(), CPU::addr_imm, CPU::exec_ldy, false, false, 0)),
        Some(OpCode::new(0xA1, "LDA".to_string(), CPU::addr_inx, CPU::exec_lda, false, false, 0)),
        Some(OpCode::new(0xA2, "LDX".to_string(), CPU::addr_imm, CPU::exec_ldx, false, false, 0)),
        None,
        Some(OpCode::new(0xA4, "LDY".to_string(), CPU::addr_zpg, CPU::exec_ldy, false, false, 0)),
        Some(OpCode::new(0xA5, "LDA".to_string(), CPU::addr_zpg, CPU::exec_lda, false, false, 0)),
        Some(OpCode::new(0xA6, "LDX".to_string(), CPU::addr_zpg, CPU::exec_ldx, false, false, 0)),
        None,
        Some(OpCode::new(0xA8, "TAY".to_string(), CPU::addr_imp, CPU::exec_tay, false, false, 0)),
        Some(OpCode::new(0xA9, "LDA".to_string(), CPU::addr_imm, CPU::exec_lda, false, false, 0)),
        Some(OpCode::new(0xAA, "TAX".to_string(), CPU::addr_imp, CPU::exec_tax, false, false, 0)),
        None,
        Some(OpCode::new(0xAC, "LDY".to_string(), CPU::addr_abs, CPU::exec_ldy, false, false, 0)),
        Some(OpCode::new(0xAD, "LDA".to_string(), CPU::addr_abs, CPU::exec_lda, false, false, 0)),
        Some(OpCode::new(0xAE, "LDX".to_string(), CPU::addr_abs, CPU::exec_ldx, false, false, 0)),
        None,
        Some(OpCode::new(0xB0, "BCS".to_string(), CPU::addr_rel, CPU::exec_bcs, false, true,  0)),
        Some(OpCode::new(0xB1, "LDA".to_string(), CPU::addr_iny, CPU::exec_lda, true,  false, 0)),
        None,
        None,
        Some(OpCode::new(0xB4, "LDY".to_string(), CPU::addr_zpx, CPU::exec_ldy, false, false, 0)),
        Some(OpCode::new(0xB5, "LDA".to_string(), CPU::addr_zpx, CPU::exec_lda, false, false, 0)),
        Some(OpCode::new(0xB6, "LDX".to_string(), CPU::addr_zpy, CPU::exec_ldx, false, false, 0)),
        None,
        Some(OpCode::new(0xB8, "CLV".to_string(), CPU::addr_imp, CPU::exec_clv, false, false, 0)),
        Some(OpCode::new(0xB9, "LDA".to_string(), CPU::addr_aby, CPU::exec_lda, true,  false, 0)),
        Some(OpCode::new(0xBA, "TSX".to_string(), CPU::addr_imp, CPU::exec_tsx, false, false, 0)),
        None,
        Some(OpCode::new(0xBC, "LDY".to_string(), CPU::addr_abx, CPU::exec_ldy, true,  false, 0)),
        Some(OpCode::new(0xBD, "LDA".to_string(), CPU::addr_abx, CPU::exec_lda, true,  false, 0)),
        Some(OpCode::new(0xBE, "LDX".to_string(), CPU::addr_aby, CPU::exec_ldx, true,  false, 0)),
        None,
        Some(OpCode::new(0xC0, "CPY".to_string(), CPU::addr_imm, CPU::exec_cpy, false, false, 0)),
        Some(OpCode::new(0xC1, "CMP".to_string(), CPU::addr_inx, CPU::exec_cmp, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0xC4, "CPY".to_string(), CPU::addr_zpg, CPU::exec_cpy, false, false, 0)),
        Some(OpCode::new(0xC5, "CMP".to_string(), CPU::addr_zpg, CPU::exec_cmp, false, false, 0)),
        Some(OpCode::new(0xC6, "DEC".to_string(), CPU::addr_zpg, CPU::exec_dec, false, false, 0)),
        None,
        Some(OpCode::new(0xC8, "INY".to_string(), CPU::addr_imp, CPU::exec_iny, false, false, 0)),
        Some(OpCode::new(0xC9, "CMP".to_string(), CPU::addr_imm, CPU::exec_cmp, false, false, 0)),
        Some(OpCode::new(0xCA, "DEX".to_string(), CPU::addr_imp, CPU::exec_dex, false, false, 0)),
        None,
        Some(OpCode::new(0xCC, "CPY".to_string(), CPU::addr_abs, CPU::exec_cpy, false, false, 0)),
        Some(OpCode::new(0xCD, "CMP".to_string(), CPU::addr_abs, CPU::exec_cmp, false, false, 0)),
        Some(OpCode::new(0xCE, "DEC".to_string(), CPU::addr_abs, CPU::exec_dec, false, false, 0)),
        None,
        Some(OpCode::new(0xD0, "BNE".to_string(), CPU::addr_rel, CPU::exec_bne, false, true,  0)),
        Some(OpCode::new(0xD1, "CMP".to_string(), CPU::addr_iny, CPU::exec_cmp, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0xD5, "CMP".to_string(), CPU::addr_zpx, CPU::exec_cmp, false, false, 0)),
        Some(OpCode::new(0xD6, "DEC".to_string(), CPU::addr_zpx, CPU::exec_dec, false, false, 0)),
        None,
        Some(OpCode::new(0xD8, "CLD".to_string(), CPU::addr_imp, CPU::exec_cld, false, false, 0)),
        Some(OpCode::new(0xD9, "CMP".to_string(), CPU::addr_aby, CPU::exec_cmp, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0xDD, "CMP".to_string(), CPU::addr_abx, CPU::exec_cmp, true,  false, 0)),
        Some(OpCode::new(0xDE, "DEC".to_string(), CPU::addr_abx, CPU::exec_dec, false, false, 0)),
        None,
        Some(OpCode::new(0xE0, "CPX".to_string(), CPU::addr_imm, CPU::exec_cpx, false, false, 0)),
        Some(OpCode::new(0xE1, "SBC".to_string(), CPU::addr_inx, CPU::exec_sbc, false, false, 0)),
        None,
        None,
        Some(OpCode::new(0xE4, "CPX".to_string(), CPU::addr_zpg, CPU::exec_cpx, false, false, 0)),
        Some(OpCode::new(0xE5, "SBC".to_string(), CPU::addr_zpg, CPU::exec_sbc, false, false, 0)),
        Some(OpCode::new(0xE6, "INC".to_string(), CPU::addr_zpg, CPU::exec_inc, false, false, 0)),
        None,
        Some(OpCode::new(0xE8, "INX".to_string(), CPU::addr_imp, CPU::exec_inx, false, false, 0)),
        Some(OpCode::new(0xE9, "SBC".to_string(), CPU::addr_imm, CPU::exec_sbc, false, false, 0)),
        Some(OpCode::new(0xEA, "NOP".to_string(), CPU::addr_imp, CPU::exec_nop, false, false, 0)),
        None,
        Some(OpCode::new(0xEC, "CPX".to_string(), CPU::addr_abs, CPU::exec_cpx, false, false, 0)),
        Some(OpCode::new(0xED, "SBC".to_string(), CPU::addr_abs, CPU::exec_sbc, false, false, 0)),
        Some(OpCode::new(0xEE, "INC".to_string(), CPU::addr_abs, CPU::exec_inc, false, false, 0)),
        None,
        Some(OpCode::new(0xF0, "BEQ".to_string(), CPU::addr_rel, CPU::exec_beq, false, true,  0)),
        Some(OpCode::new(0xF1, "SBC".to_string(), CPU::addr_iny, CPU::exec_sbc, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0xF5, "SBC".to_string(), CPU::addr_zpx, CPU::exec_sbc, false, false, 0)),
        Some(OpCode::new(0xF6, "INC".to_string(), CPU::addr_zpx, CPU::exec_inc, false, false, 0)),
        None,
        Some(OpCode::new(0xF8, "SED".to_string(), CPU::addr_imp, CPU::exec_sed, false, false, 0)),
        Some(OpCode::new(0xF9, "SBC".to_string(), CPU::addr_aby, CPU::exec_sbc, true,  false, 0)),
        None,
        None,
        None,
        Some(OpCode::new(0xFD, "SBC".to_string(), CPU::addr_abx, CPU::exec_sbc, true,  false, 0)),
        Some(OpCode::new(0xFE, "INC".to_string(), CPU::addr_abx, CPU::exec_inc, false, false, 0)),
        None,
    ];
}
