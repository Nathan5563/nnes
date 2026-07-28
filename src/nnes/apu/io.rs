use super::channels::NOISE_PERIODS;
use super::APU;

impl APU {
    pub fn reg_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 => {
                self.pulse1.envelope.write(data);
                self.pulse1.length.halt = data & 0b0010_0000 != 0;
            }
            0x4001 => self.pulse1.sweep.write(data),
            0x4002 => {
                self.pulse1.period =
                    (self.pulse1.period & 0x700) | data as u16;
            }
            0x4003 => {
                self.pulse1.period = (self.pulse1.period & 0x0FF)
                    | ((data as u16 & 0b111) << 8);
                self.pulse1.length.load(data >> 3);
                self.pulse1.envelope.restart();
            }
            0x4004 => {
                self.pulse2.envelope.write(data);
                self.pulse2.length.halt = data & 0b0010_0000 != 0;
            }
            0x4005 => self.pulse2.sweep.write(data),
            0x4006 => {
                self.pulse2.period =
                    (self.pulse2.period & 0x700) | data as u16;
            }
            0x4007 => {
                self.pulse2.period = (self.pulse2.period & 0x0FF)
                    | ((data as u16 & 0b111) << 8);
                self.pulse2.length.load(data >> 3);
                self.pulse2.envelope.restart();
            }
            0x4008 => {
                self.triangle.control = data & 0b1000_0000 != 0;
                self.triangle.length.halt = data & 0b1000_0000 != 0;
                self.triangle.linear_reload = data & 0b0111_1111;
            }
            0x400A => {
                self.triangle.period =
                    (self.triangle.period & 0x700) | data as u16;
            }
            0x400B => {
                self.triangle.period = (self.triangle.period & 0x0FF)
                    | ((data as u16 & 0b111) << 8);
                self.triangle.length.load(data >> 3);
                self.triangle.linear_reload_flag = true;
            }
            0x400C => {
                self.noise.envelope.write(data);
                self.noise.length.halt = data & 0b0010_0000 != 0;
            }
            0x400E => {
                self.noise.mode = data & 0b1000_0000 != 0;
                self.noise.period = NOISE_PERIODS[(data & 0b1111) as usize];
            }
            0x400F => {
                self.noise.length.load(data >> 3);
                self.noise.envelope.restart();
            }
            0x4015 => {
                self.pulse1.length.set_enabled(data & 0b0001 != 0);
                self.pulse2.length.set_enabled(data & 0b0010 != 0);
                self.triangle.length.set_enabled(data & 0b0100 != 0);
                self.noise.length.set_enabled(data & 0b1000 != 0);
            }
            0x4017 => {
                self.frame.write(data, self.cpu_cycle % 2 == 1);
            }
            _ => {}
        }
    }

    // Only $4015 is readable ($4000-$4014 short-circuit to open bus at the
    // bus level); bit 5 is filled in from open bus by the caller
    pub fn reg_read(&mut self, addr: u16) -> u8 {
        if addr != 0x4015 {
            return 0;
        }
        let data = self.status();
        // Reading clears the frame IRQ flag, unless the flag is being set
        // on this very cycle, in which case it reads as set and survives
        if !self.frame.irq_set_now {
            self.frame.irq_flag = false;
        }
        data
    }

    pub fn peek(&self, addr: u16) -> u8 {
        if addr == 0x4015 {
            self.status()
        } else {
            0
        }
    }

    fn status(&self) -> u8 {
        let mut data = 0;
        if self.pulse1.length.active() {
            data |= 0b0001;
        }
        if self.pulse2.length.active() {
            data |= 0b0010;
        }
        if self.triangle.length.active() {
            data |= 0b0100;
        }
        if self.noise.length.active() {
            data |= 0b1000;
        }
        if self.frame.irq_flag {
            data |= 0b0100_0000;
        }
        data
    }
}
