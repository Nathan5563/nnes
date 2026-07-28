use super::units::{Envelope, LengthCounter, Sweep};

// Output waveforms per duty setting, read with an upward index
const DUTY: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

pub struct Pulse {
    pub duty: u8,
    pub seq_pos: u8,
    timer: u16,
    pub period: u16,
    pub envelope: Envelope,
    pub sweep: Sweep,
    pub length: LengthCounter,
}

impl Pulse {
    pub fn new(ones_complement: bool) -> Self {
        Pulse {
            duty: 0,
            seq_pos: 0,
            timer: 0,
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

    // Clocked every other CPU cycle (APU rate)
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            self.seq_pos = (self.seq_pos + 1) % 8;
        } else {
            self.timer -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.length.active()
            || self.sweep.muting(self.period)
            || DUTY[self.duty as usize][self.seq_pos as usize] == 0
        {
            0
        } else {
            self.envelope.output()
        }
    }
}

const TRI_SEQ: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, //
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

pub struct Triangle {
    timer: u16,
    seq_pos: u8,
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
            timer: 0,
            seq_pos: 0,
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

    // Clocked every CPU cycle; the sequencer only advances while both the
    // linear counter and length counter are nonzero, and holds its output
    // otherwise. Ultrasonic periods are held too, to keep the pops the
    // real DAC would smear out of the decimated stream.
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            if self.linear_counter > 0
                && self.length.active()
                && self.period >= 2
            {
                self.seq_pos = (self.seq_pos + 1) % 32;
            }
        } else {
            self.timer -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        TRI_SEQ[self.seq_pos as usize]
    }
}

// NTSC noise timer periods, in CPU cycles
pub const NOISE_PERIODS: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub struct Noise {
    pub mode: bool,
    timer: u16,
    lfsr: u16,
    pub period: u16,
    pub envelope: Envelope,
    pub length: LengthCounter,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            mode: false,
            timer: 0,
            lfsr: 1,
            period: NOISE_PERIODS[0],
            envelope: Envelope::new(),
            length: LengthCounter::new(),
        }
    }

    // Clocked every CPU cycle (the period table is in CPU cycles)
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            let tap = if self.mode { 6 } else { 1 };
            let feedback = (self.lfsr ^ (self.lfsr >> tap)) & 1;
            self.lfsr = (self.lfsr >> 1) | (feedback << 14);
        } else {
            self.timer -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if self.lfsr & 1 != 0 || !self.length.active() {
            0
        } else {
            self.envelope.output()
        }
    }

    pub fn clock_quarter(&mut self) {
        self.envelope.clock_quarter();
    }

    pub fn clock_half(&mut self) {
        self.length.clock_half();
    }
}
