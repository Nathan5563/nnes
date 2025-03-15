#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

#[macro_use]
extern crate lazy_static;

mod nnes;
use nnes::{AddressingMode, Flag, Register, NNES};

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lda_immediate() {
        let mut nnes = NNES::new();
        // Program: LDA immediate (0xa9), operand, then BRK.
        nnes.play_test(vec![0xa9, 0x42, 0x00]);
        assert_eq!(nnes.get_register(Register::ACCUMULATOR), 0x42);
    }

    #[test]
    fn test_lda_zero_page() {
        let mut nnes = NNES::new();
        // Program: LDA zero page (0xa5), operand 0x10, then BRK.
        nnes.load(vec![0xa5, 0x10, 0x00]);
        // Set memory at zero page address 0x10.
        nnes.memory_write(0x10, 0x55);
        nnes.run();
        assert_eq!(nnes.get_register(Register::ACCUMULATOR), 0x55);
    }

    #[test]
    fn test_lda_zero_page_x() {
        let mut nnes = NNES::new();
        // Program: LDA zero page,X (0xb5), operand 0x10, then BRK.
        nnes.load(vec![0xb5, 0x10, 0x00]);
        // Set register X and value at (operand + X)
        nnes.set_register(Register::XIndex, 0x05);
        nnes.memory_write(0x10 + 0x05, 0x66);
        nnes.run();
        assert_eq!(nnes.get_register(Register::ACCUMULATOR), 0x66);
    }

    #[test]
    fn test_lda_absolute() {
        let mut nnes = NNES::new();
        // Program: LDA absolute (0xad), operand low/high for 0x1234, then BRK.
        nnes.load(vec![0xad, 0x34, 0x12, 0x00]);
        nnes.memory_write(0x1234, 0x77);
        nnes.run();
        assert_eq!(nnes.get_register(Register::ACCUMULATOR), 0x77);
    }

    #[test]
    fn test_lda_absolute_x() {
        let mut nnes = NNES::new();
        // Program: LDA absolute,X (0xbd), operands for base address 0x1200, then BRK.
        nnes.load(vec![0xbd, 0x00, 0x12, 0x00]);
        nnes.set_register(Register::XIndex, 0x05);
        nnes.memory_write(0x1200 + 0x05, 0x88);
        nnes.run();
        assert_eq!(nnes.get_register(Register::ACCUMULATOR), 0x88);
    }

    #[test]
    fn test_lda_absolute_y() {
        let mut nnes = NNES::new();
        // Program: LDA absolute,Y (0xb9), operands for base address 0x1200, then BRK.
        nnes.load(vec![0xb9, 0x00, 0x12, 0x00]);
        nnes.set_register(Register::YIndex, 0x06);
        nnes.memory_write(0x1200 + 0x06, 0x99);
        nnes.run();
        assert_eq!(nnes.get_register(Register::ACCUMULATOR), 0x99);
    }

    #[test]
    fn test_lda_indirect_x() {
        let mut nnes = NNES::new();
        // We'll call handle_lda directly with AddressingMode::IndirectX.
        // Prepare program at 0x8000: two bytes for the pointer.
        nnes.load(vec![0x00, 0x01, 0x00]); // dummy program; opcode isn't used here.
                                           // Set register X to offset.
        nnes.set_register(Register::XIndex, 0x04);
        // Place the two pointer bytes at 0x8000.
        // When handle_indirect is called it reads two bytes: low, high => pointer.
        nnes.memory_write(0x8000, 0x00);
        nnes.memory_write(0x8001, 0x01); // pointer = 0x0100.
                                         // In the XIndex branch, effective pointer = (0x0100 + X).
                                         // Now at address (0x0100 + 0x04) put the low/high of final address.
        nnes.memory_write(0x0100 + 0x04, 0x00);
        nnes.memory_write(0x0100 + 0x04 + 1, 0x02); // final address = 0x0200.
                                                    // At final address 0x0200, store the value.
        nnes.memory_write(0x0200, 0xAA);
        // Directly call LDA with IndirectX addressing mode.
        nnes.handle_lda(AddressingMode::IndirectX);
        assert_eq!(nnes.get_register(Register::ACCUMULATOR), 0xAA);
    }

    #[test]
    fn test_lda_indirect_y() {
        let mut nnes = NNES::new();
        // Call handle_lda with AddressingMode::IndirectY.
        nnes.load(vec![0x00, 0x01, 0x00]);
        // Set register Y.
        nnes.set_register(Register::YIndex, 0x03);
        // At 0x8000, write pointer bytes.
        nnes.memory_write(0x8000, 0x00);
        nnes.memory_write(0x8001, 0x01); // pointer = 0x0100.
                                         // In IndirectY branch, the base address is read from 0x0100 and 0x0101.
        nnes.memory_write(0x0100, 0x00);
        nnes.memory_write(0x0101, 0x03); // base address = 0x0300.
                                         // Final effective address = 0x0300 + Y (0x03) = 0x0303.
        nnes.memory_write(0x0303, 0xBB);
        nnes.handle_lda(AddressingMode::IndirectY);
        assert_eq!(nnes.get_register(Register::ACCUMULATOR), 0xBB);
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
    fn test_tax() {
        let mut nnes = NNES::new();
        // Program: LDA immediate then TAX.
        nnes.play_test(vec![0xa9, 0x55, 0xaa, 0x00]);
        // TAX copies accumulator into X.
        assert_eq!(nnes.get_register(Register::XIndex), 0x55);
    }

    #[test]
    fn test_inx() {
        let mut nnes = NNES::new();
        // Program: Set X to 0xff, then INX.
        nnes.load(vec![0xe8, 0x00]);
        nnes.set_register(Register::XIndex, 0xff);
        nnes.run();
        // When X is 0xff, it wraps to 0.
        assert_eq!(nnes.get_register(Register::XIndex), 0x00);

        // Test normal increment.
        let mut nnes2 = NNES::new();
        nnes2.load(vec![0xe8, 0x00]);
        nnes2.set_register(Register::XIndex, 0x10);
        nnes2.run();
        assert_eq!(nnes2.get_register(Register::XIndex), 0x11);
    }

    #[test]
    fn test_sta_zero_page() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 0xAB);
        // STA zero page (0x85) at address 0x10, then BRK.
        nnes.load(vec![0x85, 0x10, 0x00]);
        nnes.run();
        assert_eq!(nnes.memory_read(0x10), 0xAB);
    }

    #[test]
    fn test_sta_zero_page_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 0xCD);
        nnes.set_register(Register::XIndex, 0x04);
        // STA zero page,X (0x95): base 0x20 + X = 0x20+0x04=0x24.
        nnes.load(vec![0x95, 0x20, 0x00]);
        nnes.run();
        assert_eq!(nnes.memory_read(0x20 + 0x04), 0xCD);
    }

    #[test]
    fn test_sta_absolute() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 0xEF);
        // STA absolute (0x8D) to address 0x3000 (0x00, 0x30), then BRK.
        nnes.load(vec![0x8d, 0x00, 0x30, 0x00]);
        nnes.run();
        assert_eq!(nnes.memory_read(0x3000), 0xEF);
    }

    #[test]
    fn test_sta_absolute_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 0x12);
        nnes.set_register(Register::XIndex, 0x05);
        // STA absolute,X (0x9D): base 0x4000 + X = 0x4005, then BRK.
        nnes.load(vec![0x9d, 0x00, 0x40, 0x00]);
        nnes.run();
        assert_eq!(nnes.memory_read(0x4000 + 0x05), 0x12);
    }

    #[test]
    fn test_sta_absolute_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 0x34);
        nnes.set_register(Register::YIndex, 0x06);
        // STA absolute,Y (0x99): base 0x5000 + Y = 0x5006, then BRK.
        nnes.load(vec![0x99, 0x00, 0x50, 0x00]);
        nnes.run();
        assert_eq!(nnes.memory_read(0x5000 + 0x06), 0x34);
    }

    #[test]
    fn test_sta_indirect_x() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 0x56);
        nnes.set_register(Register::XIndex, 0x03);
        // STA indirect,X (0x81). Program loaded at 0x8000:
        // [opcode, pointer_low, pointer_high, BRK] where pointer becomes 0x0060.
        nnes.load(vec![0x81, 0x60, 0x00, 0x00]);
        // In indirect,X mode: effective address = word at (pointer + X).
        // Write pointer table at address (0x60 + 0x03) = 0x63.
        // Let effective address be 0x7000 (low=0x00, high=0x70)
        nnes.memory_write(0x60 + 0x03, 0x00); // low byte of effective address
        nnes.memory_write(0x60 + 0x03 + 1, 0x70); // high byte
        nnes.run();
        assert_eq!(nnes.memory_read(0x7000), 0x56);
    }

    #[test]
    fn test_sta_indirect_y() {
        let mut nnes = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 0x78);
        nnes.set_register(Register::YIndex, 0x02);
        // STA indirect,Y (0x91). Program loaded at 0x8000:
        // [opcode, pointer, BRK] where pointer becomes 0x0080.
        nnes.load(vec![0x91, 0x80, 0x00, 0x00]);
        // In indirect,Y mode: effective address = (word at pointer) + Y.
        // Set pointer table at address 0x0080 to base 0x2020 (0x20, 0x20), so effective = 0x2020 + 0x02 = 0x2022.
        nnes.memory_write(0x0080, 0x20); // low
        nnes.memory_write(0x0080 + 1, 0x20); // high
        nnes.run();
        assert_eq!(nnes.memory_read(0x2020 + 0x02), 0x78);
    }
}
