// The APU's DAC is nonlinear; these are the standard lookup-table
// approximations of the mixing formulas
lazy_static! {
    static ref PULSE_TABLE: [f32; 31] = {
        let mut t = [0.0; 31];
        for (n, v) in t.iter_mut().enumerate().skip(1) {
            *v = 95.52 / (8128.0 / n as f32 + 100.0);
        }
        t
    };
    static ref TND_TABLE: [f32; 203] = {
        let mut t = [0.0; 203];
        for (n, v) in t.iter_mut().enumerate().skip(1) {
            *v = 163.67 / (24329.0 / n as f32 + 100.0);
        }
        t
    };
}

pub fn mix(p1: u8, p2: u8, t: u8, n: u8, d: u8) -> f32 {
    PULSE_TABLE[(p1 + p2) as usize]
        + TND_TABLE[3 * t as usize + 2 * n as usize + d as usize]
}

const CPU_HZ: f64 = 1_789_772.727;
const SAMPLE_HZ: f64 = 44_100.0;
const CYCLES_PER_SAMPLE: f64 = CPU_HZ / SAMPLE_HZ;
// One-pole ~90 Hz high-pass, matching the console's DC-blocking filter
const HP_COEFF: f32 = 0.987;
// ~1.5 s of audio; if nothing drains the buffer (headless use), drop it
const BUF_CAP: usize = 65_536;

pub struct Sampler {
    frac: f64,
    acc: f32,
    acc_n: u32,
    trim: f64,
    hp_prev_in: f32,
    hp_prev_out: f32,
    pub buf: Vec<f32>,
}

impl Sampler {
    pub fn new() -> Self {
        Sampler {
            frac: 0.0,
            acc: 0.0,
            acc_n: 0,
            trim: 0.0,
            hp_prev_in: 0.0,
            hp_prev_out: 0.0,
            buf: Vec::new(),
        }
    }

    // Small rate adjustment used by the frontend to keep its audio queue
    // depth in band without resampling
    #[allow(dead_code)]
    pub fn set_trim(&mut self, trim: f64) {
        self.trim = trim;
    }

    // Box-average decimation from CPU rate down to the output rate
    pub fn push(&mut self, level: f32) {
        self.acc += level;
        self.acc_n += 1;
        self.frac += 1.0;
        let step = CYCLES_PER_SAMPLE * (1.0 + self.trim);
        if self.frac >= step {
            self.frac -= step;
            let sample = self.acc / self.acc_n as f32;
            self.acc = 0.0;
            self.acc_n = 0;
            let hp = HP_COEFF * (self.hp_prev_out + sample - self.hp_prev_in);
            self.hp_prev_in = sample;
            self.hp_prev_out = hp;
            if self.buf.len() >= BUF_CAP {
                self.buf.clear();
            }
            self.buf.push(hp);
        }
    }
}
