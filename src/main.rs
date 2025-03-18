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

    #[test]
    fn test_and_immediate() {
        let mut nnes = NNES::new();
        // LDA #$CA, then AND #$AA: 0xCA & 0xAA = 0x8A.
        nnes.play_test(vec![0xa9, 0xCA, 0x29, 0xAA, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x8A);
    }

    #[test]
    fn test_and_zero_page() {
        let mut nnes = NNES::new();
        // Set accumulator to 0xCA then AND with value at zero page 0x10 (0xAA): result = 0x8A.
        nnes.set_register(Register::Accumulator, 0xCA);
        nnes.memory_write_u8(0x10, 0xAA);
        nnes.play_test(vec![0x25, 0x10, 0x00]); // AND zero page
        assert_eq!(nnes.get_register(Register::Accumulator), 0x8A);
    }

    #[test]
    fn test_ora_immediate() {
        let mut nnes = NNES::new();
        // LDA #$55, then ORA #$AA: 0x55 | 0xAA = 0xFF.
        nnes.play_test(vec![0xa9, 0x55, 0x09, 0xAA, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0xFF);
    }

    #[test]
    fn test_ora_zero_page() {
        let mut nnes = NNES::new();
        // Set accumulator to 0x55 then OR with value at zero page 0x20 (0xAA): result = 0xFF.
        nnes.set_register(Register::Accumulator, 0x55);
        nnes.memory_write_u8(0x20, 0xAA);
        nnes.play_test(vec![0x05, 0x20, 0x00]); // ORA zero page
        assert_eq!(nnes.get_register(Register::Accumulator), 0xFF);
    }

    #[test]
    fn test_eor_immediate() {
        let mut nnes = NNES::new();
        // LDA #$FF, then EOR #$0F: 0xFF ^ 0x0F = 0xF0.
        nnes.play_test(vec![0xa9, 0xFF, 0x49, 0x0F, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0xF0);
    }

    #[test]
    fn test_eor_zero_page() {
        let mut nnes = NNES::new();
        // Set accumulator to 0x0F then EOR with value at zero page 0x30 (0xFF): 0x0F ^ 0xFF = 0xF0.
        nnes.set_register(Register::Accumulator, 0x0F);
        nnes.memory_write_u8(0x30, 0xFF);
        nnes.play_test(vec![0x45, 0x30, 0x00]); // EOR zero page
        assert_eq!(nnes.get_register(Register::Accumulator), 0xF0);
    }

    // NEW: Tests for LDX instruction
    #[test]
    fn test_ldx_immediate() {
        let mut nnes = NNES::new();
        // LDX immediate: opcode 0xa2
        nnes.play_test(vec![0xa2, 0x55, 0x00]);
        assert_eq!(nnes.get_register(Register::XIndex), 0x55);
    }

    #[test]
    fn test_ldx_zero_page() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x10, 0x66);
        // LDX zero page: opcode 0xa6
        nnes.play_test(vec![0xa6, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::XIndex), 0x66);
    }

    #[test]
    fn test_ldx_zero_page_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::YIndex, 0x03);
        nnes.memory_write_u8(0x10 + 0x03, 0x77);
        // LDX zero page,Y: opcode 0xb6
        nnes.play_test(vec![0xb6, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::XIndex), 0x77);
    }

    #[test]
    fn test_ldx_absolute() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x1234, 0x88);
        // LDX absolute: opcode 0xae, low then high byte of address 0x1234
        nnes.play_test(vec![0xae, 0x34, 0x12, 0x00]);
        assert_eq!(nnes.get_register(Register::XIndex), 0x88);
    }

    #[test]
    fn test_ldx_absolute_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::YIndex, 0x05);
        nnes.memory_write_u8(0x1200 + 0x05, 0x99);
        // LDX absolute,Y: opcode 0xbe, low then high byte of base address 0x1200
        nnes.play_test(vec![0xbe, 0x00, 0x12, 0x00]);
        assert_eq!(nnes.get_register(Register::XIndex), 0x99);
    }

    // NEW: Tests for LDY instruction
    #[test]
    fn test_ldy_immediate() {
        let mut nnes = NNES::new();
        // LDY immediate: opcode 0xa0
        nnes.play_test(vec![0xa0, 0x44, 0x00]);
        assert_eq!(nnes.get_register(Register::YIndex), 0x44);
    }

    #[test]
    fn test_ldy_zero_page() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x20, 0x55);
        // LDY zero page: opcode 0xa4
        nnes.play_test(vec![0xa4, 0x20, 0x00]);
        assert_eq!(nnes.get_register(Register::YIndex), 0x55);
    }

    #[test]
    fn test_ldy_zero_page_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::XIndex, 0x03);
        nnes.memory_write_u8(0x20 + 0x03, 0x66);
        // LDY zero page,X: opcode 0xb4
        nnes.play_test(vec![0xb4, 0x20, 0x00]);
        assert_eq!(nnes.get_register(Register::YIndex), 0x66);
    }

    #[test]
    fn test_ldy_absolute() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x2345, 0x77);
        // LDY absolute: opcode 0xac
        nnes.play_test(vec![0xac, 0x45, 0x23, 0x00]);
        assert_eq!(nnes.get_register(Register::YIndex), 0x77);
    }

    #[test]
    fn test_ldy_absolute_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::XIndex, 0x04);
        nnes.memory_write_u8(0x3000 + 0x04, 0x88);
        // LDY absolute,X: opcode 0xbc
        nnes.play_test(vec![0xbc, 0x00, 0x30, 0x00]);
        assert_eq!(nnes.get_register(Register::YIndex), 0x88);
    }

    // NEW: Tests for memory-mode shift/rotate instructions

    #[test]
    fn test_asl_zero_page() {
        let mut nnes = NNES::new();
        // Write a value with MSB set at address 0x50.
        nnes.memory_write_u8(0x50, 0x80);
        // ASL zero page: opcode 0x06, which shifts left; 0x80 << 1 = 0x00 and carry set.
        nnes.play_test(vec![0x06, 0x50, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x50), 0x00);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }

    #[test]
    fn test_lsr_zero_page() {
        let mut nnes = NNES::new();
        // Write a value with LSB set at address 0x51.
        nnes.memory_write_u8(0x51, 0x01);
        // LSR zero page: opcode 0x46; 0x01 >> 1 = 0x00 and carry set.
        nnes.play_test(vec![0x46, 0x51, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x51), 0x00);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }

    #[test]
    fn test_rol_zero_page() {
        let mut nnes = NNES::new();
        // Write a value at address 0x60.
        nnes.memory_write_u8(0x60, 0x40);
        // Ensure carry is false.
        nnes.set_flag(Flag::Carry, false);
        // ROL zero page: opcode 0x26; 0x40 << 1 = 0x80.
        nnes.play_test(vec![0x26, 0x60, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x60), 0x80);
    }

    #[test]
    fn test_ror_zero_page() {
        let mut nnes = NNES::new();
        // Write a value with LSB set at address 0x70.
        nnes.memory_write_u8(0x70, 0x03);
        // Set carry flag to true.
        nnes.set_flag(Flag::Carry, true);
        // ROR zero page: opcode 0x66; shifting right:
        // Expected: 0x03 >> 1 = 0x01, with previous carry true becomes 0x81.
        nnes.play_test(vec![0x66, 0x70, 0x00]);
        assert_eq!(nnes.memory_read_u8(0x70), 0x81);
        // Also check that the LSB (0x03's bit0) was pushed into carry.
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }

    #[test]
    fn test_asl_accumulator() {
        let mut nnes = NNES::new();
        // Set accumulator to 0x81 (1000_0001). ASL will shift left: result should be 0x02 and carry true.
        nnes.set_register(Register::Accumulator, 0x81);
        println!("ACC Before: {:08b}", nnes.get_register(Register::Accumulator));
        // Opcode for ASL (accumulator mode) is 0x0a, then BRK.
        nnes.play_test(vec![0x0a, 0x00]);
        println!("ACC After: {:08b}", nnes.get_register(Register::Accumulator));
        assert_eq!(nnes.get_register(Register::Accumulator), 0x02);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }

    #[test]
    fn test_lsr_accumulator() {
        let mut nnes = NNES::new();
        // Set accumulator to 0x03 (0000_0011). LSR will shift right: result should be 0x01 and carry true.
        nnes.set_register(Register::Accumulator, 0x03);
        // Opcode for LSR (accumulator mode) is 0x4a, then BRK.
        nnes.play_test(vec![0x4a, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x01);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }

    #[test]
    fn test_rol_accumulator() {
        let mut nnes = NNES::new();
        // Set accumulator to 0x40 (0100_0000) with carry flag false.
        nnes.set_flag(Flag::Carry, false);
        nnes.set_register(Register::Accumulator, 0x40);
        // Opcode for ROL (accumulator mode) is 0x2a, then BRK.
        nnes.play_test(vec![0x2a, 0x00]);
        // 0x40 rotated left becomes 0x80; no carry is inserted.
        assert_eq!(nnes.get_register(Register::Accumulator), 0x80);
        assert_eq!(nnes.get_flag(Flag::Carry), false);
    }

    #[test]
    fn test_ror_accumulator() {
        let mut nnes = NNES::new();
        // Set accumulator to 0x02 (0000_0010) and carry flag true.
        nnes.set_register(Register::Accumulator, 0x02);
        nnes.set_flag(Flag::Carry, true);
        // Opcode for ROR (accumulator mode) is 0x6a, then BRK.
        nnes.play_test(vec![0x6a, 0x00]);
        // 0x02 rotated right with carry set yields 0x81.
        assert_eq!(nnes.get_register(Register::Accumulator), 0x81);
        // Carry flag becomes false (bit0 of 0x02 was 0).
        assert_eq!(nnes.get_flag(Flag::Carry), false);
    }

    // ---------- ADC Instruction Tests ----------

    #[test]
    fn test_adc_immediate_no_carry_no_overflow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x10);
        // 0x10 + 0x20 = 0x30, no carry/overflow.
        nnes.play_test(vec![0x69, 0x20, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x30);
        assert!(!nnes.get_flag(Flag::Carry));
        assert!(!nnes.get_flag(Flag::Overflow));
    }

    #[test]
    fn test_adc_immediate_with_initial_carry() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x10);
        nnes.set_flag(Flag::Carry, true);
        // 0x10 + 0x20 + 1 = 0x31.
        nnes.play_test(vec![0x69, 0x20, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x31);
    }

    #[test]
    fn test_adc_immediate_overflow() {
        let mut nnes = NNES::new();
        // Example: 0x50 + 0x50 = 0xa0 produces signed overflow.
        nnes.set_register(Register::Accumulator, 0x50);
        nnes.play_test(vec![0x69, 0x50, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0xa0);
        assert!(nnes.get_flag(Flag::Overflow));
    }

    #[test]
    fn test_adc_zero_page() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x20);
        // Write value 0x15 at zero page address 0x10.
        nnes.memory_write_u8(0x10, 0x15);
        nnes.play_test(vec![0x65, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x35);
    }

    #[test]
    fn test_adc_zero_page_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x30);
        nnes.set_register(Register::XIndex, 0x05);
        // Write operand at address (0x1b + X=5) => 0x20, value 0x0f.
        nnes.memory_write_u8(0x20, 0x0f);
        nnes.play_test(vec![0x75, 0x1b, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x30 + 0x0f);
    }

    #[test]
    fn test_adc_absolute() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x40);
        // Write operand 0x20 at absolute address 0x1234.
        nnes.memory_write_u8(0x1234, 0x20);
        nnes.play_test(vec![0x6d, 0x34, 0x12, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x60);
    }

    #[test]
    fn test_adc_absolute_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x10);
        nnes.set_register(Register::XIndex, 0x04);
        // Write operand 0x20 at effective address 0x2000 + 0x04 = 0x2004.
        nnes.memory_write_u8(0x2004, 0x20);
        nnes.play_test(vec![0x7d, 0x00, 0x20, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x30);
    }

    #[test]
    fn test_adc_absolute_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x05);
        nnes.set_register(Register::YIndex, 0x03);
        // Write operand 0x0a at effective address 0x3000 + 0x03 = 0x3003.
        nnes.memory_write_u8(0x3003, 0x0a);
        nnes.play_test(vec![0x79, 0x00, 0x30, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x0f);
    }

    #[test]
    fn test_adc_indirect_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x08);
        nnes.set_register(Register::XIndex, 0x04);
        // For ADC Indirect,X (opcode 0x61), operand 0x10.
        // Effective pointer = 0x10 + X = 0x14; pointer at 0x14/0x15 points to 0x4000.
        nnes.memory_write_u8(0x14, 0x00);
        nnes.memory_write_u8(0x15, 0x40);
        // Write operand 0x22 at 0x4000.
        nnes.memory_write_u8(0x4000, 0x22);
        nnes.play_test(vec![0x61, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x08 + 0x22);
    }

    #[test]
    fn test_adc_indirect_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x15);
        nnes.set_register(Register::YIndex, 0x03);
        // For ADC Indirect,Y (opcode 0x71), operand 0x20.
        // Pointer at 0x20 holds base address 0x5000; effective address = 0x5000 + 0x03.
        nnes.memory_write_u8(0x20, 0x00);
        nnes.memory_write_u8(0x21, 0x50);
        nnes.memory_write_u8(0x5003, 0x0a);
        nnes.play_test(vec![0x71, 0x20, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x15 + 0x0a);
    }

    // ---------- SBC Instruction Tests ----------

    #[test]
    fn test_sbc_immediate_no_borrow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x50);
        nnes.set_flag(Flag::Carry, true);
        // SBC Immediate (opcode 0xe9): 0x50 - 0x10 = 0x40.
        nnes.play_test(vec![0xe9, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x40);
        // No borrow: Carry remains set.
        assert!(nnes.get_flag(Flag::Carry));
    }

    #[test]
    fn test_sbc_immediate_with_borrow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x50);
        nnes.set_flag(Flag::Carry, false);
        // SBC Immediate: 0x50 - 0x10 - 1 = 0x3f.
        nnes.play_test(vec![0xe9, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x3f);
    }

    #[test]
    fn test_sbc_immediate_underflow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x05);
        nnes.set_flag(Flag::Carry, true);
        // Underflow: 0x05 - 0x10 wraps to 0xf5.
        nnes.play_test(vec![0xe9, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0xf5);
        // Borrow occurred, so Carry is cleared.
        assert!(!nnes.get_flag(Flag::Carry));
    }

    #[test]
    fn test_sbc_zero_page() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x70);
        nnes.set_flag(Flag::Carry, true);
        nnes.memory_write_u8(0x10, 0x20);
        nnes.play_test(vec![0xe5, 0x10, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x50);
    }

    #[test]
    fn test_sbc_zero_page_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x80);
        nnes.set_flag(Flag::Carry, true);
        nnes.set_register(Register::XIndex, 0x04);
        // Write operand at address (0x30+0x04 = 0x34) with value 0x10.
        nnes.memory_write_u8(0x34, 0x10);
        nnes.play_test(vec![0xf5, 0x30, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x80 - 0x10);
    }

    #[test]
    fn test_sbc_absolute() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x90);
        nnes.set_flag(Flag::Carry, true);
        nnes.memory_write_u8(0x1234, 0x20);
        nnes.play_test(vec![0xed, 0x34, 0x12, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x90 - 0x20);
    }

    #[test]
    fn test_sbc_absolute_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x80);
        nnes.set_flag(Flag::Carry, true);
        nnes.set_register(Register::XIndex, 0x04);
        nnes.memory_write_u8(0x2004, 0x10);
        nnes.play_test(vec![0xfd, 0x00, 0x20, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x80 - 0x10);
    }

    #[test]
    fn test_sbc_absolute_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x70);
        nnes.set_flag(Flag::Carry, true);
        nnes.set_register(Register::YIndex, 0x03);
        nnes.memory_write_u8(0x3003, 0x10);
        nnes.play_test(vec![0xf9, 0x00, 0x30, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x70 - 0x10);
    }

    #[test]
    fn test_sbc_indirect_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x50);
        nnes.set_flag(Flag::Carry, true);
        nnes.set_register(Register::XIndex, 0x03);
        // For SBC Indirect,X (opcode 0xe1) with operand 0x60:
        // Effective pointer = 0x60 + X (0x03) => 0x63; pointer at 0x63/0x64 points to 0x7000.
        nnes.memory_write_u8(0x63, 0x00);
        nnes.memory_write_u8(0x64, 0x70);
        nnes.memory_write_u8(0x7000, 0x20);
        nnes.play_test(vec![0xe1, 0x60, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x50 - 0x20);
    }

    #[test]
    fn test_sbc_indirect_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::Accumulator, 0x60);
        nnes.set_flag(Flag::Carry, true);
        nnes.set_register(Register::YIndex, 0x02);
        // For SBC Indirect,Y (opcode 0xf1), operand 0x80.
        // Pointer at 0x80 holds base address 0x2020; effective = 0x2020 + 0x02.
        nnes.memory_write_u8(0x80, 0x20);
        nnes.memory_write_u8(0x81, 0x20);
        nnes.memory_write_u8(0x2022, 0x10);
        nnes.play_test(vec![0xf1, 0x80, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x60 - 0x10);
    }

    #[test]
    fn test_inc_no_overflow() {
        let mut nnes = NNES::new();
        // Write 0x10 at zero page address 0x40, INC should increment to 0x11.
        nnes.memory_write_u8(0x40, 0x10);
        nnes.play_test(vec![0xe6, 0x40, 0x00]); // INC zero page, BRK
        assert_eq!(nnes.memory_read_u8(0x40), 0x11);
    }
    
    #[test]
    fn test_inc_overflow() {
        let mut nnes = NNES::new();
        // Write 0xff at zero page address 0x50, INC should wrap to 0x00.
        nnes.memory_write_u8(0x50, 0xff);
        nnes.play_test(vec![0xe6, 0x50, 0x00]); // INC zero page, BRK
        assert_eq!(nnes.memory_read_u8(0x50), 0x00);
    }
    
    #[test]
    fn test_iny_no_overflow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::YIndex, 0x10);
        nnes.play_test(vec![0xc8, 0x00]); // INY, BRK; 0x10 -> 0x11
        assert_eq!(nnes.get_register(Register::YIndex), 0x11);
    }
    
    #[test]
    fn test_iny_overflow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::YIndex, 0xff);
        nnes.play_test(vec![0xc8, 0x00]); // INY, BRK; 0xff -> 0x00
        assert_eq!(nnes.get_register(Register::YIndex), 0x00);
    }
    
    #[test]
    fn test_dex_no_overflow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::XIndex, 0x20);
        nnes.play_test(vec![0xca, 0x00]); // DEX, BRK; 0x20 -> 0x1F
        assert_eq!(nnes.get_register(Register::XIndex), 0x1F);
    }
    
    #[test]
    fn test_dex_overflow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::XIndex, 0x00);
        nnes.play_test(vec![0xca, 0x00]); // DEX, BRK; 0x00 -> 0xff
        assert_eq!(nnes.get_register(Register::XIndex), 0xff);
    }
    
    #[test]
    fn test_dey_no_overflow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::YIndex, 0x20);
        nnes.play_test(vec![0x88, 0x00]); // DEY, BRK; 0x20 -> 0x1F
        assert_eq!(nnes.get_register(Register::YIndex), 0x1F);
    }
    
    #[test]
    fn test_dey_overflow() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::YIndex, 0x00);
        nnes.play_test(vec![0x88, 0x00]); // DEY, BRK; 0x00 -> 0xff
        assert_eq!(nnes.get_register(Register::YIndex), 0xff);
    }
    
    #[test]
    fn test_dec_no_underflow() {
        let mut nnes = NNES::new();
        // Write 0x11 at zero page address 0x60, DEC should decrement to 0x10.
        nnes.memory_write_u8(0x60, 0x11);
        nnes.play_test(vec![0xc6, 0x60, 0x00]); // DEC zero page, BRK
        assert_eq!(nnes.memory_read_u8(0x60), 0x10);
    }
    
    #[test]
    fn test_dec_underflow() {
        let mut nnes = NNES::new();
        // Write 0x00 at zero page address 0x70, DEC should wrap to 0xff.
        nnes.memory_write_u8(0x70, 0x00);
        nnes.play_test(vec![0xc6, 0x70, 0x00]); // DEC zero page, BRK
        assert_eq!(nnes.memory_read_u8(0x70), 0xff);
    }

    // New tests for CMP instruction
    #[test]
    fn test_cmp_immediate_equal() {
        let mut nnes = NNES::new();
        // LDA #$55, then CMP #$55, BRK
        nnes.play_test(vec![0xA9, 0x55, 0xC9, 0x55, 0x00]);
        assert_eq!(nnes.get_register(Register::Accumulator), 0x55);
        assert_eq!(nnes.get_flag(Flag::Zero), true);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }
    
    #[test]
    fn test_cmp_immediate_less() {
        let mut nnes = NNES::new();
        // LDA #$40, then CMP #$50, BRK: 0x40 < 0x50 so Carry should be false
        nnes.play_test(vec![0xA9, 0x40, 0xC9, 0x50, 0x00]);
        assert_eq!(nnes.get_flag(Flag::Carry), false);
    }
    
    #[test]
    fn test_cmp_immediate_greater() {
        let mut nnes = NNES::new();
        // LDA #$60, then CMP #$50, BRK: 0x60 >= 0x50 so Carry should be true
        nnes.play_test(vec![0xA9, 0x60, 0xC9, 0x50, 0x00]);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }
    
    #[test]
    fn test_cmp_zero_page() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x10, 0x55);
        // LDA #$55, then CMP (zero page) using opcode 0xC5 at address 0x10, then BRK
        nnes.play_test(vec![0xA9, 0x55, 0xC5, 0x10, 0x00]);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }
    
    // New tests for CPX (cmx) instruction
    #[test]
    fn test_cpx_immediate_equal() {
        let mut nnes = NNES::new();
        // LDX #$30, then CPX immediate (opcode 0xE0) with operand 0x30, then BRK
        nnes.play_test(vec![0xA2, 0x30, 0xE0, 0x30, 0x00]);
        assert_eq!(nnes.get_register(Register::XIndex), 0x30);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }
    
    #[test]
    fn test_cpx_immediate_less() {
        let mut nnes = NNES::new();
        // LDX #$20, then CPX immediate with operand 0x30, BRK: Carry should be false.
        nnes.play_test(vec![0xA2, 0x20, 0xE0, 0x30, 0x00]);
        assert_eq!(nnes.get_flag(Flag::Carry), false);
    }
    
    #[test]
    fn test_cpx_zero_page() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x10, 0x40);
        // LDX #$40, then CPX zero page (opcode 0xE4) at address 0x10, then BRK
        nnes.play_test(vec![0xA2, 0x40, 0xE4, 0x10, 0x00]);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }
    
    #[test]
    fn test_cpx_absolute() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x1234, 0x30);
        // LDX #$30, then CPX absolute (opcode 0xEC) at address 0x1234, then BRK
        nnes.play_test(vec![0xA2, 0x30, 0xEC, 0x34, 0x12, 0x00]);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }
    
    // New tests for CPY (cmy) instruction
    #[test]
    fn test_cpy_immediate_equal() {
        let mut nnes = NNES::new();
        // LDY #$30, then CPY immediate (opcode 0xC0) with operand 0x30, then BRK
        nnes.play_test(vec![0xA0, 0x30, 0xC0, 0x30, 0x00]);
        assert_eq!(nnes.get_register(Register::YIndex), 0x30);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }
    
    #[test]
    fn test_cpy_immediate_less() {
        let mut nnes = NNES::new();
        // LDY #$20, then CPY immediate with operand 0x30, BRK: Carry should be false.
        nnes.play_test(vec![0xA0, 0x20, 0xC0, 0x30, 0x00]);
        assert_eq!(nnes.get_flag(Flag::Carry), false);
    }
    
    #[test]
    fn test_cpy_zero_page() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x10, 0x50);
        // LDY #$50, then CPY zero page (opcode 0xC4) at address 0x10, then BRK
        nnes.play_test(vec![0xA0, 0x50, 0xC4, 0x10, 0x00]);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }
    
    #[test]
    fn test_cpy_absolute() {
        let mut nnes = NNES::new();
        nnes.memory_write_u8(0x1234, 0x30);
        // LDY #$30, then CPY absolute (opcode 0xCC) at address 0x1234, then BRK
        nnes.play_test(vec![0xA0, 0x30, 0xCC, 0x34, 0x12, 0x00]);
        assert_eq!(nnes.get_flag(Flag::Carry), true);
    }
}
