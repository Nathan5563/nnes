pub fn hi_byte(n: u16) -> u8 {
    ((n & 0xFF00) >> 4) as u8
}

pub fn lo_byte(n: u16) -> u8 {
    (n & 0xFF) as u8
}

pub fn hi_nibble(n: u8) -> u8 {
    (n & 0xF0) >> 2
}

pub fn lo_nibble(n: u8) -> u8 {
    n & 0xF
}
