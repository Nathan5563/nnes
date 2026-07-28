use super::{Flags, CPU, IRQ_VECTOR, NMI_VECTOR};
use crate::utils::{bit_0, bit_5, bit_6, bit_7, hi_byte, lo_byte};

#[derive(Copy, Clone, PartialEq)]
pub enum AddressingMode {
    IMP,
    ACC,
    IMM,
    ZPG,
    ZPX,
    ZPY,
    ABS,
    ABX,
    ABY,
    IND,
    INX,
    INY,
    REL,
}

pub struct OpCode {
    pub code: u8,
    pub name: String,
    pub mode: AddressingMode,
    pub decode_fn: Option<fn(&mut CPU, subcycle: u8) -> bool>,
    pub execute_fn: fn(&mut CPU, subcycle: u8) -> bool,
    pub cycles: u8,
    pub bytes: u8,
}

impl OpCode {
    pub fn new(
        code: u8,
        name: String,
        mode: AddressingMode,
        decode_fn: Option<fn(&mut CPU, subcycle: u8) -> bool>,
        execute_fn: fn(&mut CPU, subcycle: u8) -> bool,
        cycles: u8,
        bytes: u8,
    ) -> Self {
        OpCode {
            code,
            name,
            mode,
            decode_fn,
            execute_fn,
            cycles,
            bytes,
        }
    }
}

