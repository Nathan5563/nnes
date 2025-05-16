#[derive(Debug, Copy, Clone)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

#[derive(Debug, Copy, Clone)]
pub enum Register {
    Accumulator,
    XIndex,
    YIndex,
}

#[derive(Debug, Copy, Clone)]
pub enum Flag {
    Carry,
    Zero,
    InterruptDisable,
    DecimalMode,
    Break,
    Overflow,
    Negative,
}