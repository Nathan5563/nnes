use crate::nnes::NNES;

pub enum Flag {
    Carry,
    Zero,
    InterruptDisable,
    DecimalMode,
    Break,
    Overflow,
    Negative,
}

pub static CF: u8 = 0b00000001;
pub static NEG_CF: u8 = 0b11111110;
pub static ZF: u8 = 0b00000010;
pub static NEG_ZF: u8 = 0b11111101;
pub static IF: u8 = 0b00000100;
pub static NEG_IF: u8 = 0b11111011;
pub static DF: u8 = 0b00001000;
pub static NEG_DF: u8 = 0b11110111;
pub static BF: u8 = 0b00010000;
pub static NEG_BF: u8 = 0b11101111;
pub static VF: u8 = 0b01000000;
pub static NEG_VF: u8 = 0b10111111;
pub static NF: u8 = 0b10000000;
pub static NEG_NF: u8 = 0b01111111;

impl NNES {
    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Carry => (self.flags & CF) != 0,
            Flag::Zero => (self.flags & ZF) != 0,
            Flag::InterruptDisable => (self.flags & IF) != 0,
            Flag::DecimalMode => (self.flags & DF) != 0,
            Flag::Break => (self.flags & BF) != 0,
            Flag::Overflow => (self.flags & VF) != 0,
            Flag::Negative => (self.flags & NF) != 0,
        }
    }

    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    pub fn set_flag(&mut self, flag: Flag, status: bool) {
        match flag {
            Flag::Carry => {
                if status {
                    self.flags |= CF;
                } else {
                    self.flags &= NEG_CF;
                }
            }
            Flag::Zero => {
                if status {
                    self.flags |= ZF;
                } else {
                    self.flags &= NEG_ZF;
                }
            }
            Flag::InterruptDisable => {
                if status {
                    self.flags |= IF;
                } else {
                    self.flags &= NEG_IF;
                }
            }
            Flag::DecimalMode => {
                if status {
                    self.flags |= DF;
                } else {
                    self.flags &= NEG_DF;
                }
            }
            Flag::Break => {
                if status {
                    self.flags |= BF;
                } else {
                    self.flags &= NEG_BF;
                }
            }
            Flag::Overflow => {
                if status {
                    self.flags |= VF;
                } else {
                    self.flags &= NEG_VF;
                }
            }
            Flag::Negative => {
                if status {
                    self.flags |= NF;
                } else {
                    self.flags &= NEG_NF;
                }
            }
        }
    }

    pub fn set_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

    pub fn update_op_flags(&mut self, res: u8) {
        if res == 0 {
            self.set_flag(Flag::Zero, true);
        } else {
            self.set_flag(Flag::Zero, false);
        }

        if res & NF != 0 {
            self.set_flag(Flag::Negative, true);
        } else {
            self.set_flag(Flag::Negative, false);
        }
    }

    pub fn reset_flags(&mut self) {
        self.flags = 0;
    }
}