lazy_static! {
    pub static ref opcodes_list: [Option<OpCode>; 256] = [
        Some(OpCode::new(0x00, "BRK".to_string(), AddressingMode::IMM, None,                CPU::brk, 7, 2)),
        Some(OpCode::new(0x01, "ORA".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::ora, 6, 2)),
        Some(OpCode::new(0x02, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0x03, "*SLO".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::slo, 8, 2)),
        Some(OpCode::new(0x04, "*NOP".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::nop, 3, 2)),
        Some(OpCode::new(0x05, "ORA".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::ora, 3, 2)),
        Some(OpCode::new(0x06, "ASL".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::asl, 5, 2)),
        Some(OpCode::new(0x07, "*SLO".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::slo, 5, 2)),
        Some(OpCode::new(0x08, "PHP".to_string(), AddressingMode::IMP, None,                CPU::php, 3, 1)),
        Some(OpCode::new(0x09, "ORA".to_string(), AddressingMode::IMM, None,                CPU::ora, 2, 2)),
        Some(OpCode::new(0x0A, "ASL".to_string(), AddressingMode::ACC, None,                CPU::asl, 2, 1)),
        Some(OpCode::new(0x0B, "*ANC".to_string(), AddressingMode::IMM, None, CPU::anc, 2, 2)),
        Some(OpCode::new(0x0C, "*NOP".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::nop, 4, 3)),
        Some(OpCode::new(0x0D, "ORA".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::ora, 4, 3)),
        Some(OpCode::new(0x0E, "ASL".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::asl, 6, 3)),
        Some(OpCode::new(0x0F, "*SLO".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::slo, 6, 3)),
        Some(OpCode::new(0x10, "BPL".to_string(), AddressingMode::REL, None,                CPU::bpl, 2, 2)),
        Some(OpCode::new(0x11, "ORA".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::ora, 5, 2)),
        Some(OpCode::new(0x12, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0x13, "*SLO".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::slo, 8, 2)),
        Some(OpCode::new(0x14, "*NOP".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::nop, 4, 2)),
        Some(OpCode::new(0x15, "ORA".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::ora, 4, 2)),
        Some(OpCode::new(0x16, "ASL".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::asl, 6, 2)),
        Some(OpCode::new(0x17, "*SLO".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::slo, 6, 2)),
        Some(OpCode::new(0x18, "CLC".to_string(), AddressingMode::IMP, None,                CPU::clc, 2, 1)),
        Some(OpCode::new(0x19, "ORA".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::ora, 4, 3)),
        Some(OpCode::new(0x1A, "*NOP".to_string(), AddressingMode::IMP, None, CPU::nop, 2, 1)),
        Some(OpCode::new(0x1B, "*SLO".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::slo, 7, 3)),
        Some(OpCode::new(0x1C, "*NOP".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::nop, 4, 3)),
        Some(OpCode::new(0x1D, "ORA".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::ora, 4, 3)),
        Some(OpCode::new(0x1E, "ASL".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::asl, 7, 3)),
        Some(OpCode::new(0x1F, "*SLO".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::slo, 7, 3)),
        Some(OpCode::new(0x20, "JSR".to_string(), AddressingMode::ABS, None,                CPU::jsr, 6, 3)),
        Some(OpCode::new(0x21, "AND".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::and, 6, 2)),
        Some(OpCode::new(0x22, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0x23, "*RLA".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::rla, 8, 2)),
        Some(OpCode::new(0x24, "BIT".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::bit, 3, 2)),
        Some(OpCode::new(0x25, "AND".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::and, 3, 2)),
        Some(OpCode::new(0x26, "ROL".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::rol, 5, 2)),
        Some(OpCode::new(0x27, "*RLA".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::rla, 5, 2)),
        Some(OpCode::new(0x28, "PLP".to_string(), AddressingMode::IMP, None,                CPU::plp, 4, 1)),
        Some(OpCode::new(0x29, "AND".to_string(), AddressingMode::IMM, None,                CPU::and, 2, 2)),
        Some(OpCode::new(0x2A, "ROL".to_string(), AddressingMode::ACC, None,                CPU::rol, 2, 1)),
        Some(OpCode::new(0x2B, "*ANC".to_string(), AddressingMode::IMM, None, CPU::anc, 2, 2)),
        Some(OpCode::new(0x2C, "BIT".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::bit, 4, 3)),
        Some(OpCode::new(0x2D, "AND".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::and, 4, 3)),
        Some(OpCode::new(0x2E, "ROL".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::rol, 6, 3)),
        Some(OpCode::new(0x2F, "*RLA".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::rla, 6, 3)),
        Some(OpCode::new(0x30, "BMI".to_string(), AddressingMode::REL, None,                CPU::bmi, 2, 2)),
        Some(OpCode::new(0x31, "AND".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::and, 5, 2)),
        Some(OpCode::new(0x32, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0x33, "*RLA".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::rla, 8, 2)),
        Some(OpCode::new(0x34, "*NOP".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::nop, 4, 2)),
        Some(OpCode::new(0x35, "AND".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::and, 4, 2)),
        Some(OpCode::new(0x36, "ROL".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::rol, 6, 2)),
        Some(OpCode::new(0x37, "*RLA".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::rla, 6, 2)),
        Some(OpCode::new(0x38, "SEC".to_string(), AddressingMode::IMP, None,                CPU::sec, 2, 1)),
        Some(OpCode::new(0x39, "AND".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::and, 4, 3)),
        Some(OpCode::new(0x3A, "*NOP".to_string(), AddressingMode::IMP, None, CPU::nop, 2, 1)),
        Some(OpCode::new(0x3B, "*RLA".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::rla, 7, 3)),
        Some(OpCode::new(0x3C, "*NOP".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::nop, 4, 3)),
        Some(OpCode::new(0x3D, "AND".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::and, 4, 3)),
        Some(OpCode::new(0x3E, "ROL".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::rol, 7, 3)),
        Some(OpCode::new(0x3F, "*RLA".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::rla, 7, 3)),
        Some(OpCode::new(0x40, "RTI".to_string(), AddressingMode::IMP, None,                CPU::rti, 6, 1)),
        Some(OpCode::new(0x41, "EOR".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::eor, 6, 2)),
        Some(OpCode::new(0x42, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0x43, "*SRE".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::sre, 8, 2)),
        Some(OpCode::new(0x44, "*NOP".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::nop, 3, 2)),
        Some(OpCode::new(0x45, "EOR".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::eor, 3, 2)),
        Some(OpCode::new(0x46, "LSR".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::lsr, 5, 2)),
        Some(OpCode::new(0x47, "*SRE".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::sre, 5, 2)),
        Some(OpCode::new(0x48, "PHA".to_string(), AddressingMode::IMP, None,                CPU::pha, 3, 1)),
        Some(OpCode::new(0x49, "EOR".to_string(), AddressingMode::IMM, None,                CPU::eor, 2, 2)),
        Some(OpCode::new(0x4A, "LSR".to_string(), AddressingMode::ACC, None,                CPU::lsr, 2, 1)),
        Some(OpCode::new(0x4B, "*ALR".to_string(), AddressingMode::IMM, None, CPU::alr, 2, 2)),
        Some(OpCode::new(0x4C, "JMP".to_string(), AddressingMode::ABS, None,                CPU::jmp, 3, 3)),
        Some(OpCode::new(0x4D, "EOR".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::eor, 4, 3)),
        Some(OpCode::new(0x4E, "LSR".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::lsr, 6, 3)),
        Some(OpCode::new(0x4F, "*SRE".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::sre, 6, 3)),
        Some(OpCode::new(0x50, "BVC".to_string(), AddressingMode::REL, None,                CPU::bvc, 2, 2)),
        Some(OpCode::new(0x51, "EOR".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::eor, 5, 2)),
        Some(OpCode::new(0x52, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0x53, "*SRE".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::sre, 8, 2)),
        Some(OpCode::new(0x54, "*NOP".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::nop, 4, 2)),
        Some(OpCode::new(0x55, "EOR".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::eor, 4, 2)),
        Some(OpCode::new(0x56, "LSR".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::lsr, 6, 2)),
        Some(OpCode::new(0x57, "*SRE".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::sre, 6, 2)),
        Some(OpCode::new(0x58, "CLI".to_string(), AddressingMode::IMP, None,                CPU::cli, 2, 1)),
        Some(OpCode::new(0x59, "EOR".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::eor, 4, 3)),
        Some(OpCode::new(0x5A, "*NOP".to_string(), AddressingMode::IMP, None, CPU::nop, 2, 1)),
        Some(OpCode::new(0x5B, "*SRE".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::sre, 7, 3)),
        Some(OpCode::new(0x5C, "*NOP".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::nop, 4, 3)),
        Some(OpCode::new(0x5D, "EOR".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::eor, 4, 3)),
        Some(OpCode::new(0x5E, "LSR".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::lsr, 7, 3)),
        Some(OpCode::new(0x5F, "*SRE".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::sre, 7, 3)),
        Some(OpCode::new(0x60, "RTS".to_string(), AddressingMode::IMP, None,                CPU::rts, 6, 1)),
        Some(OpCode::new(0x61, "ADC".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::adc, 6, 2)),
        Some(OpCode::new(0x62, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0x63, "*RRA".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::rra, 8, 2)),
        Some(OpCode::new(0x64, "*NOP".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::nop, 3, 2)),
        Some(OpCode::new(0x65, "ADC".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::adc, 3, 2)),
        Some(OpCode::new(0x66, "ROR".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::ror, 5, 2)),
        Some(OpCode::new(0x67, "*RRA".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::rra, 5, 2)),
        Some(OpCode::new(0x68, "PLA".to_string(), AddressingMode::IMP, None,                CPU::pla, 4, 1)),
        Some(OpCode::new(0x69, "ADC".to_string(), AddressingMode::IMM, None,                CPU::adc, 2, 2)),
        Some(OpCode::new(0x6A, "ROR".to_string(), AddressingMode::ACC, None,                CPU::ror, 2, 1)),
        Some(OpCode::new(0x6B, "*ARR".to_string(), AddressingMode::IMM, None, CPU::arr, 2, 2)),
        Some(OpCode::new(0x6C, "JMP".to_string(), AddressingMode::IND, None,                CPU::jmp, 5, 3)),  // Indirect bug
        Some(OpCode::new(0x6D, "ADC".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::adc, 4, 3)),
        Some(OpCode::new(0x6E, "ROR".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::ror, 6, 3)),
        Some(OpCode::new(0x6F, "*RRA".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::rra, 6, 3)),
        Some(OpCode::new(0x70, "BVS".to_string(), AddressingMode::REL, None,                CPU::bvs, 2, 2)),
        Some(OpCode::new(0x71, "ADC".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::adc, 5, 2)),
        Some(OpCode::new(0x72, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0x73, "*RRA".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::rra, 8, 2)),
        Some(OpCode::new(0x74, "*NOP".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::nop, 4, 2)),
        Some(OpCode::new(0x75, "ADC".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::adc, 4, 2)),
        Some(OpCode::new(0x76, "ROR".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::ror, 6, 2)),
        Some(OpCode::new(0x77, "*RRA".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::rra, 6, 2)),
        Some(OpCode::new(0x78, "SEI".to_string(), AddressingMode::IMP, None,                CPU::sei, 2, 1)),
        Some(OpCode::new(0x79, "ADC".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::adc, 4, 3)),
        Some(OpCode::new(0x7A, "*NOP".to_string(), AddressingMode::IMP, None, CPU::nop, 2, 1)),
        Some(OpCode::new(0x7B, "*RRA".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::rra, 7, 3)),
        Some(OpCode::new(0x7C, "*NOP".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::nop, 4, 3)),
        Some(OpCode::new(0x7D, "ADC".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::adc, 4, 3)),
        Some(OpCode::new(0x7E, "ROR".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::ror, 7, 3)),
        Some(OpCode::new(0x7F, "*RRA".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::rra, 7, 3)),
        Some(OpCode::new(0x80, "*NOP".to_string(), AddressingMode::IMM, None, CPU::nop, 2, 2)),
        Some(OpCode::new(0x81, "STA".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::sta, 6, 2)),
        Some(OpCode::new(0x82, "*NOP".to_string(), AddressingMode::IMM, None, CPU::nop, 2, 2)),
        Some(OpCode::new(0x83, "*SAX".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::sax, 6, 2)),
        Some(OpCode::new(0x84, "STY".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::sty, 3, 2)),
        Some(OpCode::new(0x85, "STA".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::sta, 3, 2)),
        Some(OpCode::new(0x86, "STX".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::stx, 3, 2)),
        Some(OpCode::new(0x87, "*SAX".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::sax, 3, 2)),
        Some(OpCode::new(0x88, "DEY".to_string(), AddressingMode::IMP, None,                CPU::dey, 2, 1)),
        Some(OpCode::new(0x89, "*NOP".to_string(), AddressingMode::IMM, None, CPU::nop, 2, 2)),
        Some(OpCode::new(0x8A, "TXA".to_string(), AddressingMode::IMP, None,                CPU::txa, 2, 1)),
        Some(OpCode::new(0x8B, "*ANE".to_string(), AddressingMode::IMM, None, CPU::ane, 2, 2)),
        Some(OpCode::new(0x8C, "STY".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::sty, 4, 3)),
        Some(OpCode::new(0x8D, "STA".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::sta, 4, 3)),
        Some(OpCode::new(0x8E, "STX".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::stx, 4, 3)),
        Some(OpCode::new(0x8F, "*SAX".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::sax, 4, 3)),
        Some(OpCode::new(0x90, "BCC".to_string(), AddressingMode::REL, None,                CPU::bcc, 2, 2)),
        Some(OpCode::new(0x91, "STA".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::sta, 6, 2)),
        Some(OpCode::new(0x92, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0x93, "*SHA".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::sha, 6, 2)),
        Some(OpCode::new(0x94, "STY".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::sty, 4, 2)),
        Some(OpCode::new(0x95, "STA".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::sta, 4, 2)),
        Some(OpCode::new(0x96, "STX".to_string(), AddressingMode::ZPY, Some(CPU::addr_zpy), CPU::stx, 4, 2)),
        Some(OpCode::new(0x97, "*SAX".to_string(), AddressingMode::ZPY, Some(CPU::addr_zpy), CPU::sax, 4, 2)),
        Some(OpCode::new(0x98, "TYA".to_string(), AddressingMode::IMP, None,                CPU::tya, 2, 1)),
        Some(OpCode::new(0x99, "STA".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::sta, 5, 3)),
        Some(OpCode::new(0x9A, "TXS".to_string(), AddressingMode::IMP, None,                CPU::txs, 2, 1)),
        Some(OpCode::new(0x9B, "*TAS".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::tas, 5, 3)),
        Some(OpCode::new(0x9C, "*SHY".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::shy, 5, 3)),
        Some(OpCode::new(0x9D, "STA".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::sta, 5, 3)),
        Some(OpCode::new(0x9E, "*SHX".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::shx, 5, 3)),
        Some(OpCode::new(0x9F, "*SHA".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::sha, 5, 3)),
        Some(OpCode::new(0xA0, "LDY".to_string(), AddressingMode::IMM, None,                CPU::ldy, 2, 2)),
        Some(OpCode::new(0xA1, "LDA".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::lda, 6, 2)),
        Some(OpCode::new(0xA2, "LDX".to_string(), AddressingMode::IMM, None,                CPU::ldx, 2, 2)),
        Some(OpCode::new(0xA3, "*LAX".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::lax, 6, 2)),
        Some(OpCode::new(0xA4, "LDY".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::ldy, 3, 2)),
        Some(OpCode::new(0xA5, "LDA".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::lda, 3, 2)),
        Some(OpCode::new(0xA6, "LDX".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::ldx, 3, 2)),
        Some(OpCode::new(0xA7, "*LAX".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::lax, 3, 2)),
        Some(OpCode::new(0xA8, "TAY".to_string(), AddressingMode::IMP, None,                CPU::tay, 2, 1)),
        Some(OpCode::new(0xA9, "LDA".to_string(), AddressingMode::IMM, None,                CPU::lda, 2, 2)),
        Some(OpCode::new(0xAA, "TAX".to_string(), AddressingMode::IMP, None,                CPU::tax, 2, 1)),
        Some(OpCode::new(0xAB, "*LXA".to_string(), AddressingMode::IMM, None, CPU::lxa, 2, 2)),
        Some(OpCode::new(0xAC, "LDY".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::ldy, 4, 3)),
        Some(OpCode::new(0xAD, "LDA".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::lda, 4, 3)),
        Some(OpCode::new(0xAE, "LDX".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::ldx, 4, 3)),
        Some(OpCode::new(0xAF, "*LAX".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::lax, 4, 3)),
        Some(OpCode::new(0xB0, "BCS".to_string(), AddressingMode::REL, None,                CPU::bcs, 2, 2)),
        Some(OpCode::new(0xB1, "LDA".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::lda, 5, 2)),
        Some(OpCode::new(0xB2, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0xB3, "*LAX".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::lax, 5, 2)),
        Some(OpCode::new(0xB4, "LDY".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::ldy, 4, 2)),
        Some(OpCode::new(0xB5, "LDA".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::lda, 4, 2)),
        Some(OpCode::new(0xB6, "LDX".to_string(), AddressingMode::ZPY, Some(CPU::addr_zpy), CPU::ldx, 4, 2)),
        Some(OpCode::new(0xB7, "*LAX".to_string(), AddressingMode::ZPY, Some(CPU::addr_zpy), CPU::lax, 4, 2)),
        Some(OpCode::new(0xB8, "CLV".to_string(), AddressingMode::IMP, None,                CPU::clv, 2, 1)),
        Some(OpCode::new(0xB9, "LDA".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::lda, 4, 3)),
        Some(OpCode::new(0xBA, "TSX".to_string(), AddressingMode::IMP, None,                CPU::tsx, 2, 1)),
        Some(OpCode::new(0xBB, "*LAS".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::las, 4, 3)),
        Some(OpCode::new(0xBC, "LDY".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::ldy, 4, 3)),
        Some(OpCode::new(0xBD, "LDA".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::lda, 4, 3)),
        Some(OpCode::new(0xBE, "LDX".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::ldx, 4, 3)),
        Some(OpCode::new(0xBF, "*LAX".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::lax, 4, 3)),
        Some(OpCode::new(0xC0, "CPY".to_string(), AddressingMode::IMM, None,                CPU::cpy, 2, 2)),
        Some(OpCode::new(0xC1, "CMP".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::cmp, 6, 2)),
        Some(OpCode::new(0xC2, "*NOP".to_string(), AddressingMode::IMM, None, CPU::nop, 2, 2)),
        Some(OpCode::new(0xC3, "*DCP".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::dcp, 8, 2)),
        Some(OpCode::new(0xC4, "CPY".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::cpy, 3, 2)),
        Some(OpCode::new(0xC5, "CMP".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::cmp, 3, 2)),
        Some(OpCode::new(0xC6, "DEC".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::dec, 5, 2)),
        Some(OpCode::new(0xC7, "*DCP".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::dcp, 5, 2)),
        Some(OpCode::new(0xC8, "INY".to_string(), AddressingMode::IMP, None,                CPU::iny, 2, 1)),
        Some(OpCode::new(0xC9, "CMP".to_string(), AddressingMode::IMM, None,                CPU::cmp, 2, 2)),
        Some(OpCode::new(0xCA, "DEX".to_string(), AddressingMode::IMP, None,                CPU::dex, 2, 1)),
        Some(OpCode::new(0xCB, "*SBX".to_string(), AddressingMode::IMM, None, CPU::sbx, 2, 2)),
        Some(OpCode::new(0xCC, "CPY".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::cpy, 4, 3)),
        Some(OpCode::new(0xCD, "CMP".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::cmp, 4, 3)),
        Some(OpCode::new(0xCE, "DEC".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::dec, 6, 3)),
        Some(OpCode::new(0xCF, "*DCP".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::dcp, 6, 3)),
        Some(OpCode::new(0xD0, "BNE".to_string(), AddressingMode::REL, None,                CPU::bne, 2, 2)),
        Some(OpCode::new(0xD1, "CMP".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::cmp, 5, 2)),
        Some(OpCode::new(0xD2, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0xD3, "*DCP".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::dcp, 8, 2)),
        Some(OpCode::new(0xD4, "*NOP".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::nop, 4, 2)),
        Some(OpCode::new(0xD5, "CMP".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::cmp, 4, 2)),
        Some(OpCode::new(0xD6, "DEC".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::dec, 6, 2)),
        Some(OpCode::new(0xD7, "*DCP".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::dcp, 6, 2)),
        Some(OpCode::new(0xD8, "CLD".to_string(), AddressingMode::IMP, None,                CPU::cld, 2, 1)),
        Some(OpCode::new(0xD9, "CMP".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::cmp, 4, 3)),
        Some(OpCode::new(0xDA, "*NOP".to_string(), AddressingMode::IMP, None, CPU::nop, 2, 1)),
        Some(OpCode::new(0xDB, "*DCP".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::dcp, 7, 3)),
        Some(OpCode::new(0xDC, "*NOP".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::nop, 4, 3)),
        Some(OpCode::new(0xDD, "CMP".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::cmp, 4, 3)),
        Some(OpCode::new(0xDE, "DEC".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::dec, 7, 3)),
        Some(OpCode::new(0xDF, "*DCP".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::dcp, 7, 3)),
        Some(OpCode::new(0xE0, "CPX".to_string(), AddressingMode::IMM, None,                CPU::cpx, 2, 2)),
        Some(OpCode::new(0xE1, "SBC".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::sbc, 6, 2)),
        Some(OpCode::new(0xE2, "*NOP".to_string(), AddressingMode::IMM, None, CPU::nop, 2, 2)),
        Some(OpCode::new(0xE3, "*ISB".to_string(), AddressingMode::INX, Some(CPU::addr_inx), CPU::isb, 8, 2)),
        Some(OpCode::new(0xE4, "CPX".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::cpx, 3, 2)),
        Some(OpCode::new(0xE5, "SBC".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::sbc, 3, 2)),
        Some(OpCode::new(0xE6, "INC".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::inc, 5, 2)),
        Some(OpCode::new(0xE7, "*ISB".to_string(), AddressingMode::ZPG, Some(CPU::addr_zpg), CPU::isb, 5, 2)),
        Some(OpCode::new(0xE8, "INX".to_string(), AddressingMode::IMP, None,                CPU::inx, 2, 1)),
        Some(OpCode::new(0xE9, "SBC".to_string(), AddressingMode::IMM, None,                CPU::sbc, 2, 2)),
        Some(OpCode::new(0xEA, "NOP".to_string(), AddressingMode::IMP, None,                CPU::nop, 2, 1)),
        Some(OpCode::new(0xEB, "*SBC".to_string(), AddressingMode::IMM, None, CPU::sbc, 2, 2)),
        Some(OpCode::new(0xEC, "CPX".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::cpx, 4, 3)),
        Some(OpCode::new(0xED, "SBC".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::sbc, 4, 3)),
        Some(OpCode::new(0xEE, "INC".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::inc, 6, 3)),
        Some(OpCode::new(0xEF, "*ISB".to_string(), AddressingMode::ABS, Some(CPU::addr_abs), CPU::isb, 6, 3)),
        Some(OpCode::new(0xF0, "BEQ".to_string(), AddressingMode::REL, None,                CPU::beq, 2, 2)),
        Some(OpCode::new(0xF1, "SBC".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::sbc, 5, 2)),
        Some(OpCode::new(0xF2, "*JAM".to_string(), AddressingMode::IMP, None, CPU::jam, 2, 1)),
        Some(OpCode::new(0xF3, "*ISB".to_string(), AddressingMode::INY, Some(CPU::addr_iny), CPU::isb, 8, 2)),
        Some(OpCode::new(0xF4, "*NOP".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::nop, 4, 2)),
        Some(OpCode::new(0xF5, "SBC".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::sbc, 4, 2)),
        Some(OpCode::new(0xF6, "INC".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::inc, 6, 2)),
        Some(OpCode::new(0xF7, "*ISB".to_string(), AddressingMode::ZPX, Some(CPU::addr_zpx), CPU::isb, 6, 2)),
        Some(OpCode::new(0xF8, "SED".to_string(), AddressingMode::IMP, None,                CPU::sed, 2, 1)),
        Some(OpCode::new(0xF9, "SBC".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::sbc, 4, 3)),
        Some(OpCode::new(0xFA, "*NOP".to_string(), AddressingMode::IMP, None, CPU::nop, 2, 1)),
        Some(OpCode::new(0xFB, "*ISB".to_string(), AddressingMode::ABY, Some(CPU::addr_aby), CPU::isb, 7, 3)),
        Some(OpCode::new(0xFC, "*NOP".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::nop, 4, 3)),
        Some(OpCode::new(0xFD, "SBC".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::sbc, 4, 3)),
        Some(OpCode::new(0xFE, "INC".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::inc, 7, 3)),
        Some(OpCode::new(0xFF, "*ISB".to_string(), AddressingMode::ABX, Some(CPU::addr_abx), CPU::isb, 7, 3)),
    ];
}

