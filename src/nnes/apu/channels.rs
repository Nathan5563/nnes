use super::units::{Envelope, LengthCounter, Sweep};

pub struct Pulse {
    pub period: u16,
    pub envelope: Envelope,
    pub sweep: Sweep,
    pub length: LengthCounter,
}

impl Pulse {
    pub fn new(ones_complement: bool) -> Self {
        Pulse {
            period: 0,
            envelope: Envelope::new(),
            sweep: Sweep::new(ones_complement),
            length: LengthCounter::new(),
        }
    }

    pub fn clock_quarter(&mut self) {
        self.envelope.clock_quarter();
    }

    pub fn clock_half(&mut self) {
        self.sweep.clock_half(&mut self.period);
        self.length.clock_half();
    }
}

pub struct Triangle {
    pub period: u16,
    linear_counter: u8,
    pub linear_reload: u8,
    pub linear_reload_flag: bool,
    pub control: bool,
    pub length: LengthCounter,
}

impl Triangle {
    pub fn new() -> Self {
        Triangle {
            period: 0,
            linear_counter: 0,
            linear_reload: 0,
            linear_reload_flag: false,
            control: false,
            length: LengthCounter::new(),
        }
    }

    pub fn clock_quarter(&mut self) {
        if self.linear_reload_flag {
            self.linear_counter = self.linear_reload;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        // The reload flag persists while the control flag is set
        if !self.control {
            self.linear_reload_flag = false;
        }
    }

    pub fn clock_half(&mut self) {
        self.length.clock_half();
    }
}

// NTSC noise timer periods, in CPU cycles
pub const NOISE_PERIODS: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub struct Noise {
    pub mode: bool,
    pub period: u16,
    pub envelope: Envelope,
    pub length: LengthCounter,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            mode: false,
            period: 0,
            envelope: Envelope::new(),
            length: LengthCounter::new(),
        }
    }

    pub fn clock_quarter(&mut self) {
        self.envelope.clock_quarter();
    }

    pub fn clock_half(&mut self) {
        self.length.clock_half();
    }
}
