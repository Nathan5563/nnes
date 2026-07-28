// Note lengths are indexed by bits [7,3] of the length-load registers
pub const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, //
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct Envelope {
    start: bool,
    divider: u8,
    decay: u8,
    period: u8,
    pub loop_flag: bool,
    constant: bool,
}

impl Envelope {
    pub fn new() -> Self {
        Envelope {
            start: false,
            divider: 0,
            decay: 0,
            period: 0,
            loop_flag: false,
            constant: false,
        }
    }

    // Bits [5,0] of $4000/$4004/$400C: loop, constant volume, period
    pub fn write(&mut self, data: u8) {
        self.loop_flag = data & 0b0010_0000 != 0;
        self.constant = data & 0b0001_0000 != 0;
        self.period = data & 0b1111;
    }

    pub fn restart(&mut self) {
        self.start = true;
    }

    pub fn clock_quarter(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = self.period;
        } else if self.divider == 0 {
            self.divider = self.period;
            if self.decay > 0 {
                self.decay -= 1;
            } else if self.loop_flag {
                self.decay = 15;
            }
        } else {
            self.divider -= 1;
        }
    }

    // Consumed by the channel output stage once the sequencers land
    #[allow(dead_code)]
    pub fn output(&self) -> u8 {
        if self.constant {
            self.period
        } else {
            self.decay
        }
    }
}

pub struct LengthCounter {
    counter: u8,
    pub halt: bool,
    enabled: bool,
}

impl LengthCounter {
    pub fn new() -> Self {
        LengthCounter {
            counter: 0,
            halt: false,
            enabled: false,
        }
    }

    // Disabling forces the counter to 0 and blocks loads until re-enabled;
    // enabling has no immediate effect
    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
        if !on {
            self.counter = 0;
        }
    }

    pub fn load(&mut self, index: u8) {
        if self.enabled {
            self.counter = LENGTH_TABLE[(index & 0b11111) as usize];
        }
    }

    pub fn clock_half(&mut self) {
        if self.counter > 0 && !self.halt {
            self.counter -= 1;
        }
    }

    pub fn active(&self) -> bool {
        self.counter > 0
    }
}

pub struct Sweep {
    enabled: bool,
    period: u8,
    negate: bool,
    shift: u8,
    reload: bool,
    divider: u8,
    // Pulse 1 negates with ones' complement, pulse 2 with two's complement
    ones_complement: bool,
}

impl Sweep {
    pub fn new(ones_complement: bool) -> Self {
        Sweep {
            enabled: false,
            period: 0,
            negate: false,
            shift: 0,
            reload: false,
            divider: 0,
            ones_complement,
        }
    }

    // $4001/$4005: enable, divider period, negate, shift count
    pub fn write(&mut self, data: u8) {
        self.enabled = data & 0b1000_0000 != 0;
        self.period = (data >> 4) & 0b111;
        self.negate = data & 0b0000_1000 != 0;
        self.shift = data & 0b111;
        self.reload = true;
    }

    fn target(&self, period: u16) -> i32 {
        let change = (period >> self.shift) as i32;
        if self.negate {
            let sub = if self.ones_complement {
                change + 1
            } else {
                change
            };
            (period as i32 - sub).max(0)
        } else {
            period as i32 + change
        }
    }

    // Muting applies continuously, regardless of the enable flag or the
    // divider state
    pub fn muting(&self, period: u16) -> bool {
        period < 8 || self.target(period) > 0x7FF
    }

    pub fn clock_half(&mut self, period: &mut u16) {
        if self.divider == 0
            && self.enabled
            && self.shift != 0
            && !self.muting(*period)
        {
            *period = self.target(*period) as u16;
        }
        if self.divider == 0 || self.reload {
            self.divider = self.period;
            self.reload = false;
        } else {
            self.divider -= 1;
        }
    }
}
