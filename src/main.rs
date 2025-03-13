#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod nnes;
use nnes::*;

fn main()
{

}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn t0x00_brk() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Break) == true);
    }

    #[test]
    fn t0xa9_lda_immediate() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0xa9, 0x05, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::ACCUMULATOR) == 0x05);
        assert!(nnes.get_flag(Flag::Zero) == false);
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0xa9_lda_zero_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0xa9, 0x00, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Zero) == true);
        nnes.reset();
        nnes.load(vec![0xa9, 0x80, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Zero) == false);
    }

    #[test]
    fn t0xa9_lda_negative_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0xa9, 0x80, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == true);
        nnes.reset();
        nnes.load(vec![0xa9, 0x7F, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0xaa_tax_implied() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 10);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 10);
    }

    #[test]
    fn t0xaa_tax_zero_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 0);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Zero) == true);
        nnes.reset();
        nnes.set_register(Register::ACCUMULATOR, 128);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Zero) == false);
    }

    #[test]
    fn t0xaa_tax_negative_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::ACCUMULATOR, 128);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == true);
        nnes.reset();
        nnes.set_register(Register::ACCUMULATOR, 127);
        nnes.load(vec![0xaa, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0xe8_inx_implied() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::XIndex, 10);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 11);
    }

    #[test]
    fn t0xe8_inx_overflow() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::XIndex, 0xfe);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == true);
        nnes.reset();
        nnes.set_register(Register::XIndex, 0xff);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 0);
        assert!(nnes.get_flag(Flag::Zero) == true);
        assert!(nnes.get_flag(Flag::Negative) == false);
        nnes.reset();
        nnes.set_register(Register::XIndex, 0xff);
        nnes.load(vec![0xe8, 0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 1);
        assert!(nnes.get_flag(Flag::Zero) == false);
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0xe8_inx_negative_flag() {
        let mut nnes: NNES = NNES::new();
        nnes.set_register(Register::XIndex, 127);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == true);
        nnes.reset();
        nnes.set_register(Register::XIndex, 126);
        nnes.load(vec![0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_flag(Flag::Negative) == false);
    }

    #[test]
    fn t0x00_t0xa9_immediate_t0xaa_implied_t0xe8_implied() {
        let mut nnes: NNES = NNES::new();
        nnes.load(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        nnes.interpret();
        assert!(nnes.get_register(Register::XIndex) == 0xc1);
        assert!(nnes.get_flag(Flag::Zero) == false);
        assert!(nnes.get_flag(Flag::Negative) == true);
    }    
}
