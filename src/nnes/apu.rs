mod channels;
mod dmc;
mod frame;
mod io;
mod units;

use channels::{Noise, Pulse, Triangle};
use dmc::DMC;
use frame::FrameCounter;

pub struct APU {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: DMC,
    frame: FrameCounter,
    cpu_cycle: u64,
}

impl APU {
    pub fn new() -> Self {
        APU {
            // Pulse 1's sweep negates with ones' complement
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: DMC::new(),
            frame: FrameCounter::new(),
            cpu_cycle: 0,
        }
    }

    // Reset silences everything, as if $4015 = 0 were written; the frame
    // counter mode survives but its sequence restarts
    pub fn reset(&mut self) {
        self.pulse1.length.set_enabled(false);
        self.pulse2.length.set_enabled(false);
        self.triangle.length.set_enabled(false);
        self.noise.length.set_enabled(false);
        self.dmc.set_enabled(false);
        self.dmc.irq_flag = false;
        self.frame.reset();
    }

    // Ticked once per CPU cycle: the triangle timer and the frame counter
    // run at CPU rate, everything else divides down from here
    pub fn tick(&mut self) {
        let (quarter, half) = self.frame.tick();
        if quarter {
            self.pulse1.clock_quarter();
            self.pulse2.clock_quarter();
            self.triangle.clock_quarter();
            self.noise.clock_quarter();
        }
        if half {
            self.pulse1.clock_half();
            self.pulse2.clock_half();
            self.triangle.clock_half();
            self.noise.clock_half();
        }
        self.dmc.clock_timer();
        self.cpu_cycle = self.cpu_cycle.wrapping_add(1);
    }

    // Level of the IRQ line: held while the frame or DMC interrupt flag
    // is set
    pub fn irq_line(&self) -> bool {
        self.frame.irq_flag || self.dmc.irq_flag
    }

    // DMC sample fetches are serviced by the NNES struct, which owns
    // access to the CPU bus
    pub fn take_dmc_request(&mut self) -> Option<u16> {
        self.dmc.request.take()
    }

    pub fn dmc_load(&mut self, data: u8) {
        self.dmc.load(data);
    }

    // Drained by the SDL frontend once audio output lands
    #[allow(dead_code)]
    pub fn take_samples(&mut self) -> Vec<f32> {
        Vec::new()
    }
}
