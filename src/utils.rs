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

pub fn byte_from_nibbles(lo: u8, hi: u8) -> u8 {
    (hi << 4) | lo
}

pub fn byte_from_bits(b7: u8, b6: u8, b5: u8, b4: u8, b3: u8, b2: u8, b1: u8, b0: u8) -> u8 {
    (b7 << 7) | (b6 << 6) | (b5 << 5) | (b4 << 4) | (b3 << 3) | (b2 << 2) | (b1 << 1) | b0
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
