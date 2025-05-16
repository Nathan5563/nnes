mod opcodes;

use opcodes::{opcodes_map, OpCode};

bitflags! {
    struct Flags: u8 {
        const CARRY = 0b0000_0001;
        const ZERO = 0b0000_0010;
        const INTERRUPT_DISABLE = 0b0000_0100;
        const DECIMAL_MODE = 0b0000_1000; // Always 0
        const BREAK = 0b0001_0000;        // Software-defined
        const UNUSED = 0b0010_0000;       // Always 1
        const OVERFLOW = 0b0100_0000;
        const NEGATIVE = 0b1000_0000;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CPUState {
    Fetch,
    Decode,
    Address { subcycle: u8 },
    Execute { subcycle: u8 },
    Complete,
    Interrupt,
}

pub struct CPU {
    // Architectural state
    pc: u16,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    p: Flags,

    // State machine metadata
    state: CPUState,
    ins: Option<&'static OpCode>,
    cache: CPUCache,
    page_crossed: bool,
    nmi_pending: bool,
    irq_pending: bool,

    // Debugging tooling
    total_ticks: u64,
}

pub struct CPUCache {
    lo: u8,
    hi: u8,
    addr: u16,
    data: u8,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            p: Flags::UNUSED | Flags::INTERRUPT_DISABLE,

            state: CPUState::Fetch,
            ins: None,
            cache: CPUCache { lo: 0, hi: 0, addr: 0, data: 0 },
            page_crossed: false,
            nmi_pending: false,
            irq_pending: false,

            total_ticks: 0,
        }
    }

    pub fn tick(&mut self) {
        match self.state {
            CPUState::Fetch => {
                // fetch byte, store in self.cache.lo
                // next: decode
            }
            CPUState::Decode => {
                // decode opcode in self.cache.lo, construct self.ins
                // next: address
            }
            CPUState::Address { subcycle } => {
                // call address handler function
                // next: address OR execute
            }
            CPUState::Execute { subcycle } => {
                // call execute handler function
                // next: execute OR complete
            }
            CPUState::Complete => {
                // check if interrupts are pending
                // next: interrupt OR fetch
            }
            CPUState::Interrupt => {
                // handle interrupts
                // next: fetch
            }
        }
        self.total_ticks += 1;
    }
}
