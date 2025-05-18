mod opcodes;

use opcodes::{opcodes_list, OpCode};
use super::bus::Bus;

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
    Decode { subcycle: u8 },
    Execute { subcycle: u8 },
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

    pub fn tick(&mut self, bus: &mut Bus) {
        match self.state {
            CPUState::Fetch => {
                let data = Bus::mem_read(bus, self.pc);
                self.pc += 1;

                if let Some(opcode) = opcodes_list[data as usize].as_ref() {
                    self.ins = Some(opcode);
                } else {
                    unimplemented!();
                }
                
                if self.ins.unwrap().decode_fn.is_none() {
                    self.state = CPUState::Execute { subcycle: 0 };
                } else {
                    self.state = CPUState::Decode { subcycle: 0 };
                }
            }
            CPUState::Decode { subcycle } => {
                let done = (self.ins.unwrap().decode_fn.unwrap())(self, subcycle);
                if done {
                    self.state = CPUState::Execute { subcycle: 0 };
                } else {
                    self.state = CPUState::Decode { subcycle: subcycle + 1 };
                }
            }
            CPUState::Execute { subcycle } => { // 1 - m cycle(s)
                let done = (self.ins.unwrap().execute_fn)(self, subcycle);
                if done {
                    self.state = CPUState::Fetch;
                } else {
                    self.state = CPUState::Execute { subcycle: subcycle + 1 };
                }
            }
        }
        self.total_ticks += 1;
    }
}
