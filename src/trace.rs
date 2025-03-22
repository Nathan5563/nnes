use crate::nnes::cpu::opcodes::opcodes_map;
use crate::nnes::cpu::registers::Register;
use crate::nnes::memory::{AddressingMode, Mem};
use crate::nnes::NNES;
use crate::types::LOWER_BYTE;

/**
 * Examples:
 * C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD
 * D0BD  61 80     ADC ($80,X) @ 80 = 0200 = 80    A:7F X:00 Y:63 P:64 SP:FB
 * D0B7  8D 00 02  STA $0200 = 7F                  A:80 X:00 Y:63 P:E5 SP:FB
 * F96E  60        RTS                             A:FF X:00 Y:6E P:27 SP:F9
 * C6D2  14 A9    *NOP $A9,X @ 40 = 00             A:AA X:97 Y:4E P:EF SP:F5
 */
pub fn trace(nnes: &mut NNES) {
    // TODO: log the current cpu instruction in the above format
    let mut buf: String = String::new();

    let pc: u16 = nnes.get_program_counter();
    buf.push_str(format!("{:04X}", pc).as_str());
    buf.push_str("  ");

    let code: u8 = nnes.memory_read_u8(pc);
    buf.push_str(format!("{:02X}", code).as_str());
    buf.push(' ');

    let opcode = opcodes_map
        .get(&code)
        .expect(&format!("OpCode {:x} is not recognized", code));
    let mode: AddressingMode = opcode.get_addressing_mode();
    let instruction: &str = opcode.get_instruction().as_str();
    let num_bytes: u8 = opcode.get_bytes();

    let mut byte1: u8 = 0;
    let mut byte2: u8 = 0;
    let mut asm: String = String::from(instruction);
    asm.push(' ');
    match mode {
        AddressingMode::Implied => {
            asm.push_str("                            ");
        }
        AddressingMode::Accumulator => {
            asm.push('A');
            asm.push_str("                           ");
        }
        AddressingMode::Immediate => {
            byte1 = nnes.memory_read_u8(pc + 1);
            asm.push_str(format!("#${:02X}", byte1).as_str());
            asm.push_str("                        ");
        }
        AddressingMode::ZeroPage => {
            byte1 = nnes.memory_read_u8(pc + 1);
            asm.push_str(format!("${:02X}", byte1).as_str());
            asm.push(' ');
            let addr: u8 = nnes.memory_read_u8(byte1 as u16);
            asm.push_str(format!("= {:02X}", addr).as_str());
            asm.push_str("                    ");
        }
        AddressingMode::ZeroPageX => {
            byte1 = nnes.memory_read_u8(pc + 1);
            asm.push_str(format!("${:02X},X", byte1).as_str());
            asm.push(' ');
            let zp_addr: u8 = nnes.memory_read_u8(byte1 as u16);
            let zp_addr_x: u16 = (zp_addr + nnes.get_register(Register::XIndex)) as u16;
            let res: u8 = (zp_addr_x & LOWER_BYTE) as u8;
            asm.push_str(format!("@ {:02X}", zp_addr).as_str());
            asm.push(' ');
            let data: u8 = nnes.memory_read_u8(zp_addr as u16);
            asm.push_str(format!("= {:02X}", data).as_str());
            asm.push_str("             ");
        }
        AddressingMode::ZeroPageY => {
            byte1 = nnes.memory_read_u8(pc + 1);
            asm.push_str(format!("${:02X},X", byte1).as_str());
            asm.push(' ');
            let mut zp_addr: u8 = nnes.memory_read_u8(byte1 as u16);
            let mut zp_addr_y: u16 = (zp_addr + nnes.get_register(Register::YIndex)) as u16;
            let res: u8 = (zp_addr_y & LOWER_BYTE) as u8;
            asm.push_str(format!("@ {:02X}", zp_addr).as_str());
            asm.push(' ');
            let data: u8 = nnes.memory_read_u8(zp_addr as u16);
            asm.push_str(format!("= {:02X}", data).as_str());
            asm.push_str("             ");
        }
        AddressingMode::Absolute => {
            byte1 = nnes.memory_read_u8(pc + 1);
            byte2 = nnes.memory_read_u8(pc + 2);
        }
        AddressingMode::AbsoluteX => {
            byte1 = nnes.memory_read_u8(pc + 1);
            byte2 = nnes.memory_read_u8(pc + 2);
        }
        AddressingMode::AbsoluteY => {
            byte1 = nnes.memory_read_u8(pc + 1);
            byte2 = nnes.memory_read_u8(pc + 2);
        }
        AddressingMode::Relative => {
            byte1 = nnes.memory_read_u8(pc + 1);
        }
        AddressingMode::Indirect => {
            byte1 = nnes.memory_read_u8(pc + 1);
        }
        AddressingMode::IndirectX => {
            byte1 = nnes.memory_read_u8(pc + 1);
        }
        AddressingMode::IndirectY => {
            byte1 = nnes.memory_read_u8(pc + 1);
        }
    }
    if num_bytes > 1 {
        buf.push_str(format!("{:02X}", byte1).as_str());
        buf.push(' ');
        if num_bytes > 2 {
            buf.push_str(format!("{:02X}", byte2).as_str());
            buf.push_str("  ");
        }
        else {
            buf.push_str("    ");
        }
    }    
    else {
        buf.push_str("       ");
    }

    buf.push_str(asm.as_str());
    println!("{buf}");

    let reg_acc: u8 = nnes.get_register(Register::Accumulator);
    let reg_x: u8 = nnes.get_register(Register::XIndex);
    let reg_y: u8 = nnes.get_register(Register::YIndex);
    let flags: u8 = nnes.get_flags();
    let sp: u8 = nnes.get_stack_pointer();
}
