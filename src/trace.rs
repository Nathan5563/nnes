use crate::nnes::cpu::opcodes::opcodes_map;
use crate::nnes::cpu::registers::Register;
use crate::nnes::memory::{AddressingMode, Mem};
use crate::nnes::NNES;
use crate::types::{LOWER_BYTE, UPPER_BYTE};
use crate::utils::{add_mod_16bit, add_mod_8bit};

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
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            asm.push_str(format!("#${:02X}", byte1).as_str());
            asm.push_str("                        ");
        }
        AddressingMode::ZeroPage => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            asm.push_str(format!("${:02X} ", byte1).as_str());
            let addr: u8 = nnes.memory_read_u8(byte1 as u16);
            asm.push_str(format!("= {:02X}", addr).as_str());
            asm.push_str("                    ");
        }
        AddressingMode::ZeroPageX => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            asm.push_str(format!("${:02X},X ", byte1).as_str());
            let zp_addr_x: u8 = add_mod_8bit(byte1, nnes.get_register(Register::XIndex));
            asm.push_str(format!("@ {:02X} ", zp_addr_x).as_str());
            let data: u8 = nnes.memory_read_u8(zp_addr_x as u16);
            asm.push_str(format!("= {:02X}", data).as_str());
            asm.push_str("             ");
        }
        AddressingMode::ZeroPageY => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            asm.push_str(format!("${:02X},Y ", byte1).as_str());
            let zp_addr_y: u8 = add_mod_8bit(byte1, nnes.get_register(Register::YIndex));
            asm.push_str(format!("@ {:02X} ", zp_addr_y).as_str());
            let data: u8 = nnes.memory_read_u8(zp_addr_y as u16);
            asm.push_str(format!("= {:02X}", data).as_str());
            asm.push_str("             ");
        }
        AddressingMode::Absolute => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            byte2 = nnes.memory_read_u8(add_mod_16bit(pc, 2));
            let addr: u16 = ((byte2 as u16) << 8) | (byte1 as u16);
            asm.push_str(format!("${:04X} ", addr).as_str());
            if instruction != "JMP" && instruction != "JSR" {
                let data: u8 = nnes.memory_read_u8(addr);
                asm.push_str(format!("= {:02X}", data).as_str());
                asm.push_str("                  ");
            } else {
                asm.push_str("                      ");
            }
        }
        AddressingMode::AbsoluteX => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            byte2 = nnes.memory_read_u8(add_mod_16bit(pc, 2));
            let base_addr: u16 = ((byte2 as u16) << 8) | (byte1 as u16);
            asm.push_str(format!("${:04X},X ", base_addr).as_str());
            let addr_x: u16 = add_mod_16bit(base_addr, nnes.get_register(Register::XIndex) as u16);
            asm.push_str(format!("@ {:04X} ", addr_x).as_str());
            let data: u8 = nnes.memory_read_u8(addr_x);
            asm.push_str(format!("= {:02X}", data).as_str());
            asm.push_str("         ");
        }
        AddressingMode::AbsoluteY => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            byte2 = nnes.memory_read_u8(add_mod_16bit(pc, 2));
            let base_addr: u16 = ((byte2 as u16) << 8) | (byte1 as u16);
            asm.push_str(format!("${:04X},Y ", base_addr).as_str());
            let addr_y: u16 = add_mod_16bit(base_addr, nnes.get_register(Register::YIndex) as u16);
            asm.push_str(format!("@ {:04X} ", addr_y).as_str());
            let data: u8 = nnes.memory_read_u8(addr_y);
            asm.push_str(format!("= {:02X}", data).as_str());
            asm.push_str("         ");
        }
        AddressingMode::Relative => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            let offset: i8 = byte1 as i8;
            let res: i32 = nnes.get_program_counter() as i32 + offset as i32 + 2;
            asm.push_str(format!("${:04X}", res as u16).as_str());
            asm.push_str("                       ");
        }
        AddressingMode::Indirect => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            byte2 = nnes.memory_read_u8(add_mod_16bit(pc, 2));
            let indirect: u16 = ((byte2 as u16) << 8) | (byte1 as u16);
            asm.push_str(format!("(${:04X}) ", indirect).as_str());
            let addrl: u8 = nnes.memory_read_u8(indirect);
            let addrh: u8;
            if indirect & LOWER_BYTE == 0xff {
                addrh = nnes.memory_read_u8(indirect & UPPER_BYTE);
            }
            else {
                addrh = nnes.memory_read_u8(indirect + 1);
            }
            let addr: u16 = ((addrh as u16) << 8) | (addrl as u16);
            asm.push_str(format!("= {:04X}", addr).as_str());
            asm.push_str("              ");
        }
        AddressingMode::IndirectX => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            asm.push_str(format!("(${:02X},X) ", byte1).as_str());
            let indirect: u8 = add_mod_8bit(byte1, nnes.get_register(Register::XIndex));
            asm.push_str(format!("@ {:02X} ", indirect).as_str());
            let addrl: u8 = nnes.memory_read_u8(indirect as u16);
            let addrh: u8 = nnes.memory_read_u8(add_mod_8bit(indirect, 1) as u16);
            let addr: u16 = ((addrh as u16) << 8) | (addrl as u16);
            asm.push_str(format!("= {:04X} ", addr).as_str());
            let data: u8 = nnes.memory_read_u8(addr);
            asm.push_str(format!("= {:02X}", data).as_str());
            asm.push_str("    ");
        }
        AddressingMode::IndirectY => {
            byte1 = nnes.memory_read_u8(add_mod_16bit(pc, 1));
            asm.push_str(format!("(${:02X}),Y ", byte1).as_str());
            let addrl: u8 = nnes.memory_read_u8(byte1 as u16);
            let addrh: u8 = nnes.memory_read_u8(add_mod_8bit(byte1, 1) as u16);
            let addr: u16 = ((addrh as u16) << 8) | (addrl as u16);
            asm.push_str(format!("= {:04X} ", addr).as_str());
            let indexed: u16 = add_mod_16bit(addr, nnes.get_register(Register::YIndex) as u16);
            asm.push_str(format!("@ {:04X} ", indexed).as_str());
            let data: u8 = nnes.memory_read_u8(indexed);
            asm.push_str(format!("= {:02X}", data).as_str());
            asm.push_str("  ");
        }
    }
    if num_bytes > 1 {
        buf.push_str(format!("{:02X}", byte1).as_str());
        buf.push(' ');
        if num_bytes > 2 {
            buf.push_str(format!("{:02X}", byte2).as_str());
            buf.push_str("  ");
        } else {
            buf.push_str("    ");
        }
    } else {
        buf.push_str("       ");
    }

    buf.push_str(asm.as_str());

    let reg_acc: u8 = nnes.get_register(Register::Accumulator);
    let reg_x: u8 = nnes.get_register(Register::XIndex);
    let reg_y: u8 = nnes.get_register(Register::YIndex);
    let flags: u8 = nnes.get_flags();
    let sp: u8 = nnes.get_stack_pointer();

    buf.push_str(
        format!(
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            reg_acc, reg_x, reg_y, flags, sp
        )
        .as_str(),
    );
    println!("{buf}");
}
