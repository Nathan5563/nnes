use crate::cpu::enums::AddressingMode;
use std::collections::HashMap;

pub struct InsState {
    cpu_state: u64, // [8 empty bits|pc|sp|a|x|y|p]
    code: u8,
    name: String,
    mode: AddressingMode,
    micro_ops: &'static [fn(&mut InsState)],
    current_cycle: usize,
    addr_low: u8,
    addr_high: u8,
    data: u8,
}

impl InsState {
    pub fn new(cpu_state: u64, opcode: &'static OpCode) -> Self {
        InsState {
            cpu_state: cpu_state,
            code: opcode.code,
            name: opcode.name.clone(),
            mode: opcode.mode,
            micro_ops: opcode.micro_ops,
            current_cycle: 0,
            addr_low: 0,
            addr_high: 0,
            data: 0,
        }
    }

    pub fn execute(ins_state: &mut InsState) -> u64 {
        0
    }
}

pub struct OpCode {
    code: u8,
    name: String,
    mode: AddressingMode,
    micro_ops: &'static [fn(&mut InsState)],
}

impl OpCode {
    pub fn new(
        code: u8,
        name: String,
        mode: AddressingMode,
        micro_ops: &'static [fn(&mut InsState)],
    ) -> Self {
        OpCode {
            code: code,
            name: name,
            mode: mode,
            micro_ops: micro_ops,
        }
    }
}

