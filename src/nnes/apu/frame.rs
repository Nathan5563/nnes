// NTSC frame counter step points, in CPU cycles since sequence start.
// Quarter clocks drive envelopes and the triangle's linear counter; half
// clocks additionally drive length counters and sweeps.
const STEP_1: u32 = 7457;
const STEP_2: u32 = 14913;
const STEP_3: u32 = 22371;
const MODE_0_IRQ: u32 = 29828;
const MODE_0_STEP_4: u32 = 29829;
const MODE_0_END: u32 = 29830;
const MODE_1_STEP_5: u32 = 37281;
const MODE_1_END: u32 = 37282;

pub struct FrameCounter {
    mode5: bool,
    pub irq_inhibit: bool,
    pub irq_flag: bool,
    // Set on the exact cycle the flag is asserted: a $4015 read on that
    // cycle sees the flag but does not clear it
    pub irq_set_now: bool,
    cycles: u32,
    pending_write: Option<u8>,
    write_delay: u8,
}

impl FrameCounter {
    pub fn new() -> Self {
        FrameCounter {
            mode5: false,
            irq_inhibit: false,
            irq_flag: false,
            irq_set_now: false,
            cycles: 0,
            pending_write: None,
            write_delay: 0,
        }
    }

    pub fn reset(&mut self) {
        // The $4017 mode survives reset; the sequence restarts
        self.irq_flag = false;
        self.irq_set_now = false;
        self.cycles = 0;
        self.pending_write = None;
    }

    // $4017: the IRQ inhibit applies immediately, but the sequencer reset
    // lands 3 or 4 CPU cycles later depending on write-cycle parity
    pub fn write(&mut self, data: u8, odd_cycle: bool) {
        self.irq_inhibit = data & 0b0100_0000 != 0;
        if self.irq_inhibit {
            self.irq_flag = false;
        }
        self.pending_write = Some(data);
        self.write_delay = if odd_cycle { 3 } else { 4 };
    }

    fn set_irq(&mut self) {
        if !self.irq_inhibit {
            self.irq_flag = true;
            self.irq_set_now = true;
        }
    }

    // Advance one CPU cycle; returns (quarter, half) clock pulses
    pub fn tick(&mut self) -> (bool, bool) {
        self.irq_set_now = false;

        if let Some(data) = self.pending_write {
            self.write_delay -= 1;
            if self.write_delay == 0 {
                self.pending_write = None;
                self.mode5 = data & 0b1000_0000 != 0;
                self.cycles = 0;
                if self.mode5 {
                    // Selecting 5-step mode clocks quarter and half
                    // immediately, at the point the delayed reset lands
                    return (true, true);
                }
                return (false, false);
            }
        }

        self.cycles += 1;
        if self.mode5 {
            match self.cycles {
                STEP_1 | STEP_3 => (true, false),
                STEP_2 | MODE_1_STEP_5 => (true, true),
                MODE_1_END => {
                    self.cycles = 0;
                    (false, false)
                }
                _ => (false, false),
            }
        } else {
            match self.cycles {
                STEP_1 | STEP_3 => (true, false),
                STEP_2 => (true, true),
                MODE_0_IRQ => {
                    self.set_irq();
                    (false, false)
                }
                MODE_0_STEP_4 => {
                    self.set_irq();
                    (true, true)
                }
                MODE_0_END => {
                    // Hardware re-asserts the flag on this cycle too, but
                    // CPU reads here sample end-of-cycle state and already
                    // see the sticky flag; asserting again would make the
                    // flag's last set read one cycle late
                    self.cycles = 0;
                    (false, false)
                }
                _ => (false, false),
            }
        }
    }
}
