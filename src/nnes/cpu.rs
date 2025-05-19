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
    store: CPUStore,
    page_crossed: bool,
    software_interrupt: bool,
    nmi_pending: bool,
    irq_pending: bool,

    // Debugging tooling
    ins_ticks: u8,
    total_ticks: u64,
}

pub struct CPUStore {
    lo: u8,
    hi: u8,
    addr: u16,
    data: u8,
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
            store: CPUStore {
                lo: 0,
                hi: 0,
                addr: 0,
                data: 0,
                vector: 0,
            },
            page_crossed: false,
            software_interrupt: false,
            nmi_pending: false,
            irq_pending: false,

            ins_ticks: 0,
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

    fn fetch(&mut self) {
        let data = self.bus.mem_read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        // For implied, immediate, and accumulator reads
        self.store.addr = self.pc;

        if let Some(opcode) = opcodes_list[data as usize].as_ref() {
            self.ins = Some(opcode);
        } else {
            unimplemented!();
        }

        if self.ins.unwrap().code == 0x00 {
            self.store.vector = IRQ_VECTOR;
            self.software_interrupt = true;
        }
    }

    fn poll_interrupts(&mut self) -> bool {
        if self.nmi_pending {
            self.store.vector = NMI_VECTOR;
            true
        } else if self.irq_pending && !self.p.intersects(Flags::INTERRUPT_DISABLE) {
            self.store.vector = IRQ_VECTOR;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self) {
        match self.state {
            CPUState::Fetch => {
                // Reset instruction counter
                // Currently assumes every instruction starts here, reset for interrupts below
                self.ins_ticks = 0;

                // Run the cycle
                self.fetch();

                let mut interrupt = false;
                if self.ins.unwrap().cycles - self.ins_ticks == 2 {
                    // Poll for interrupts after the second to last cycle
                    interrupt = self.poll_interrupts();
                } else if self.ins.unwrap().code == 0x00
                    && self.ins.unwrap().cycles - self.ins_ticks > 2
                    && self.nmi_pending
                {
                    // NMI hijacking BRK
                    self.store.vector = NMI_VECTOR;
                    self.nmi_pending = false;
                }

                // Choose next state
                if interrupt {
                    self.state = CPUState::Interrupt;
                } else {
                    self.state = if self.ins.unwrap().decode_fn.is_none() {
                        CPUState::Execute { subcycle: 0 }
                    } else {
                        CPUState::Decode { subcycle: 0 }
                    };
                }
            }
            CPUState::Decode { subcycle } => {
                // Run the cycle
                let done = (self.ins.unwrap().decode_fn.unwrap())(self, subcycle);

                let mut interrupt = false;
                if self.ins.unwrap().cycles - self.ins_ticks == 2 {
                    // Poll for interrupts in the second to last cycle
                    interrupt = self.poll_interrupts();
                }

                // Choose next state
                if interrupt {
                    self.state = CPUState::Interrupt;
                } else {
                    self.state = if done {
                        CPUState::Execute { subcycle: 0 }
                    } else {
                        CPUState::Decode {
                            subcycle: subcycle + 1,
                        }
                    };
                }
            }
            CPUState::Execute { subcycle } => {
                // Run the cycle
                let done = (self.ins.unwrap().execute_fn)(self, subcycle);

                let mut interrupt = false;
                if self.ins.unwrap().cycles - self.ins_ticks == 2 {
                    // Poll for interrupts in the second to last cycle
                    interrupt = self.poll_interrupts();
                } else if self.ins.unwrap().code == 0x00
                    && self.ins.unwrap().cycles - self.ins_ticks > 2
                    && self.nmi_pending
                {
                    // NMI hijacking BRK
                    self.store.vector = NMI_VECTOR;
                    self.nmi_pending = false;
                }

                // Choose next state
                if interrupt {
                    self.state = CPUState::Interrupt;
                } else {
                    self.state = if done {
                        CPUState::Fetch
                    } else {
                        CPUState::Execute {
                            subcycle: subcycle + 1,
                        }
                    };
                }
            }
            CPUState::Interrupt => {
                // turn off appropriate signals
                // note that an NMI can hijack an IRQ, and if so, the irq_pending should remain true
            }
        }

        self.ins_ticks += 1;
        self.total_ticks += 1;
    }

    pub fn step(&mut self) {
        self.tick();
        while self.ins_ticks > 0 {
            self.tick();
        }
    }
}
