use crate::types::{LOWER_BYTE, LOWER_WORD};

pub fn add_mod_8bit(num1: u8, num2: u8) -> u8 {
    ((num1 as u16 + num2 as u16) & LOWER_BYTE) as u8
}

pub fn add_mod_16bit(num1: u16, num2: u16) -> u16 {
    ((num1 as u32 + num2 as u32) & LOWER_WORD) as u16
}