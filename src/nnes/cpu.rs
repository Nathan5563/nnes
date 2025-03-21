pub mod flags;
pub mod interrupts;
pub mod opcodes;
pub mod registers;

use crate::nnes::NNES;
use crate::nnes::memory::{AddressingMode, Mem};
use crate::nnes::cpu::opcodes::opcodes_map;

impl NNES {
    pub fn step(&mut self, exit: &mut bool) {
        let pc: u16 = self.get_program_counter();
        let code: u8 = self.memory_read_u8(pc);
        self.set_program_counter(pc + 1);
        let ins = opcodes_map
            .get(&code)
            .expect(&format!("OpCode {:x} is not recognized", code));
        let mode: AddressingMode = ins.get_addressing_mode();
    
        match code {
            0xaa => self.handle_tax(),
            0xa8 => self.handle_tay(),
            0xba => self.handle_tsx(),
            0x8a => self.handle_txa(),
            0x9a => self.handle_txs(),
            0x98 => self.handle_tya(),
            0x18 => self.handle_clc(),
            0xd8 => self.handle_cld(),
            0x58 => self.handle_cli(),
            0xb8 => self.handle_clv(),
            0x38 => self.handle_sec(),
            0xf8 => self.handle_sed(),
            0x78 => self.handle_sei(),
            0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => self.handle_lda(mode),
            0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => self.handle_ldx(mode),
            0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => self.handle_ldy(mode),
            0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => self.handle_sta(mode),
            0x86 | 0x96 | 0x8e => self.handle_stx(mode),
            0x84 | 0x94 | 0x8c => self.handle_sty(mode),
            0x48 => self.handle_pha(),
            0x08 => self.handle_php(),
            0x68 => self.handle_pla(),
            0x28 => self.handle_plp(),
            0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => self.handle_and(mode),
            0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => self.handle_ora(mode),
            0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => self.handle_eor(mode),
            0x0a | 0x06 | 0x16 | 0x0e | 0x1e => self.handle_asl(mode),
            0x4a | 0x46 | 0x56 | 0x4e | 0x5e => self.handle_lsr(mode),
            0x2a | 0x26 | 0x36 | 0x2e | 0x3e => self.handle_rol(mode),
            0x6a | 0x66 | 0x76 | 0x6e | 0x7e => self.handle_ror(mode),
            0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => self.handle_adc(mode),
            0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => self.handle_sbc(mode),
            0xe6 | 0xf6 | 0xee | 0xfe => self.handle_inc(mode),
            0xe8 => self.handle_inx(),
            0xc8 => self.handle_iny(),
            0xc6 | 0xd6 | 0xce | 0xde => self.handle_dec(mode),
            0xca => self.handle_dex(),
            0x88 => self.handle_dey(),
            0x00 => {self.handle_brk(); *exit = true},
            0xea => self.handle_nop(),
            0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => self.handle_cmp(mode),
            0xe0 | 0xe4 | 0xec => self.handle_cmx(mode),
            0xc0 | 0xc4 | 0xcc => self.handle_cmy(mode),
            0x4c | 0x6c => self.handle_jmp(mode),
            0x20 => self.handle_jsr(),
            0x40 => self.handle_rti(),
            0x60 => self.handle_rts(),
            0x90 => self.handle_bcc(),
            0xb0 => self.handle_bcs(),
            0xf0 => self.handle_beq(),
            0x24 | 0x2c => self.handle_bit(mode),
            0x30 => self.handle_bmi(),
            0xd0 => self.handle_bne(),
            0x10 => self.handle_bpl(),
            0x50 => self.handle_bvc(),
            0x70 => self.handle_bvs(),
            _ => todo!("Opcode {code} not implemented!"),
        }
    }
}
