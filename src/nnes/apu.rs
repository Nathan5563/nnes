pub struct APU {
    // Architectural state arrives with the frame counter and channels;
    // the skeleton only tracks time and the register file
    cpu_cycle: u64,
    regs: [u8; 0x18],
}

impl APU {
    pub fn new() -> Self {
        APU {
            cpu_cycle: 0,
            regs: [0; 0x18],
        }
    }

    pub fn reset(&mut self) {}

    // Ticked once per CPU cycle: the triangle timer and the frame counter
    // run at CPU rate, everything else divides down from here
    pub fn tick(&mut self) {
        self.cpu_cycle = self.cpu_cycle.wrapping_add(1);
    }

    // Level of the IRQ line: held while the frame or DMC interrupt flag
    // is set
    pub fn irq_line(&self) -> bool {
        false
    }

    // DMC sample fetches are serviced by the NNES struct, which owns
    // access to the CPU bus
    pub fn take_dmc_request(&mut self) -> Option<u16> {
        None
    }

    pub fn dmc_load(&mut self, _data: u8) {}

    // Drained by the SDL frontend once audio output lands
    #[allow(dead_code)]
    pub fn take_samples(&mut self) -> Vec<f32> {
        Vec::new()
    }

    pub fn reg_read(&mut self, _addr: u16) -> u8 {
        0
    }

    pub fn reg_write(&mut self, addr: u16, data: u8) {
        if (0x4000..0x4018).contains(&addr) {
            self.regs[(addr - 0x4000) as usize] = data;
        }
    }

    pub fn peek(&self, _addr: u16) -> u8 {
        0
    }
}
