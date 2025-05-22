pub mod bus;
mod opcodes;

use crate::utils::{add_mod_16, hi_byte, lo_byte};
use bus::Bus;
use opcodes::{opcodes_list, AddressingMode, OpCode};

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

// pc is fetched from 0xFFFC/0xFFFD on startup
const RESET_VECTOR: u16 = 0;
// on reset, three pushes occur changing this from 0 to 0xfd
const RESET_SP: u8 = 0xfd;
// unused flag is always on, interrupt disable is turned on for reset sequence
const RESET_FLAGS: Flags = Flags::UNUSED.union(Flags::INTERRUPT_DISABLE);
// reset sequence requires 7 cpu cycles
const RESET_CYCLES: u8 = 7;

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
    pub page_crossed: bool,
    pub software_interrupt: bool,
    pub nmi_pending: bool,
    pub irq_pending: bool,

    // Debugging tooling
    pub ins_ticks: u8,
    pub total_ticks: u64,
}

pub struct CPUStore {
    pub lo: u8,
    pub hi: u8,
    pub addr: u16,
    pub data: u8,
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

    fn fetch(&mut self) {
        let data = self.bus.mem_read(self.pc);
        self.pc = self.pc.wrapping_add(1);

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
                // Currently assumes every instruction starts here, decide for interrupts below
                self.ins_ticks = 0;

                // Run the cycle
                self.fetch();

                // Poll for interrupts after the second to last cycle
                let mut interrupt = false;
                if self.ins.unwrap().cycles - self.ins_ticks == 2 {
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

                // Poll for interrupts in the second to last cycle
                let mut interrupt = false;
                if self.ins.unwrap().cycles - self.ins_ticks == 2 {
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

                // Poll for interrupts in the second to last cycle
                let mut interrupt = false;
                if self.ins.unwrap().cycles - self.ins_ticks == 2 {
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

    fn trace(&mut self) {
        let data = self.bus.peek(self.pc);

        let ins;
        if let Some(opcode) = opcodes_list[data as usize].as_ref() {
            ins = Some(opcode);
        } else {
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
        let mut byte1 = 0;
        let mut byte2 = 0;
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
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("#${:02X}", byte1).as_str());
                asm.push_str("                        ");
            }
            AddressingMode::ZPG => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("${:02X} ", byte1).as_str());

                let addr = self.bus.peek(byte1 as u16);
                asm.push_str(format!("= {:02X}", addr).as_str());
                asm.push_str("                    ");
            }
            AddressingMode::ZPX => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("${:02X},X ", byte1).as_str());

                let zp_addr_x = self.x.wrapping_add(byte1);
                asm.push_str(format!("@ {:02X} ", zp_addr_x).as_str());

                let data = self.bus.peek(zp_addr_x as u16);
                asm.push_str(format!("= {:02X}", data).as_str());
                asm.push_str("             ");
            }
            AddressingMode::ZPY => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("${:02X},Y ", byte1).as_str());

                let zp_addr_y = self.y.wrapping_add(byte1);
                asm.push_str(format!("@ {:02X} ", zp_addr_y).as_str());

                let data = self.bus.peek(zp_addr_y as u16);
                asm.push_str(format!("= {:02X}", data).as_str());
                asm.push_str("             ");
            }
            AddressingMode::ABS => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                byte2 = self.bus.peek(self.pc.wrapping_add(2));
                let addr = u16::from_le_bytes([byte1, byte2]);
                asm.push_str(format!("${:04X} ", addr).as_str());

                if instruction != "JMP" && instruction != "JSR" {
                    let data = self.bus.peek(addr);
                    asm.push_str(format!("= {:02X}", data).as_str());
                    asm.push_str("                  ");
                } else {
                    asm.push_str("                      ");
                }
            }
            AddressingMode::ABX => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                byte2 = self.bus.peek(self.pc.wrapping_add(2));
                let base_addr = u16::from_le_bytes([byte1, byte2]);
                asm.push_str(format!("${:04X},X ", base_addr).as_str());

                let addr_x = add_mod_16(base_addr, self.x as u16);
                asm.push_str(format!("@ {:04X} ", addr_x).as_str());

                let data = self.bus.peek(addr_x);
                asm.push_str(format!("= {:02X}", data).as_str());
                asm.push_str("         ");
            }
            AddressingMode::ABY => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                byte2 = self.bus.peek(self.pc.wrapping_add(2));
                let base_addr = u16::from_le_bytes([byte1, byte2]);
                asm.push_str(format!("${:04X},Y ", base_addr).as_str());

                let addr_y = add_mod_16(base_addr, self.y as u16);
                asm.push_str(format!("@ {:04X} ", addr_y).as_str());

                let data = self.bus.peek(addr_y);
                asm.push_str(format!("= {:02X}", data).as_str());
                asm.push_str("         ");
            }
            AddressingMode::IND => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                byte2 = self.bus.peek(self.pc.wrapping_add(2));
                let indirect = u16::from_le_bytes([byte1, byte2]);
                asm.push_str(format!("(${:04X}) ", indirect).as_str());

                let addrl: u8 = self.bus.peek(indirect);
                let addrh: u8;
                if lo_byte(indirect) == 0xff {
                    addrh = self.bus.peek(hi_byte(indirect) as u16);
                } else {
                    addrh = self.bus.peek(indirect + 1);
                }
                let addr: u16 = u16::from_le_bytes([addrl, addrh]);
                asm.push_str(format!("= {:04X}", addr).as_str());
                asm.push_str("              ");
            }
            AddressingMode::INX => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("(${:02X},X) ", byte1).as_str());
                let indirect = self.x.wrapping_add(byte1);
                asm.push_str(format!("@ {:02X} ", indirect).as_str());

                let addrl = self.bus.peek(indirect as u16);
                let addrh = self.bus.peek(indirect.wrapping_add(1) as u16);
                let addr = u16::from_le_bytes([addrl, addrh]);
                asm.push_str(format!("= {:04X} ", addr).as_str());

                let data = self.bus.peek(addr);
                asm.push_str(format!("= {:02X}", data).as_str());
                asm.push_str("    ");
            }
            AddressingMode::INY => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                asm.push_str(format!("(${:02X}),Y ", byte1).as_str());

                let addrl: u8 = self.bus.peek(byte1 as u16);
                let addrh: u8 = self.bus.peek(byte1.wrapping_add(1) as u16);
                let addr: u16 = u16::from_le_bytes([addrl, addrh]);
                asm.push_str(format!("= {:04X} ", addr).as_str());

                let indexed: u16 = add_mod_16(addr, self.y as u16);
                asm.push_str(format!("@ {:04X} ", indexed).as_str());

                let data: u8 = self.bus.peek(indexed);
                asm.push_str(format!("= {:02X}", data).as_str());
                asm.push_str("  ");
            }
            AddressingMode::REL => {
                byte1 = self.bus.peek(self.pc.wrapping_add(1));
                let offset: i8 = byte1 as i8;
                let res: i32 = self.pc as i32 + offset as i32 + 2;
                asm.push_str(format!("${:04X}", res as u16).as_str());
                asm.push_str("                       ");
            }
        }
        if num_bytes > 1 {
            buf.push_str(format!("{:02X}", byte1).as_str());
            buf.push(' ');
            if num_bytes > 2 {
                buf.push_str(format!("{:02X}", byte2).as_str());
                buf.push_str("  ");
            } else {
                buf.push_str("    ");
            }
        } else {
            buf.push_str("       ");
        }

        buf.push_str(asm.as_str());

        let reg_acc: u8 = self.a;
        let reg_x: u8 = self.x;
        let reg_y: u8 = self.y;
        let flags: u8 = self.p.bits();
        let sp: u8 = self.sp;

        buf.push_str(
            format!(
                "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                reg_acc, reg_x, reg_y, flags, sp
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
