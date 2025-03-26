pub mod flags;
pub mod interrupts;
pub mod opcodes;
pub mod registers;

use crate::nnes::cpu::opcodes::opcodes_map;
use crate::nnes::memory::{AddressingMode, Mem};
use crate::nnes::NNES;
use crate::utils::add_mod_16bit;

impl NNES {
    pub fn cpu_tick(&mut self, cycle: &mut u8, exit: &mut bool) {
        let pc: u16 = self.get_program_counter();
        let code: u8 = self.memory_read_u8(pc);
        self.set_program_counter(add_mod_16bit(pc, 1));
        let ins = opcodes_map
            .get(&code)
            .expect(&format!("OpCode {:x} is not recognized", code));
        let mode: AddressingMode = ins.get_addressing_mode();
        let handler: &fn(&mut NNES, mode: AddressingMode, cycle: &mut u8) = ins.get_handler();

        handler(self, mode, cycle);

        if ins.get_instruction() == "BRK" {
            *exit = true;
        }
    }
}
