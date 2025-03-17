#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

#[macro_use]
extern crate lazy_static;

mod nnes;
use nnes::{types::*, utils::*, AddressingMode, Flag, Register, NNES, STACK_OFFSET};

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lda_immediate() {
        let mut nnes = NNES::new();
        // Program: LDA immediate (0xa9), operand, then BRK.
        nnes.play_test(vec![0xa9, 0x42, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x42);
    }

    #[test]
    fn test_lda_zero_page() {
        let mut nnes = NNES::new();
        // Set memory at zero page address 0x10.
        nnes.memory_write_u8(0x10, 0x55);
        // Program: LDA zero page (0xa5), operand 0x10, then BRK.
        nnes.play_test(vec![0xa5, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x55);
    }

    #[test]
    fn test_lda_zero_page_x() {
        let mut nnes = NNES::new();
        // Set register X and value at (operand + X)
        nnes.set_register(Register::XIndex, 0x05);
        nnes.memory_write_u8(0x10 + 0x05, 0x66);
        // Program: LDA zero page,X (0xb5), operand 0x10, then BRK.
        nnes.play_test(vec![0xb5, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x66);
    }

    #[test]
    fn test_lda_absolute() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x1234, 0x77);
        // Program: LDA absolute (0xad), operand low/high for 0x1234, then BRK.
        nnes.play_test(vec![0xad, 0x34, 0x12, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x77);
    }

    #[test]
    fn test_lda_absolute_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::XIndex, 0x05);
        nnes.memory_write_u8(0x1200 + 0x05, 0x88);
        // Program: LDA absolute,X (0xbd), operands for base address 0x1200, then BRK.
        nnes.play_test(vec![0xbd, 0x00, 0x12, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x88);
    }

    #[test]
    fn test_lda_absolute_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::YIndex, 0x06);
        nnes.memory_write_u8(0x1200 + 0x06, 0x99);
        // Program: LDA absolute,Y (0xb9), operands for base address 0x1200, then BRK.
        nnes.play_test(vec![0xb9, 0x00, 0x12, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x99);
    }

    #[test]
    fn test_lda_indirect_x() {
        let mut nnes = NNES::new();
        // Set X register so that (operand + X) points into our pointer table in zero page.
        nnes.set_register(Register::XIndex, 0x04);
        // We choose an operand of 0x10. In (Indirect,X) mode, effective pointer = (0x10 + X) = 0x14.
        // Write effective address 0x3000 into zero page at address 0x14 (low byte) and 0x15 (high byte).
        let effective_addr = 0x3000;
        nnes.memory_write_u8(0x14, (effective_addr & 0xff) as u8);
        nnes.memory_write_u8(0x15, (effective_addr >> 8) as u8);
        // Place the value to be loaded at the effective address.
        nnes.memory_write_u8(effective_addr, 0x99);
        // LDA Indirect,X has opcode 0xA1.
        nnes.play_test(vec![0xA1, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x99);
    }

    #[test]
    fn test_lda_indirect_y() {
        let mut nnes = NNES::new();
        // Set Y register for offset.
        nnes.set_register(Register::YIndex, 0x05);
        // Choose a zero page pointer address 0x20.
        // Write base effective address 0x4000 into zero page at 0x20 (low) and 0x21 (high).
        let base_addr = 0x4000;
        nnes.memory_write_u8(0x20, (base_addr & 0xff) as u8);
        nnes.memory_write_u8(0x21, (base_addr >> 8) as u8);
        // Effective address = base_addr + Y = 0x4000 + 0x05.
        nnes.memory_write_u8(base_addr + 0x05, 0xAB);
        // LDA Indirect,Y has opcode 0xB1.
        nnes.play_test(vec![0xB1, 0x20, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0xAB);
    }

    #[test]
    fn test_brk() {
        let mut nnes = NNES::new();
        // Program: Only BRK (0x00) opcode.
        nnes.play_test(vec![0x00]);
        // After BRK, the Break flag should be set.
        assert_eq!(nnes.get_flag(Flag::Break), true);
    }

    #[test]
    fn test_inx() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::XIndex, 0xff);
        // Program: Set X to 0xff, then INX.
        nnes.play_test(vec![0xe8, 0x00]);
        // When X is 0xff, it wraps to 0.
        assert_eq!(nnes.get_register(Register::XIndex), 0x00);

        // Test normal increment.
        let mut nnes2 = NNES::new();
        nnes2.set_register(Register::XIndex, 0x10);
        nnes2.play_test(vec![0xe8, 0x00]);
        assert_eq!(nnes2.get_register(Register::XIndex), 0x11);
    }

    #[test]
    fn test_sta_zero_page() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0xAB);
        // STA zero page (0x85) at address 0x10, then BRK.
        nnes.play_test(vec![0x85, 0x10, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x10), 0xAB);
    }

    #[test]
    fn test_sta_zero_page_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0xCD);
        nnes.set_register(Register::XIndex, 0x04);
        // STA zero page,X (0x95): base 0x20 + X = 0x20+0x04=0x24.
        nnes.play_test(vec![0x95, 0x20, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x20 + 0x04), 0xCD);
    }

    #[test]
    fn test_sta_absolute() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0xEF);
        // STA absolute (0x8D) to address 0x3000 (0x00, 0x30), then BRK.
        nnes.play_test(vec![0x8d, 0x00, 0x30, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x3000), 0xEF);
    }

    #[test]
    fn test_sta_absolute_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x12);
        nnes.set_register(Register::XIndex, 0x05);
        // STA absolute,X (0x9D): base 0x4000 + X = 0x4005, then BRK.
        nnes.play_test(vec![0x9d, 0x00, 0x40, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x4000 + 0x05), 0x12);
    }

    #[test]
    fn test_sta_absolute_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x34);
        nnes.set_register(Register::YIndex, 0x06);
        // STA absolute,Y (0x99): base 0x5000 + Y = 0x5006, then BRK.
        nnes.play_test(vec![0x99, 0x00, 0x50, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x5000 + 0x06), 0x34);
    }

    #[test]
    fn test_sta_indirect_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x56);
        nnes.set_register(Register::XIndex, 0x03);
        // In indirect,X mode: effective address = word at (pointer + X).
        // Write pointer table at address (0x60 + 0x03) = 0x63.
        // Let effective address be 0x7000 (low=0x00, high=0x70)
        nnes.memory_write_u8(0x60 + 0x03, 0x00); // low byte of effective address
        nnes.memory_write_u8(0x60 + 0x03 + 1, 0x70); // high byte
        // STA indirect,X (0x81). Program loaded at 0x8000:
        // [opcode, pointer_low, pointer_high, BRK] where pointer becomes 0x0060.
        nnes.play_test(vec![0x81, 0x60, 0x00, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x7000), 0x56);
    }

    #[test]
    fn test_sta_indirect_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x78);
        nnes.set_register(Register::YIndex, 0x02);
        // In indirect,Y mode: effective address = (word at pointer) + Y.
        // Set pointer table at address 0x0080 to base 0x2020 (0x20, 0x20), so effective = 0x2020 + 0x02 = 0x2022.
        nnes.memory_write_u8(0x0080, 0x20); // low
        nnes.memory_write_u8(0x0080 + 1, 0x20); // high
        // STA indirect,Y (0x91). Program loaded at 0x8000:
        // [opcode, pointer, BRK] where pointer becomes 0x0080.
        nnes.play_test(vec![0x91, 0x80, 0x00, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x2020 + 0x02), 0x78);
    }

    // Test PHA in normal (no wrap) scenario.
    #[test]
    fn test_pha_normal() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0xAA);
        // Initial SP is 0xff. PHA writes to STACK_OFFSET+0xff and decrements SP.
        nnes.play_test(vec![0x48, 0x00]); // PHA, BRK
        assert_eq!(nnes.memory_read_u8(STACK_OFFSET + 0xff), 0xAA);
        assert_eq!(nnes.get_stack_pointer(), 0xfe);
    }

    // Test PHA when SP is 0 so that it wraps to 0xff.
    #[test]
    fn test_pha_wrap() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x77);
        nnes.set_stack_pointer(0); // simulate stack underflow condition.
        nnes.play_test(vec![0x48, 0x00]); // PHA, BRK
        assert_eq!(nnes.memory_read_u8(STACK_OFFSET + 0), 0x77);
        assert_eq!(nnes.get_stack_pointer(), 0xff);
    }

    // Test PHP in normal (no wrap) scenario.
    #[test]
    fn test_php_normal() {
        let mut nnes = NNES::new();
        nnes.set_flags(0b01101011);
        // Initial SP is 0xff. PHP writes flags at STACK_OFFSET+0xff and decrements SP.
        nnes.play_test(vec![0x08, 0x00]); // PHP, BRK
        assert_eq!(nnes.memory_read_u8(STACK_OFFSET + 0xff), 0b01101011);
        assert_eq!(nnes.get_stack_pointer(), 0xfe);
    }

    // Test PHP when SP is 0 so that it wraps to 0xff.
    #[test]
    fn test_php_wrap() {
        let mut nnes = NNES::new();
        nnes.set_flags(0b11001100);
        nnes.set_stack_pointer(0);
        nnes.play_test(vec![0x08, 0x00]); // PHP, BRK
        assert_eq!(nnes.memory_read_u8(STACK_OFFSET + 0), 0b11001100);
        assert_eq!(nnes.get_stack_pointer(), 0xff);
    }

    // Test a sequence: PHA then PLA restores the Accumulator (normal case).
    #[test]
    fn test_pha_pla_normal() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x55);
        // PHA pushes 0x55 (SP: 0xff -> 0xfe) then PLA pops it back (SP: 0xfe -> 0xff).
        nnes.play_test(vec![0x48, 0x68, 0x00]); // PHA, PLA, BRK
        assert_eq!(nnes.get_register(Register::Accumulator), 0x55);
        assert_eq!(nnes.get_stack_pointer(), 0xff);
    }

    // Test PLA wrapping: if SP is 0xff the PLA resets it to 0 before reading.
    #[test]
    fn test_pla_wrap() {
        let mut nnes = NNES::new();
        nnes.set_stack_pointer(0xff);
        // Manually fill stack location at STACK_OFFSET+0 to simulate a previous push.
        nnes.memory_write_u8(STACK_OFFSET + 0, 0x77);
        nnes.play_test(vec![0x68, 0x00]); // PLA, BRK
        assert_eq!(nnes.get_register(Register::Accumulator), 0x77);
        assert_eq!(nnes.get_stack_pointer(), 0);
    }

    // Test a sequence: PHP then PLP restores the flags (normal case).
    #[test]
    fn test_php_plp_normal() {
        let mut nnes = NNES::new();
        nnes.set_flags(0b00110011);
        // PHP pushes the flags (SP: 0xff -> 0xfe) then PLP pops them back (SP: 0xfe -> 0xff).
        nnes.play_test(vec![0x08, 0x28, 0x00]); // PHP, PLP, BRK
        assert_eq!(nnes.get_flags(), 0b00110011);
        assert_eq!(nnes.get_stack_pointer(), 0xff);
    }

    // Test PLP wrapping: if SP is 0xff then PLP resets it to 0 before reading.
    #[test]
    fn test_plp_wrap() {
        let mut nnes = NNES::new();
        nnes.set_stack_pointer(0xff);
        // Manually fill stack location at STACK_OFFSET+0 to simulate a pushed flags value.
        nnes.memory_write_u8(STACK_OFFSET + 0, 0b01010101);
        nnes.play_test(vec![0x28, 0x00]); // PLP, BRK
        assert_eq!(nnes.get_flags(), 0b01010101);
        assert_eq!(nnes.get_stack_pointer(), 0);
    }

    // Test TAX: Transfer Accumulator to XIndex.
    #[test]
    fn test_tax() {
        let mut nnes = NNES::new();
        // Set Accumulator to a non-zero value then call TAX.
        nnes.play_test(vec![0xa9, 0x5A, 0xaa, 0x00]); // LDA #$5A, TAX, BRK
        assert_eq!(nnes.get_register(Register::XIndex), 0x5A);
    }

    // Test TXA: Transfer XIndex to Accumulator.
    #[test]
    fn test_txa() {
        let mut nnes = NNES::new();
        // Manually set XIndex, then execute TXA.
        nnes.set_register(Register::XIndex, 0x33);
        nnes.play_test(vec![0x8a, 0x00]); // TXA, BRK
        assert_eq!(nnes.get_register(Register::Accumulator), 0x33);
    }

    // Test TSX: Transfer Stack Pointer to XIndex.
    #[test]
    fn test_tsx() {
        let mut nnes = NNES::new();
        // Manually set stack pointer, then execute TSX.
        nnes.set_stack_pointer(0x80);
        nnes.play_test(vec![0xba, 0x00]); // TSX, BRK
        assert_eq!(nnes.get_register(Register::XIndex), 0x80);
    }

    // Test TXS: Transfer XIndex to Stack Pointer.
    #[test]
    fn test_txs() {
        let mut nnes = NNES::new();
        // Manually set XIndex, then execute TXS.
        nnes.set_register(Register::XIndex, 0x8F);
        nnes.play_test(vec![0x9a, 0x00]); // TXS, BRK
        assert_eq!(nnes.get_stack_pointer(), 0x8F);
    }

    // Test TAY: Transfer Accumulator to YIndex.
    #[test]
    fn test_tay() {
        let mut nnes = NNES::new();
        // Use LDA to set Accumulator then execute TAY.
        nnes.play_test(vec![0xa9, 0x77, 0xa8, 0x00]); // LDA #$77, TAY, BRK
        assert_eq!(nnes.get_register(Register::YIndex), 0x77);
    }

    // Test TYA: Transfer YIndex to Accumulator.
    #[test]
    fn test_tya() {
        let mut nnes = NNES::new();
        // Manually set YIndex then execute TYA.
        nnes.set_register(Register::YIndex, 0x99);
        nnes.play_test(vec![0x98, 0x00]); // TYA, BRK
        assert_eq!(nnes.get_register(Register::Accumulator), 0x99);
    }
}
