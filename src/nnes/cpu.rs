pub mod bus;
mod opcodes;

use crate::utils::{hi_byte, lo_byte};
use bus::Bus;
use opcodes::{opcodes_list, AddressingMode, OpCode};

bitflags! {
    pub struct Flags: u8 {
        const CARRY = 0b0000_0001;
        const ZERO = 0b0000_0010;
        const INTERRUPT_DISABLE = 0b0000_0100;
        const DECIMAL_MODE = 0b0000_1000;
        const BREAK = 0b0001_0000;
        const UNUSED = 0b0010_0000;
        const OVERFLOW = 0b0100_0000;
        const NEGATIVE = 0b1000_0000;
    }
}

// pc is fetched from 0xFFFC/0xFFFD on startup
const RESET_VECTOR: u16 = 0;
// on reset, three pushes occur changing this from 0 to 0xfd
const RESET_SP: u8 = 0xfd;
// unused flag is always on, interrupt disable is turned on for reset sequence
const RESET_FLAGS: Flags = Flags::UNUSED.union(Flags::INTERRUPT_DISABLE);
// reset sequence requires 7 cpu cycles
const RESET_CYCLES: u8 = 7;

// pc is fetched from 0xFFFA/0xFFFB on NMI
const NMI_VECTOR: u16 = 0xFFFA;

// pc is fetched from 0xFFFE/0xFFFF on IRQ/BRK
const IRQ_VECTOR: u16 = 0xFFFE;

// stack page is [0x100, 0x200)
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
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: Flags,

    // Memory bus
    pub bus: Bus,

    // State machine metadata
    pub state: CPUState,
    pub ins: Option<&'static OpCode>,
    pub store: CPUStore,
    pub software_interrupt: bool,
    pub nmi_pending: bool,
    pub irq_pending: bool,

    // Debugging tooling
    pub ins_ticks: i8,
    pub total_ticks: u64,
}