impl CPU {
    // Official addressing functions
    fn addr_zpg(&mut self, _subcycle: u8) -> bool {
        self.store.addr = self.bus.mem_read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        true
    }

    fn addr_zpx(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.addr_zpg(subcycle);
                false
            }
            1 => {
                let _ = self.bus.mem_read(self.store.addr);
                self.store.addr =
                    self.x.wrapping_add(self.store.addr as u8) as u16;
                true
            }
            _ => unreachable!(),
        }
    }

    fn addr_zpy(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.addr_zpg(subcycle);
                false
            }
            1 => {
                let _ = self.bus.mem_read(self.store.addr);
                self.store.addr =
                    self.y.wrapping_add(self.store.addr as u8) as u16;
                true
            }
            _ => unreachable!(),
        }
    }

    fn addr_abs(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.lo = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                false
            }
            1 => {
                self.store.hi = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.store.addr =
                    u16::from_le_bytes([self.store.lo, self.store.hi]);
                true
            }
            _ => unreachable!(),
        }
    }

    fn addr_abx(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.lo = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                false
            }
            1 => {
                self.store.hi = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.store.data = self.store.lo;
                self.store.lo = self.store.lo.wrapping_add(self.x);
                self.store.addr =
                    u16::from_le_bytes([self.store.lo, self.store.hi]);

                self.page_crossed = self.store.data > self.store.lo;
                let write_ins = matches!(
                    self.ins.unwrap().code,
                    0x9D | 0x1E
                        | 0x3E
                        | 0x5E
                        | 0x7E
                        | 0xDE
                        | 0xFE
                        | 0x1F
                        | 0x3F
                        | 0x5F
                        | 0x7F
                        | 0xDF
                        | 0xFF
                        | 0x9C
                );
                // Write instructions already count the dummy read cycle
                if self.page_crossed && !write_ins {
                    self.required_ins_ticks += 1;
                }
                !(self.page_crossed || write_ins)
            }
            2 => {
                let _ = self.bus.mem_read(self.store.addr);
                if self.page_crossed {
                    self.store.addr = self.store.addr.wrapping_add(0x100);
                }
                true
            }
            _ => unreachable!(),
        }
    }

    fn addr_aby(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.lo = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                false
            }
            1 => {
                self.store.hi = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.store.data = self.store.lo;
                self.store.lo = self.store.lo.wrapping_add(self.y);
                self.store.addr =
                    u16::from_le_bytes([self.store.lo, self.store.hi]);

                self.page_crossed = self.store.data > self.store.lo;
                let write_ins = matches!(
                    self.ins.unwrap().code,
                    0x99 | 0x1B
                        | 0x3B
                        | 0x5B
                        | 0x7B
                        | 0xDB
                        | 0xFB
                        | 0x9B
                        | 0x9E
                        | 0x9F
                );
                // Write instructions already count the dummy read cycle
                if self.page_crossed && !write_ins {
                    self.required_ins_ticks += 1;
                }
                !(self.page_crossed || write_ins)
            }
            2 => {
                let _ = self.bus.mem_read(self.store.addr);
                if self.page_crossed {
                    self.store.addr = self.store.addr.wrapping_add(0x100);
                }
                true
            }
            _ => unreachable!(),
        }
    }

    fn addr_inx(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.data = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                false
            }
            1 => {
                let _ = self.bus.mem_read(self.store.data as u16);
                self.store.data = self.store.data.wrapping_add(self.x);
                false
            }
            2 => {
                self.store.lo = self.bus.mem_read(self.store.data as u16);
                false
            }
            3 => {
                self.store.hi =
                    self.bus.mem_read(self.store.data.wrapping_add(1) as u16);
                self.store.addr =
                    u16::from_le_bytes([self.store.lo, self.store.hi]);
                true
            }
            _ => unreachable!(),
        }
    }

    fn addr_iny(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.data = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                false
            }
            1 => {
                self.store.lo = self.bus.mem_read(self.store.data as u16);
                false
            }
            2 => {
                self.store.hi =
                    self.bus.mem_read(self.store.data.wrapping_add(1) as u16);
                self.store.data = self.store.lo;
                self.store.lo = self.store.lo.wrapping_add(self.y);
                self.store.addr =
                    u16::from_le_bytes([self.store.lo, self.store.hi]);

                self.page_crossed = self.store.data > self.store.lo;
                let write_ins = matches!(
                    self.ins.unwrap().code,
                    0x91 | 0x13 | 0x33 | 0x53 | 0x73 | 0xD3 | 0xF3 | 0x93
                );
                // Write instructions already count the dummy read cycle
                if self.page_crossed && !write_ins {
                    self.required_ins_ticks += 1;
                }
                !(self.page_crossed || write_ins)
            }
            3 => {
                let _ = self.bus.mem_read(self.store.addr);
                if self.page_crossed {
                    self.store.addr = self.store.addr.wrapping_add(0x100);
                }
                true
            }
            _ => unreachable!(),
        }
    }

    // Official execute functions
    fn brk(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                let _ = self.bus.mem_read(self.pc);
                if self.software_interrupt {
                    self.pc = self.pc.wrapping_add(1);
                }
                false
            }
            1 => {
                self.stack_push(hi_byte(self.pc));
                false
            }
            2 => {
                self.stack_push(lo_byte(self.pc));
                false
            }
            3 => {
                let p = if self.software_interrupt {
                    self.p.bits() | Flags::BREAK.bits()
                } else {
                    self.p.bits() & !Flags::BREAK.bits()
                };
                self.stack_push(p);
                false
            }
            4 => {
                self.p.insert(Flags::INTERRUPT_DISABLE);
                match self.store.vector {
                    NMI_VECTOR => {
                        self.nmi_pending = false;
                    }
                    // IRQ is level-triggered: the device holds the line
                    // until acknowledged, so the CPU never clears it
                    IRQ_VECTOR => {}
                    _ => unreachable!(),
                }
                self.store.lo = self.bus.mem_read(self.store.vector);
                false
            }
            5 => {
                self.store.hi = self.bus.mem_read(self.store.vector + 1);
                self.store.addr =
                    u16::from_le_bytes([self.store.lo, self.store.hi]);
                self.pc = self.store.addr;

                self.software_interrupt = false;
                self.servicing_interrupt = false;
                self.hijacked = false;
                true
            }
            _ => unreachable!(),
        }
    }

    fn ora(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.a |= self.store.data;
        self.set_nz(self.a);
        true
    }

    fn asl(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => self.shift(true, false, subcycle),
            _ => unreachable!(),
        }
    }

    fn php(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                let _ = self.get_operand();
                false
            }
            1 => {
                self.stack_push(self.p.bits() | Flags::BREAK.bits());
                true
            }
            _ => unreachable!(),
        }
    }

    fn bpl(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => {
                self.branch(!self.p.contains(Flags::NEGATIVE), subcycle)
            }
            _ => unreachable!(),
        }
    }

    fn clc(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.p.remove(Flags::CARRY);
        true
    }

    fn jsr(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.lo = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                false
            }
            1 => {
                let _ = self.stack_peek();
                false
            }
            2 => {
                self.stack_push(hi_byte(self.pc));
                false
            }
            3 => {
                self.stack_push(lo_byte(self.pc));
                false
            }
            4 => {
                self.store.hi = self.bus.mem_read(self.pc);
                self.pc = u16::from_le_bytes([self.store.lo, self.store.hi]);
                true
            }
            _ => unreachable!(),
        }
    }

    fn and(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.a &= self.store.data;
        self.set_nz(self.a);
        true
    }

    fn bit(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        let res = self.a & self.store.data;
        self.p.set(Flags::ZERO, res == 0);
        self.p.set(Flags::OVERFLOW, bit_6(self.store.data) != 0);
        self.p.set(Flags::NEGATIVE, bit_7(self.store.data) != 0);
        true
    }

    fn rol(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => self.shift(true, true, subcycle),
            _ => unreachable!(),
        }
    }

    fn plp(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                let _ = self.get_operand();
                false
            }
            1 => {
                let _ = self.stack_peek();
                false
            }
            2 => {
                let flags = Flags::from_bits_truncate(self.stack_pop());
                self.p.set(Flags::CARRY, flags.contains(Flags::CARRY));
                self.p.set(Flags::ZERO, flags.contains(Flags::ZERO));
                self.p.set(
                    Flags::INTERRUPT_DISABLE,
                    flags.contains(Flags::INTERRUPT_DISABLE),
                );
                self.p.set(
                    Flags::DECIMAL_MODE,
                    flags.contains(Flags::DECIMAL_MODE),
                );
                self.p.set(Flags::OVERFLOW, flags.contains(Flags::OVERFLOW));
                self.p.set(Flags::NEGATIVE, flags.contains(Flags::NEGATIVE));
                true
            }
            _ => unreachable!(),
        }
    }

    fn bmi(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => {
                self.branch(self.p.contains(Flags::NEGATIVE), subcycle)
            }
            _ => unreachable!(),
        }
    }

    fn sec(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.p.insert(Flags::CARRY);
        true
    }

    fn rti(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                let _ = self.get_operand();
                false
            }
            1 => {
                let _ = self.stack_peek();
                false
            }
            2 => {
                let flags = Flags::from_bits_truncate(self.stack_pop());
                self.p.set(Flags::CARRY, flags.contains(Flags::CARRY));
                self.p.set(Flags::ZERO, flags.contains(Flags::ZERO));
                self.p.set(
                    Flags::INTERRUPT_DISABLE,
                    flags.contains(Flags::INTERRUPT_DISABLE),
                );
                self.p.set(
                    Flags::DECIMAL_MODE,
                    flags.contains(Flags::DECIMAL_MODE),
                );
                self.p.set(Flags::OVERFLOW, flags.contains(Flags::OVERFLOW));
                self.p.set(Flags::NEGATIVE, flags.contains(Flags::NEGATIVE));
                false
            }
            3 => {
                self.pc =
                    u16::from_le_bytes([self.stack_pop(), hi_byte(self.pc)]);
                false
            }
            4 => {
                self.pc =
                    u16::from_le_bytes([lo_byte(self.pc), self.stack_pop()]);
                true
            }
            _ => unreachable!(),
        }
    }

    fn eor(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.a ^= self.store.data;
        self.set_nz(self.a);
        true
    }

    fn lsr(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => self.shift(false, false, subcycle),
            _ => unreachable!(),
        }
    }

    fn pha(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                let _ = self.get_operand();
                false
            }
            1 => {
                self.stack_push(self.a);
                true
            }
            _ => unreachable!(),
        }
    }

    fn jmp(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.lo = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                false
            }
            1 => {
                self.store.hi = self.bus.mem_read(self.pc);
                self.store.addr =
                    u16::from_le_bytes([self.store.lo, self.store.hi]);
                if self.ins.unwrap().mode == AddressingMode::ABS {
                    self.pc = self.store.addr;
                    true
                } else {
                    self.pc = self.pc.wrapping_add(1);
                    false
                }
            }
            2 => {
                self.store.lo = self.bus.mem_read(self.store.addr);
                false
            }
            3 => {
                self.store.addr = u16::from_le_bytes([
                    lo_byte(self.store.addr.wrapping_add(1)),
                    hi_byte(self.store.addr),
                ]);
                self.store.hi = self.bus.mem_read(self.store.addr);
                self.pc = u16::from_le_bytes([self.store.lo, self.store.hi]);
                true
            }
            _ => unreachable!(),
        }
    }

    fn bvc(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => {
                self.branch(!self.p.contains(Flags::OVERFLOW), subcycle)
            }
            _ => unreachable!(),
        }
    }

    fn cli(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.p.remove(Flags::INTERRUPT_DISABLE);
        true
    }

    fn rts(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                let _ = self.get_operand();
                false
            }
            1 => {
                let _ = self.stack_peek();
                false
            }
            2 => {
                self.store.lo = self.stack_pop();
                false
            }
            3 => {
                self.store.hi = self.stack_pop();
                self.pc = u16::from_le_bytes([self.store.lo, self.store.hi]);
                false
            }
            4 => {
                let _ = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                true
            }
            _ => unreachable!(),
        }
    }

    fn adc(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.a = self.add(self.a, self.store.data, false);
        self.set_nz(self.a);
        true
    }

    fn ror(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => self.shift(false, true, subcycle),
            _ => unreachable!(),
        }
    }

    fn pla(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                let _ = self.get_operand();
                false
            }
            1 => {
                let _ = self.stack_peek();
                false
            }
            2 => {
                self.a = self.stack_pop();
                self.set_nz(self.a);
                true
            }
            _ => unreachable!(),
        }
    }

    fn bvs(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => {
                self.branch(self.p.contains(Flags::OVERFLOW), subcycle)
            }
            _ => unreachable!(),
        }
    }

    fn sei(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.p.insert(Flags::INTERRUPT_DISABLE);
        true
    }

    fn sta(&mut self, _subcycle: u8) -> bool {
        self.bus.mem_write(self.store.addr, self.a);
        true
    }

    fn sty(&mut self, _subcycle: u8) -> bool {
        self.bus.mem_write(self.store.addr, self.y);
        true
    }

    fn stx(&mut self, _subcycle: u8) -> bool {
        self.bus.mem_write(self.store.addr, self.x);
        true
    }

    fn dey(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.y = self.y.wrapping_sub(1);
        self.set_nz(self.y);
        true
    }

    fn txa(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.a = self.x;
        self.set_nz(self.a);
        true
    }

    fn bcc(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => self.branch(!self.p.contains(Flags::CARRY), subcycle),
            _ => unreachable!(),
        }
    }

    fn tya(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.a = self.y;
        self.set_nz(self.a);
        true
    }

    fn txs(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.sp = self.x;
        true
    }

    fn ldy(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.y = self.store.data;
        self.set_nz(self.y);
        true
    }

    fn lda(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.a = self.store.data;
        self.set_nz(self.a);
        true
    }


    fn rmw_combo(
        &mut self,
        subcycle: u8,
        modify: fn(&mut CPU, u8) -> u8,
        combine: fn(&mut CPU, u8),
    ) -> bool {
        match subcycle {
            0 => {
                self.store.data = self.get_operand();
                false
            }
            1 => {
                self.set_result();
                self.store.data = modify(self, self.store.data);
                false
            }
            2 => {
                self.set_result();
                combine(self, self.store.data);
                true
            }
            _ => unreachable!(),
        }
    }

    fn slo(&mut self, subcycle: u8) -> bool {
        self.rmw_combo(
            subcycle,
            |cpu, data| cpu.shift_data(true, false, false, data),
            |cpu, data| {
                cpu.a |= data;
                cpu.set_nz(cpu.a);
            },
        )
    }

    fn rla(&mut self, subcycle: u8) -> bool {
        self.rmw_combo(
            subcycle,
            |cpu, data| {
                let cf = cpu.p.contains(Flags::CARRY);
                cpu.shift_data(true, true, cf, data)
            },
            |cpu, data| {
                cpu.a &= data;
                cpu.set_nz(cpu.a);
            },
        )
    }

    fn sre(&mut self, subcycle: u8) -> bool {
        self.rmw_combo(
            subcycle,
            |cpu, data| cpu.shift_data(false, false, false, data),
            |cpu, data| {
                cpu.a ^= data;
                cpu.set_nz(cpu.a);
            },
        )
    }

    fn rra(&mut self, subcycle: u8) -> bool {
        self.rmw_combo(
            subcycle,
            |cpu, data| {
                let cf = cpu.p.contains(Flags::CARRY);
                cpu.shift_data(false, true, cf, data)
            },
            // the ADC consumes the carry the ROR just produced
            |cpu, data| {
                cpu.a = cpu.add(cpu.a, data, false);
                cpu.set_nz(cpu.a);
            },
        )
    }

    fn dcp(&mut self, subcycle: u8) -> bool {
        self.rmw_combo(
            subcycle,
            |_cpu, data| data.wrapping_sub(1),
            |cpu, data| cpu.compare(cpu.a, data),
        )
    }

    fn isb(&mut self, subcycle: u8) -> bool {
        self.rmw_combo(
            subcycle,
            |_cpu, data| data.wrapping_add(1),
            |cpu, data| {
                cpu.a = cpu.add(cpu.a, data, true);
                cpu.set_nz(cpu.a);
            },
        )
    }



    fn jam(&mut self, _subcycle: u8) -> bool {
        // JAM wedges the CPU: nothing executes and no interrupt is serviced
        // until reset
        self.jammed = true;
        true
    }

    fn anc(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.a &= self.store.data;
        self.set_nz(self.a);
        self.p.set(Flags::CARRY, bit_7(self.a) != 0);
        true
    }

    fn alr(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.a &= self.store.data;
        self.a = self.shift_data(false, false, false, self.a);
        self.set_nz(self.a);
        true
    }

    fn arr(&mut self, _subcycle: u8) -> bool {
        // AND, then ROR through carry; C comes from bit 6 of the result and
        // V from bit 6 XOR bit 5
        self.store.data = self.get_operand();
        let cf = self.p.contains(Flags::CARRY);
        self.a = ((self.a & self.store.data) >> 1) | ((cf as u8) << 7);
        self.set_nz(self.a);
        self.p.set(Flags::CARRY, bit_6(self.a) != 0);
        self.p
            .set(Flags::OVERFLOW, bit_6(self.a) != bit_5(self.a));
        true
    }

    fn ane(&mut self, _subcycle: u8) -> bool {
        // Unstable: the real result depends on a chip- and temperature-
        // dependent constant; $EE is the widely accepted model
        self.store.data = self.get_operand();
        self.a = (self.a | 0xEE) & self.x & self.store.data;
        self.set_nz(self.a);
        true
    }

    fn lxa(&mut self, _subcycle: u8) -> bool {
        // Unstable: modeled with the constant $FF, so A = X = operand
        self.store.data = self.get_operand();
        self.a = self.store.data;
        self.x = self.store.data;
        self.set_nz(self.a);
        true
    }

    fn sbx(&mut self, _subcycle: u8) -> bool {
        // X = (A & X) - operand without borrow-in; C is set like CMP
        self.store.data = self.get_operand();
        let t = self.a & self.x;
        self.p.set(Flags::CARRY, t >= self.store.data);
        self.x = t.wrapping_sub(self.store.data);
        self.set_nz(self.x);
        true
    }

    fn las(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        let value = self.store.data & self.sp;
        self.a = value;
        self.x = value;
        self.sp = value;
        self.set_nz(value);
        true
    }

    fn sh_write(&mut self, reg: u8) {
        // The stored value is the register ANDed with (high byte of the
        // pre-index address + 1); when the index crosses a page, the
        // corrupted value also replaces the high byte of the target
        let value = reg & self.store.hi.wrapping_add(1);
        if self.page_crossed {
            self.store.addr =
                ((value as u16) << 8) | (self.store.addr & 0x00FF);
        }
        self.bus.mem_write(self.store.addr, value);
    }

    fn sha(&mut self, _subcycle: u8) -> bool {
        self.sh_write(self.a & self.x);
        true
    }

    fn shx(&mut self, _subcycle: u8) -> bool {
        self.sh_write(self.x);
        true
    }

    fn shy(&mut self, _subcycle: u8) -> bool {
        self.sh_write(self.y);
        true
    }

    fn tas(&mut self, _subcycle: u8) -> bool {
        self.sp = self.a & self.x;
        self.sh_write(self.a & self.x);
        true
    }

    fn lax(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.a = self.store.data;
        self.x = self.store.data;
        self.set_nz(self.a);
        true
    }

    fn sax(&mut self, _subcycle: u8) -> bool {
        self.bus.mem_write(self.store.addr, self.a & self.x);
        true
    }

    fn ldx(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.x = self.store.data;
        self.set_nz(self.x);
        true
    }

    fn tay(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.y = self.a;
        self.set_nz(self.y);
        true
    }

    fn tax(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.x = self.a;
        self.set_nz(self.x);
        true
    }

    fn bcs(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => self.branch(self.p.contains(Flags::CARRY), subcycle),
            _ => unreachable!(),
        }
    }

    fn clv(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.p.remove(Flags::OVERFLOW);
        true
    }

    fn tsx(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.x = self.sp;
        self.set_nz(self.x);
        true
    }

    fn cpy(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.compare(self.y, self.store.data);
        true
    }

    fn cmp(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.compare(self.a, self.store.data);
        true
    }

    fn dec(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.data = self.get_operand();
                false
            }
            1 => {
                self.set_result();
                self.store.data = self.store.data.wrapping_sub(1);
                false
            }
            2 => {
                self.set_result();
                self.set_nz(self.store.data);
                true
            }
            _ => unreachable!(),
        }
    }

    fn iny(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.y = self.y.wrapping_add(1);
        self.set_nz(self.y);
        true
    }

    fn dex(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.x = self.x.wrapping_sub(1);
        self.set_nz(self.x);
        true
    }

    fn bne(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => self.branch(!self.p.contains(Flags::ZERO), subcycle),
            _ => unreachable!(),
        }
    }

    fn cld(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.p.remove(Flags::DECIMAL_MODE);
        true
    }

    fn cpx(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.compare(self.x, self.store.data);
        true
    }

    fn sbc(&mut self, _subcycle: u8) -> bool {
        self.store.data = self.get_operand();
        self.a = self.add(self.a, self.store.data, true);
        self.set_nz(self.a);
        true
    }

    fn inc(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.data = self.get_operand();
                false
            }
            1 => {
                self.set_result();
                self.store.data = self.store.data.wrapping_add(1);
                false
            }
            2 => {
                self.set_result();
                self.set_nz(self.store.data);
                true
            }
            _ => unreachable!(),
        }
    }

    fn inx(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.x = self.x.wrapping_add(1);
        self.set_nz(self.x);
        true
    }

    fn nop(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        true
    }

    fn beq(&mut self, subcycle: u8) -> bool {
        match subcycle {
            0 | 1 | 2 => self.branch(self.p.contains(Flags::ZERO), subcycle),
            _ => unreachable!(),
        }
    }

    fn sed(&mut self, _subcycle: u8) -> bool {
        let _ = self.get_operand();
        self.p.insert(Flags::DECIMAL_MODE);
        true
    }

    // Helpers
    fn add(&mut self, n: u8, mut m: u8, subtract: bool) -> u8 {
        let cbit = self.p.contains(Flags::CARRY) as u8;

        if subtract {
            m = !m;
        }
        let mut res = n as u16 + m as u16 + cbit as u16;
        if res > 0xff {
            self.p.insert(Flags::CARRY);
        } else {
            self.p.remove(Flags::CARRY);
        }

        res &= 0xFF;
        if bit_7((n ^ res as u8) & (m ^ res as u8)) != 0 {
            self.p.insert(Flags::OVERFLOW);
        } else {
            self.p.remove(Flags::OVERFLOW);
        }

        res as u8
    }

    fn shift(&mut self, left: bool, with_carry: bool, subcycle: u8) -> bool {
        let cf = self.p.contains(Flags::CARRY);
        if self.ins.unwrap().mode == AddressingMode::ACC {
            let _ = self.get_operand();
            self.a = self.shift_data(left, with_carry, cf, self.a);
            self.set_nz(self.a);
            true
        } else {
            match subcycle {
                0 => {
                    self.store.data = self.get_operand();
                    false
                }
                1 => {
                    self.set_result();
                    self.store.data =
                        self.shift_data(left, with_carry, cf, self.store.data);
                    false
                }
                2 => {
                    self.set_result();
                    self.set_nz(self.store.data);
                    true
                }
                _ => unreachable!(),
            }
        }
    }

    fn shift_data(
        &mut self,
        left: bool,
        with_carry: bool,
        cf: bool,
        mut data: u8,
    ) -> u8 {
        if left {
            self.p.set(Flags::CARRY, bit_7(data) != 0);
            data <<= 1;
            if with_carry && cf {
                data |= 1;
            }
        } else {
            self.p.set(Flags::CARRY, bit_0(data) != 0);
            data >>= 1;
            if with_carry && cf {
                data |= 128;
            }
        }
        data
    }

    fn compare(&mut self, n: u8, m: u8) {
        self.set_nz(n.wrapping_sub(m));
        self.p.set(Flags::CARRY, n >= m);
    }

    fn branch(&mut self, jmp: bool, subcycle: u8) -> bool {
        match subcycle {
            0 => {
                self.store.offset = self.get_operand() as i8;
                if jmp {
                    self.required_ins_ticks += 1;
                    self.suppress_poll = true;
                }
                !jmp
            }
            1 => {
                let _ = self.bus.mem_read(self.pc);
                self.store.addr =
                    self.pc.wrapping_add(self.store.offset as i16 as u16);
                self.pc = u16::from_le_bytes([
                    lo_byte(self.store.addr),
                    hi_byte(self.pc),
                ]);
                if hi_byte(self.pc) != hi_byte(self.store.addr) {
                    self.required_ins_ticks += 1;
                }
                hi_byte(self.pc) == hi_byte(self.store.addr)
            }
            2 => {
                let _ = self.bus.mem_read(self.pc);
                self.pc = self.store.addr;
                true
            }
            _ => unreachable!(),
        }
    }

    fn get_operand(&mut self) -> u8 {
        match self.ins.unwrap().mode {
            AddressingMode::IMP | AddressingMode::ACC => {
                self.bus.mem_read(self.pc)
            }
            AddressingMode::IMM | AddressingMode::REL => {
                self.pc = self.pc.wrapping_add(1);
                self.bus.mem_read(self.pc.wrapping_sub(1))
            }
            _ => self.bus.mem_read(self.store.addr),
        }
    }

    fn set_result(&mut self) {
        match self.ins.unwrap().mode {
            AddressingMode::ACC => {
                self.a = self.store.data;
            }
            _ => self.bus.mem_write(self.store.addr, self.store.data),
        }
    }

    fn set_nz(&mut self, data: u8) {
        self.p.set(Flags::ZERO, data == 0);
        self.p.set(Flags::NEGATIVE, bit_7(data) != 0);
    }
}
