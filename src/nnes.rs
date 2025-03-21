pub mod cpu;

static PROGRAM_START_ADDR: u16 = 0xfffc;
static PROGRAM_START_PC: u16 = 0x8000;
static GAME_PROGRAM_START_PC: u16 = 0x0600; // new for game code

pub struct NNES {
    program_counter: u16,
    stack_pointer: u8,
    reg_accumulator: u8,
    reg_xindex: u8,
    reg_yindex: u8,
    flags: u8,
    memory: [u8; 0x10000],
}

impl NNES {
    pub fn new() -> Self {
        NNES {
            program_counter: 0,
            stack_pointer: 0xff,
            reg_accumulator: 0,
            reg_xindex: 0,
            reg_yindex: 0,
            flags: 0b00100100,
            memory: [0; 0x10000],
        }
    }

    // Existing load method for tests
    pub fn load(&mut self, program: Vec<u8>) {
        let mut idx = 0;
        for data in program {
            self.memory_write_u8(PROGRAM_START_PC + idx, data);
            idx += 1;
        }
        self.memory_write_u16(PROGRAM_START_ADDR, PROGRAM_START_PC);
    }

    // New method for game code expecting start at 0x0600
    pub fn load_game(&mut self, program: Vec<u8>) {
        let mut idx = 0;
        for data in program {
            self.memory_write_u8(GAME_PROGRAM_START_PC + idx, data);
            idx += 1;
        }
        self.memory_write_u16(PROGRAM_START_ADDR, GAME_PROGRAM_START_PC);
    }

    pub fn reset_state(&mut self) {
        self.reset_registers();
        self.reset_flags();
        self.set_program_counter(self.memory_read_u16(PROGRAM_START_ADDR));
        self.set_stack_pointer(0xff);
    }

    pub fn run(&mut self) {
        self.run_callback(|_| {});
    }

    pub fn run_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut NNES),
    {
        let mut exit: bool = false;
        while !exit {
            self.step(&mut exit);
            callback(self);
        }
    }

    pub fn play(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset_state();
        self.run();
    }

    pub fn play_test(&mut self, program: Vec<u8>) {
        self.load(program);
        self.set_program_counter(self.memory_read_u16(PROGRAM_START_ADDR));
        self.run();
    }
}