pub struct CPUStore {
    pub lo: u8,
    pub hi: u8,
    pub addr: u16,
    pub data: u8,
    pub offset: i8,
    pub vector: u16,
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
                offset: 0,
                vector: 0,
            },
            software_interrupt: false,
            nmi_pending: false,
            irq_pending: false,

            ins_ticks: 0,
            total_ticks: 0,
        }
    }

    pub fn reset(&mut self) {
        self.store.lo = self.bus.mem_read(RESET_VECTOR);
        self.store.hi = self.bus.mem_read(RESET_VECTOR + 1);
        self.pc = u16::from_le_bytes([self.store.lo, self.store.hi]);
        self.sp = RESET_SP;
        self.p = RESET_FLAGS;
        self.total_ticks = RESET_CYCLES as u64;
    }

    pub fn stack_push(&mut self, data: u8) {
        self.bus.mem_write(STACK_OFFSET + self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.mem_read(STACK_OFFSET + self.sp as u16)
    }

    pub fn stack_peek(&mut self) -> u8 {
        self.bus.mem_read(STACK_OFFSET + self.sp as u16)
    }

    fn fetch(&mut self) {
        self.store.data = self.bus.mem_read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        if let Some(opcode) = opcodes_list[self.store.data as usize].as_ref() {
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
                // Run the cycle
                self.fetch();

                // Poll for interrupts after the second to last cycle
                let mut interrupt = false;
                // if self.ins.unwrap().cycles - self.ins_ticks == 2 {
                //     interrupt = self.poll_interrupts();
                // } else if self.ins.unwrap().code == 0x00
                //     && self.ins.unwrap().cycles - self.ins_ticks > 2
                //     && self.nmi_pending
                // {
                //     // NMI hijacking BRK
                //     self.store.vector = NMI_VECTOR;
                //     self.nmi_pending = false;
                // }

                // Choose next state
                self.state = if interrupt {
                    CPUState::Interrupt
                } else if self.ins.unwrap().decode_fn.is_none() {
                    CPUState::Execute { subcycle: 0 }
                } else {
                    CPUState::Decode { subcycle: 0 }
                };
            }
            CPUState::Decode { subcycle } => {
                // Run the cycle
                let done = (self.ins.unwrap().decode_fn.unwrap())(self, subcycle);

                // Poll for interrupts in the second to last cycle
                let mut interrupt = false;
                // if self.ins.unwrap().cycles - self.ins_ticks == 2 {
                //     interrupt = self.poll_interrupts();
                // }

                // Choose next state
                self.state = if interrupt {
                    CPUState::Interrupt
                } else if done {
                    CPUState::Execute { subcycle: 0 }
                } else {
                    CPUState::Decode {
                        subcycle: subcycle + 1,
                    }
                }
            }
            CPUState::Execute { subcycle } => {
                // Run the cycle
                let done = (self.ins.unwrap().execute_fn)(self, subcycle);

                // Poll for interrupts in the second to last cycle
                let mut interrupt = false;
                // if self.ins.unwrap().cycles - self.ins_ticks == 2 {
                //     interrupt = self.poll_interrupts();
                // } else if self.ins.unwrap().code == 0x00
                //     && self.ins.unwrap().cycles - self.ins_ticks > 2
                //     && self.nmi_pending
                // {
                //     // NMI hijacking BRK
                //     self.store.vector = NMI_VECTOR;
                //     self.nmi_pending = false;
                // }

                // Choose next state
                self.state = if interrupt {
                    CPUState::Interrupt
                } else if done {
                    CPUState::Fetch
                } else {
                    CPUState::Execute {
                        subcycle: subcycle + 1,
                    }
                };

                if done { self.ins_ticks = -1 };
            }
            CPUState::Interrupt => {
                // turn off appropriate signals
                // note that an NMI can hijack an IRQ, and if so, the irq_pending should remain true
            }
        }

        self.ins_ticks += 1;
        self.total_ticks += 1;
    }

    fn trace(&mut self) {
        self.store.data = self.bus.peek(self.pc);
        
        let ins;
        if let Some(opcode) = opcodes_list[self.store.data as usize].as_ref() {
            ins = Some(opcode);
        } else {
            println!("op: {:02x}", self.store.data);
            unimplemented!();
        }

        let mut buf = String::new();

        buf.push_str(format!("{:04X}", self.pc).as_str());
        buf.push_str("  ");

        buf.push_str(format!("{:02X}", ins.unwrap().code).as_str());
        buf.push(' ');

        let mode = ins.unwrap().mode;
        let instruction = ins.unwrap().name.as_str();
        let num_bytes = ins.unwrap().bytes;
        let mut lo = 0;
        let mut hi = 0;
        let mut asm = String::from(instruction);
        asm.push(' ');
        match mode {
            AddressingMode::IMP => {
                asm.push_str("                            ");
            }
            AddressingMode::ACC => {
                asm.push('A');
                asm.push_str("                           ");
            }
            AddressingMode::IMM => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("#${:02X}", lo).as_str());
                asm.push_str("                        ");
            }
            AddressingMode::ZPG => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("${:02X} ", lo).as_str());

                let addr = self.bus.peek(lo as u16);
                asm.push_str(format!("= {:02X}", addr).as_str());
                asm.push_str("                    ");
            }
            AddressingMode::ZPX => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("${:02X},X ", lo).as_str());

                self.store.addr = self.x.wrapping_add(lo) as u16;
                asm.push_str(format!("@ {:02X} ", self.store.addr).as_str());

                self.store.data = self.bus.peek(self.store.addr);
                asm.push_str(format!("= {:02X}", self.store.data).as_str());
                asm.push_str("             ");
            }
            AddressingMode::ZPY => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("${:02X},Y ", lo).as_str());

                self.store.addr = self.y.wrapping_add(lo) as u16;
                asm.push_str(format!("@ {:02X} ", self.store.addr).as_str());

                self.store.data = self.bus.peek(self.store.addr);
                asm.push_str(format!("= {:02X}", self.store.data).as_str());
                asm.push_str("             ");
            }
            AddressingMode::ABS => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                hi = self.bus.peek(self.pc.wrapping_add(2));
                self.store.addr = u16::from_le_bytes([lo, hi]);
                asm.push_str(format!("${:04X} ", self.store.addr).as_str());

                if instruction != "JMP" && instruction != "JSR" {
                    self.store.data = self.bus.peek(self.store.addr);
                    asm.push_str(format!("= {:02X}", self.store.data).as_str());
                    asm.push_str("                  ");
                } else {
                    asm.push_str("                      ");
                }
            }
            AddressingMode::ABX => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                hi = self.bus.peek(self.pc.wrapping_add(2));
                self.store.addr = u16::from_le_bytes([lo, hi]);
                asm.push_str(format!("${:04X},X ", self.store.addr).as_str());

                self.store.addr = self.store.addr.wrapping_add(self.x as u16);
                asm.push_str(format!("@ {:04X} ", self.store.addr).as_str());

                self.store.data = self.bus.peek(self.store.addr);
                asm.push_str(format!("= {:02X}", self.store.data).as_str());
                asm.push_str("         ");
            }
            AddressingMode::ABY => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                hi = self.bus.peek(self.pc.wrapping_add(2));
                self.store.addr = u16::from_le_bytes([lo, hi]);
                asm.push_str(format!("${:04X},Y ", self.store.addr).as_str());

                self.store.addr = self.store.addr.wrapping_add(self.y as u16);
                asm.push_str(format!("@ {:04X} ", self.store.addr).as_str());

                self.store.data = self.bus.peek(self.store.addr);
                asm.push_str(format!("= {:02X}", self.store.data).as_str());
                asm.push_str("         ");
            }
            AddressingMode::IND => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                hi = self.bus.peek(self.pc.wrapping_add(2));
                let indirect = u16::from_le_bytes([lo, hi]);
                asm.push_str(format!("(${:04X}) ", indirect).as_str());

                self.store.addr = indirect;
                self.store.lo = self.bus.peek(indirect);
                self.store.addr = u16::from_le_bytes([
                    lo_byte(self.store.addr.wrapping_add(1)),
                    hi_byte(self.store.addr),
                ]);
                self.store.hi = self.bus.mem_read(self.store.addr);
                self.store.addr = u16::from_le_bytes([self.store.lo, self.store.hi]);
                asm.push_str(format!("= {:04X}", self.store.addr).as_str());
                asm.push_str("              ");
            }
            AddressingMode::INX => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("(${:02X},X) ", lo).as_str());

                let indexed = self.x.wrapping_add(lo);
                asm.push_str(format!("@ {:02X} ", indexed).as_str());

                self.store.lo = self.bus.peek(indexed as u16);
                self.store.hi = self.bus.peek(indexed.wrapping_add(1) as u16);
                self.store.addr = u16::from_le_bytes([self.store.lo, self.store.hi]);
                asm.push_str(format!("= {:04X} ", self.store.addr).as_str());

                self.store.data = self.bus.peek(self.store.addr);
                asm.push_str(format!("= {:02X}", self.store.data).as_str());
                asm.push_str("    ");
            }
            AddressingMode::INY => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("(${:02X}),Y ", lo).as_str());

                self.store.lo = self.bus.peek(lo as u16);
                self.store.hi = self.bus.peek(lo.wrapping_add(1) as u16);
                let indirect = u16::from_le_bytes([self.store.lo, self.store.hi]);
                asm.push_str(format!("= {:04X} ", indirect).as_str());

                self.store.addr = indirect.wrapping_add(self.y as u16);
                asm.push_str(format!("@ {:04X} ", self.store.addr).as_str());

                self.store.data = self.bus.peek(self.store.addr);
                asm.push_str(format!("= {:02X}", self.store.data).as_str());
                asm.push_str("  ");
            }
            AddressingMode::REL => {
                lo = self.bus.peek(self.pc.wrapping_add(1));
                self.store.offset = lo as i8;
                self.store.addr = (self.pc as i32 + 2 + self.store.offset as i32) as u16;
                asm.push_str(format!("${:04X}", self.store.addr as u16).as_str());
                asm.push_str("                       ");
            }
        }
        if num_bytes > 1 {
            buf.push_str(format!("{:02X}", lo).as_str());
            buf.push(' ');
            if num_bytes > 2 {
                buf.push_str(format!("{:02X}", hi).as_str());
                buf.push_str("  ");
            } else {
                buf.push_str("    ");
            }
        } else {
            buf.push_str("       ");
        }

        buf.push_str(asm.as_str());

        buf.push_str(
            format!(
                "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                self.a, self.x, self.y, self.p.bits(), self.sp, self.total_ticks
            )
            .as_str(),
        );
        println!("{buf}");
    }

    pub fn step(&mut self, dbg: bool) {
        if dbg {
            self.trace();
        }
        self.tick();
        while self.ins_ticks > 0 {
            self.tick();
        }
    }
}
