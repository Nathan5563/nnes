pub mod bus;
mod opcodes;

use bus::Bus;
use opcodes::{opcodes_list, OpCode};

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

// usually reset vector in 0xFFFC/0xFFFD
const RESET_PC: u16 = 0;
// on reset, three pushes occur changing this from 0 to 0xfd
const RESET_SP: u8 = 0xfd;
const RESET_P: Flags = Flags::UNUSED.union(Flags::INTERRUPT_DISABLE);

const NMI_VECTOR: u16 = 0xFFFA;
const IRQ_VECTOR: u16 = 0xFFFE;

const STACK_OFFSET: u16 = 0x100;

#[derive(Debug, Copy, Clone)]
pub enum CPUState {
    Fetch,
    Decode { subcycle: u8 },
    Execute { subcycle: u8 },
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

    // Memory bus
    bus: Bus,

    // State machine metadata
    state: CPUState,
    ins: Option<&'static OpCode>,
    cache: CPUCache,
    page_crossed: bool,
    software_interrupt: bool,
    nmi_pending: bool,
    irq_pending: bool,

    // Debugging tooling
    total_ticks: u64,
}

pub struct CPUCache {
    lo: u8,
    hi: u8,
    addr: u16,
    vector: u16,
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        CPU {
            pc: 0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            p: Flags::UNUSED,

            bus,

            state: CPUState::Fetch,
            ins: None,
            cache: CPUCache {
                lo: 0,
                hi: 0,
                addr: 0,
                vector: 0,
            },
            page_crossed: false,
            software_interrupt: false,
            nmi_pending: false,
            irq_pending: false,

            total_ticks: 0,
        }
    }

    pub fn reset(&mut self) {}

    pub fn stack_push(&mut self, data: u8) {
        self.bus.mem_write(STACK_OFFSET + self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.mem_read(STACK_OFFSET + self.sp as u16)
    }

    pub fn tick(&mut self) {
        match self.state {
            CPUState::Fetch => {
                let data = self.bus.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);

                if let Some(opcode) = opcodes_list[data as usize].as_ref() {
                    self.ins = Some(opcode);
                } else {
                    unimplemented!();
                }

                if self.ins.unwrap().code == 0 {
                    self.cache.vector = IRQ_VECTOR;
                    self.software_interrupt = true;
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
                    self.state = CPUState::Decode {
                        subcycle: subcycle + 1,
                    };
                }
            }
            CPUState::Execute { subcycle } => {
                let done = (self.ins.unwrap().execute_fn)(self, subcycle);
                if done {
                    if self.nmi_pending {
                        self.state = CPUState::Interrupt;
                        self.cache.vector = NMI_VECTOR;
                    } else if self.irq_pending && !self.p.intersects(Flags::INTERRUPT_DISABLE) {
                        self.state = CPUState::Interrupt;
                        self.cache.vector = IRQ_VECTOR;
                    } else {
                        self.state = CPUState::Fetch;
                    }
                } else {
                    self.state = CPUState::Execute {
                        subcycle: subcycle + 1,
                    };
                }
            }
            CPUState::Interrupt => {

            }
        }
        self.total_ticks += 1;
    }
}
