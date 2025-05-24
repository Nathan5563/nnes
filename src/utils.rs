pub fn add_mod_8(n: u8, m: u8) -> u8 {
    n.wrapping_add(m)
}

pub fn add_mod_16(n: u16, m: u16) -> u16 {
    n.wrapping_add(m)
}

pub fn hi_byte(n: u16) -> u8 {
    (n >> 8) as u8
}

pub fn lo_byte(n: u16) -> u8 {
    (n & 0xFF) as u8
}

pub fn hi_nibble(n: u8) -> u8 {
    n >> 4
}

pub fn lo_nibble(n: u8) -> u8 {
    n & 0xF
}

pub fn bit_7(n: u8) -> u8 {
    n & 0b1000_0000
}

pub fn bit_6(n: u8) -> u8 {
    n & 0b0100_0000
}

pub fn bit_5(n: u8) -> u8 {
    n & 0b0010_0000
}

pub fn bit_4(n: u8) -> u8 {
    n & 0b0001_0000
}

pub fn bit_3(n: u8) -> u8 {
    n & 0b0000_1000
}

pub fn bit_2(n: u8) -> u8 {
    n & 0b0000_0100
}

pub fn bit_1(n: u8) -> u8 {
    n & 0b0000_0010
}

pub fn bit_0(n: u8) -> u8 {
    n & 0b0000_0001
}