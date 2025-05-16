mod enums;
mod opcodes;
use opcodes::{InsState, opcodes_map};

pub struct CPU {
    pc: u16,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ins_state: Option<InsState>,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            ins_state: None,
        }
    }

    pub fn flush(&mut self, cpu_state: u64) {
        
    }

    pub fn tick(&mut self) {
        if self.ins_state.is_none() {
            let pc = self.pc;
            // read byte, decode instruction
            let code = 0;
            let ins = opcodes_map
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized.", code));

            self.ins_state = Some(InsState::new(0, ins))
        }

        let ins_state = self.ins_state.as_mut().unwrap();
        let cpu_state = InsState::execute(ins_state);
        self.flush(cpu_state);
    }
}