lazy_static! {
    pub static ref opcodes_list: Vec<OpCode> = vec![
    // CPU
        // Transfer
        OpCode::new(0xaa, "TAX".to_string(), AddressingMode::Implied, InsState::tax_aa),
        OpCode::new(0xba, "TSX".to_string(), AddressingMode::Implied, InsState::tsx_ba),
        OpCode::new(0xa8, "TAY".to_string(), AddressingMode::Implied, InsState::tay_a8),
        OpCode::new(0x8a, "TXA".to_string(), AddressingMode::Implied, InsState::txa_8a),
        OpCode::new(0x9a, "TXS".to_string(), AddressingMode::Implied, InsState::txs_9a),
        OpCode::new(0x98, "TYA".to_string(), AddressingMode::Implied, InsState::tya_98),
        // Flags
        OpCode::new(0x18, "CLC".to_string(), AddressingMode::Implied, InsState::clc_18),
        OpCode::new(0xd8, "CLD".to_string(), AddressingMode::Implied, InsState::cld_d8),
        OpCode::new(0x58, "CLI".to_string(), AddressingMode::Implied, InsState::cli_58),
        OpCode::new(0xb8, "CLV".to_string(), AddressingMode::Implied, InsState::clv_b8),
        OpCode::new(0x38, "SEC".to_string(), AddressingMode::Implied, InsState::sec_38),
        OpCode::new(0xf8, "SED".to_string(), AddressingMode::Implied, InsState::sed_f8),
        OpCode::new(0x78, "SEI".to_string(), AddressingMode::Implied, InsState::sei_78),

    // Memory
        // Load
        OpCode::new(0xa9, "LDA".to_string(), AddressingMode::Immediate, InsState::lda_a9),
        OpCode::new(0xa5, "LDA".to_string(), AddressingMode::ZeroPage, InsState::lda_a5),
        OpCode::new(0xb5, "LDA".to_string(), AddressingMode::ZeroPageX, InsState::lda_b5),
        OpCode::new(0xad, "LDA".to_string(), AddressingMode::Absolute, InsState::lda_ad),
        OpCode::new(0xbd, "LDA".to_string(), AddressingMode::AbsoluteX, InsState::lda_bd),
        OpCode::new(0xb9, "LDA".to_string(), AddressingMode::AbsoluteY, InsState::lda_b9),
        OpCode::new(0xa1, "LDA".to_string(), AddressingMode::IndirectX, InsState::lda_a1),
        OpCode::new(0xb1, "LDA".to_string(), AddressingMode::IndirectY, InsState::lda_b1),
        OpCode::new(0xa2, "LDX".to_string(), AddressingMode::Immediate, InsState::ldx_a2),
        OpCode::new(0xa6, "LDX".to_string(), AddressingMode::ZeroPage, InsState::ldx_a6),
        OpCode::new(0xb6, "LDX".to_string(), AddressingMode::ZeroPageY, InsState::ldx_b6),
        OpCode::new(0xae, "LDX".to_string(), AddressingMode::Absolute, InsState::ldx_ae),
        OpCode::new(0xbe, "LDX".to_string(), AddressingMode::AbsoluteY, InsState::ldx_be),
        OpCode::new(0xa0, "LDY".to_string(), AddressingMode::Immediate, InsState::ldy_a0),
        OpCode::new(0xa4, "LDY".to_string(), AddressingMode::ZeroPage, InsState::ldy_a4),
        OpCode::new(0xb4, "LDY".to_string(), AddressingMode::ZeroPageX, InsState::ldy_b4),
        OpCode::new(0xac, "LDY".to_string(), AddressingMode::Absolute, InsState::ldy_ac),
        OpCode::new(0xbc, "LDY".to_string(), AddressingMode::AbsoluteX, InsState::ldy_bc),
        // Store
        OpCode::new(0x85, "STA".to_string(), AddressingMode::ZeroPage, InsState::sta_85),
        OpCode::new(0x95, "STA".to_string(), AddressingMode::ZeroPageX, InsState::sta_95),
        OpCode::new(0x8d, "STA".to_string(), AddressingMode::Absolute, InsState::sta_8d),
        OpCode::new(0x9d, "STA".to_string(), AddressingMode::AbsoluteX, InsState::sta_9d),
        OpCode::new(0x99, "STA".to_string(), AddressingMode::AbsoluteY, InsState::sta_99),
        OpCode::new(0x81, "STA".to_string(), AddressingMode::IndirectX, InsState::sta_81),
        OpCode::new(0x91, "STA".to_string(), AddressingMode::IndirectY, InsState::sta_91),
        OpCode::new(0x86, "STX".to_string(), AddressingMode::ZeroPage, InsState::stx_86),
        OpCode::new(0x96, "STX".to_string(), AddressingMode::ZeroPageY, InsState::stx_96),
        OpCode::new(0x8e, "STX".to_string(), AddressingMode::Absolute, InsState::stx_8e),
        OpCode::new(0x84, "STY".to_string(), AddressingMode::ZeroPage, InsState::sty_84),
        OpCode::new(0x94, "STY".to_string(), AddressingMode::ZeroPageX, InsState::sty_94),
        OpCode::new(0x8c, "STY".to_string(), AddressingMode::Absolute, InsState::sty_8c),
        // Stack
        OpCode::new(0x48, "PHA".to_string(), AddressingMode::Implied, InsState::pha_48),
        OpCode::new(0x08, "PHP".to_string(), AddressingMode::Implied, InsState::php_08),
        OpCode::new(0x68, "PLA".to_string(), AddressingMode::Implied, InsState::pla_68),
        OpCode::new(0x28, "PLP".to_string(), AddressingMode::Implied, InsState::plp_28),

    // Binary
        // AND
        OpCode::new(0x29, "AND".to_string(), AddressingMode::Immediate, InsState::and_29),
        OpCode::new(0x25, "AND".to_string(), AddressingMode::ZeroPage, InsState::and_25),
        OpCode::new(0x35, "AND".to_string(), AddressingMode::ZeroPageX, InsState::and_35),
        OpCode::new(0x2d, "AND".to_string(), AddressingMode::Absolute, InsState::and_2d),
        OpCode::new(0x3d, "AND".to_string(), AddressingMode::AbsoluteX, InsState::and_3d),
        OpCode::new(0x39, "AND".to_string(), AddressingMode::AbsoluteY, InsState::and_39),
        OpCode::new(0x21, "AND".to_string(), AddressingMode::IndirectX, InsState::and_21),
        OpCode::new(0x31, "AND".to_string(), AddressingMode::IndirectY, InsState::and_31),
        // OR
        OpCode::new(0x09, "ORA".to_string(), AddressingMode::Immediate, InsState::ora_09),
        OpCode::new(0x05, "ORA".to_string(), AddressingMode::ZeroPage, InsState::ora_05),
        OpCode::new(0x15, "ORA".to_string(), AddressingMode::ZeroPageX, InsState::ora_15),
        OpCode::new(0x0d, "ORA".to_string(), AddressingMode::Absolute, InsState::ora_0d),
        OpCode::new(0x1d, "ORA".to_string(), AddressingMode::AbsoluteX, InsState::ora_1d),
        OpCode::new(0x19, "ORA".to_string(), AddressingMode::AbsoluteY, InsState::ora_19),
        OpCode::new(0x01, "ORA".to_string(), AddressingMode::IndirectX, InsState::ora_01),
        OpCode::new(0x11, "ORA".to_string(), AddressingMode::IndirectY, InsState::ora_11),
        // XOR
        OpCode::new(0x49, "EOR".to_string(), AddressingMode::Immediate, InsState::eor_49),
        OpCode::new(0x45, "EOR".to_string(), AddressingMode::ZeroPage, InsState::eor_45),
        OpCode::new(0x55, "EOR".to_string(), AddressingMode::ZeroPageX, InsState::eor_55),
        OpCode::new(0x4d, "EOR".to_string(), AddressingMode::Absolute, InsState::eor_4d),
        OpCode::new(0x5d, "EOR".to_string(), AddressingMode::AbsoluteX, InsState::eor_5d),
        OpCode::new(0x59, "EOR".to_string(), AddressingMode::AbsoluteY, InsState::eor_59),
        OpCode::new(0x41, "EOR".to_string(), AddressingMode::IndirectX, InsState::eor_41),
        OpCode::new(0x51, "EOR".to_string(), AddressingMode::IndirectY, InsState::eor_51),
        // SAL
        OpCode::new(0x0a, "ASL".to_string(), AddressingMode::Accumulator, InsState::asl_0a),
        OpCode::new(0x06, "ASL".to_string(), AddressingMode::ZeroPage, InsState::asl_06),
        OpCode::new(0x16, "ASL".to_string(), AddressingMode::ZeroPageX, InsState::asl_16),
        OpCode::new(0x0e, "ASL".to_string(), AddressingMode::Absolute, InsState::asl_0e),
        OpCode::new(0x1e, "ASL".to_string(), AddressingMode::AbsoluteX, InsState::asl_1e),
        // SHR
        OpCode::new(0x4a, "LSR".to_string(), AddressingMode::Accumulator, InsState::lsr_4a),
        OpCode::new(0x46, "LSR".to_string(), AddressingMode::ZeroPage, InsState::lsr_46),
        OpCode::new(0x56, "LSR".to_string(), AddressingMode::ZeroPageX, InsState::lsr_56),
        OpCode::new(0x4e, "LSR".to_string(), AddressingMode::Absolute, InsState::lsr_4e),
        OpCode::new(0x5e, "LSR".to_string(), AddressingMode::AbsoluteX, InsState::lsr_5e),
        // RCL
        OpCode::new(0x2a, "ROL".to_string(), AddressingMode::Accumulator, InsState::rol_2a),
        OpCode::new(0x26, "ROL".to_string(), AddressingMode::ZeroPage, InsState::rol_26),
        OpCode::new(0x36, "ROL".to_string(), AddressingMode::ZeroPageX, InsState::rol_36),
        OpCode::new(0x2e, "ROL".to_string(), AddressingMode::Absolute, InsState::rol_2e),
        OpCode::new(0x3e, "ROL".to_string(), AddressingMode::AbsoluteX, InsState::rol_3e),
        // RCR
        OpCode::new(0x6a, "ROR".to_string(), AddressingMode::Accumulator, InsState::ror_6a),
        OpCode::new(0x66, "ROR".to_string(), AddressingMode::ZeroPage, InsState::ror_66),
        OpCode::new(0x76, "ROR".to_string(), AddressingMode::ZeroPageX, InsState::ror_76),
        OpCode::new(0x6e, "ROR".to_string(), AddressingMode::Absolute, InsState::ror_6e),
        OpCode::new(0x7e, "ROR".to_string(), AddressingMode::AbsoluteX, InsState::ror_7e),

    // Arithmetic
        // ADC
        OpCode::new(0x69, "ADC".to_string(), AddressingMode::Immediate, InsState::adc_69),
        OpCode::new(0x65, "ADC".to_string(), AddressingMode::ZeroPage, InsState::adc_65),
        OpCode::new(0x75, "ADC".to_string(), AddressingMode::ZeroPageX, InsState::adc_75),
        OpCode::new(0x6d, "ADC".to_string(), AddressingMode::Absolute, InsState::adc_6d),
        OpCode::new(0x7d, "ADC".to_string(), AddressingMode::AbsoluteX, InsState::adc_7d),
        OpCode::new(0x79, "ADC".to_string(), AddressingMode::AbsoluteY, InsState::adc_79),
        OpCode::new(0x61, "ADC".to_string(), AddressingMode::IndirectX, InsState::adc_61),
        OpCode::new(0x71, "ADC".to_string(), AddressingMode::IndirectY, InsState::adc_71),
        // SBC
        OpCode::new(0xe9, "SBC".to_string(), AddressingMode::Immediate, InsState::sbc_e9),
        OpCode::new(0xe5, "SBC".to_string(), AddressingMode::ZeroPage, InsState::sbc_e5),
        OpCode::new(0xf5, "SBC".to_string(), AddressingMode::ZeroPageX, InsState::sbc_f5),
        OpCode::new(0xed, "SBC".to_string(), AddressingMode::Absolute, InsState::sbc_ed),
        OpCode::new(0xfd, "SBC".to_string(), AddressingMode::AbsoluteX, InsState::sbc_fd),
        OpCode::new(0xf9, "SBC".to_string(), AddressingMode::AbsoluteY, InsState::sbc_f9),
        OpCode::new(0xe1, "SBC".to_string(), AddressingMode::IndirectX, InsState::sbc_e1),
        OpCode::new(0xf1, "SBC".to_string(), AddressingMode::IndirectY, InsState::sbc_f1),
        // INC
        OpCode::new(0xe6, "INC".to_string(), AddressingMode::ZeroPage, InsState::inc_e6),
        OpCode::new(0xf6, "INC".to_string(), AddressingMode::ZeroPageX, InsState::inc_f6),
        OpCode::new(0xee, "INC".to_string(), AddressingMode::Absolute, InsState::inc_ee),
        OpCode::new(0xfe, "INC".to_string(), AddressingMode::AbsoluteX, InsState::inc_fe),
        OpCode::new(0xe8, "INX".to_string(), AddressingMode::Implied, InsState::inx_e8),
        OpCode::new(0xc8, "INY".to_string(), AddressingMode::Implied, InsState::iny_c8),
        // DEC
        OpCode::new(0xc6, "DEC".to_string(), AddressingMode::ZeroPage, InsState::dec_c6),
        OpCode::new(0xd6, "DEC".to_string(), AddressingMode::ZeroPageX, InsState::dec_d6),
        OpCode::new(0xce, "DEC".to_string(), AddressingMode::Absolute, InsState::dec_ce),
        OpCode::new(0xde, "DEC".to_string(), AddressingMode::AbsoluteX, InsState::dec_de),
        OpCode::new(0xca, "DEX".to_string(), AddressingMode::Implied, InsState::dex_ca),
        OpCode::new(0x88, "DEY".to_string(), AddressingMode::Implied, InsState::dey_88),

    // Control flow
        // Stop
        OpCode::new(0x00, "BRK".to_string(), AddressingMode::Implied, InsState::brk_00),
        OpCode::new(0xea, "NOP".to_string(), AddressingMode::Implied, InsState::nop_ea),
        // Compare
        OpCode::new(0xc9, "CMP".to_string(), AddressingMode::Immediate, InsState::cmp_c9),
        OpCode::new(0xc5, "CMP".to_string(), AddressingMode::ZeroPage, InsState::cmp_c5),
        OpCode::new(0xd5, "CMP".to_string(), AddressingMode::ZeroPageX, InsState::cmp_d5),
        OpCode::new(0xcd, "CMP".to_string(), AddressingMode::Absolute, InsState::cmp_cd),
        OpCode::new(0xdd, "CMP".to_string(), AddressingMode::AbsoluteX, InsState::cmp_dd),
        OpCode::new(0xd9, "CMP".to_string(), AddressingMode::AbsoluteY, InsState::cmp_d9),
        OpCode::new(0xc1, "CMP".to_string(), AddressingMode::IndirectX, InsState::cmp_c1),
        OpCode::new(0xd1, "CMP".to_string(), AddressingMode::IndirectY, InsState::cmp_d1),
        OpCode::new(0xe0, "CPX".to_string(), AddressingMode::Immediate, InsState::cpx_e0),
        OpCode::new(0xe4, "CPX".to_string(), AddressingMode::ZeroPage, InsState::cpx_e4),
        OpCode::new(0xec, "CPX".to_string(), AddressingMode::Absolute, InsState::cpx_ec),
        OpCode::new(0xc0, "CPY".to_string(), AddressingMode::Immediate, InsState::cpy_c0),
        OpCode::new(0xc4, "CPY".to_string(), AddressingMode::ZeroPage, InsState::cpy_c4),
        OpCode::new(0xcc, "CPY".to_string(), AddressingMode::Absolute, InsState::cpy_cc),
        // Jump
        OpCode::new(0x4c, "JMP".to_string(), AddressingMode::Absolute, InsState::jmp_4c),
        OpCode::new(0x6c, "JMP".to_string(), AddressingMode::Indirect, InsState::jmp_6c), // 6502 bug with 0xXXFF
        OpCode::new(0x20, "JSR".to_string(), AddressingMode::Absolute, InsState::jsr_20),
        OpCode::new(0x40, "RTI".to_string(), AddressingMode::Implied, InsState::rti_40),
        OpCode::new(0x60, "RTS".to_string(), AddressingMode::Implied, InsState::rts_60),
        // Conditionals
        OpCode::new(0x90, "BCC".to_string(), AddressingMode::Relative, InsState::bcc_90),
        OpCode::new(0xb0, "BCS".to_string(), AddressingMode::Relative, InsState::bcs_b0),
        OpCode::new(0xf0, "BEQ".to_string(), AddressingMode::Relative, InsState::beq_f0),
        OpCode::new(0x24, "BIT".to_string(), AddressingMode::ZeroPage, InsState::bit_24),
        OpCode::new(0x2c, "BIT".to_string(), AddressingMode::Absolute, InsState::bit_2c),
        OpCode::new(0x30, "BMI".to_string(), AddressingMode::Relative, InsState::bmi_30),
        OpCode::new(0xd0, "BNE".to_string(), AddressingMode::Relative, InsState::bne_d0),
        OpCode::new(0x10, "BPL".to_string(), AddressingMode::Relative, InsState::bpl_10),
        OpCode::new(0x50, "BVC".to_string(), AddressingMode::Relative, InsState::bvc_50),
        OpCode::new(0x70, "BVS".to_string(), AddressingMode::Relative, InsState::bvs_70),
    ];


    pub static ref opcodes_map: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for op in &*opcodes_list {
            map.insert(op.code, op);
        };
        map
    };
}
