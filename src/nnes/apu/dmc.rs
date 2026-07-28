// NTSC DMC timer periods, in CPU cycles per output bit
const DMC_RATES: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72,
    54,
];

pub struct DMC {
    irq_enabled: bool,
    loop_flag: bool,
    period: u16,
    timer: u16,
    pub output_level: u8,
    sample_address: u8,
    sample_length: u8,
    current_address: u16,
    pub bytes_remaining: u16,
    sample_buffer: Option<u8>,
    shift_reg: u8,
    bits_remaining: u8,
    silence: bool,
    pub irq_flag: bool,
    // Sample fetches go through the NNES struct, which owns bus access
    pub request: Option<u16>,
}

impl DMC {
    pub fn new() -> Self {
        DMC {
            irq_enabled: false,
            loop_flag: false,
            period: DMC_RATES[0],
            timer: DMC_RATES[0],
            output_level: 0,
            sample_address: 0,
            sample_length: 0,
            current_address: 0xC000,
            bytes_remaining: 0,
            sample_buffer: None,
            shift_reg: 0,
            bits_remaining: 8,
            silence: true,
            irq_flag: false,
            request: None,
        }
    }

    // $4010: IRQ enable, loop, rate index
    pub fn write_ctrl(&mut self, data: u8) {
        self.irq_enabled = data & 0b1000_0000 != 0;
        if !self.irq_enabled {
            self.irq_flag = false;
        }
        self.loop_flag = data & 0b0100_0000 != 0;
        self.period = DMC_RATES[(data & 0b1111) as usize];
    }

    // $4011: direct 7-bit load of the output level
    pub fn write_load(&mut self, data: u8) {
        self.output_level = data & 0b0111_1111;
    }

    // $4012: sample address = $C000 + A * 64
    pub fn write_address(&mut self, data: u8) {
        self.sample_address = data;
    }

    // $4013: sample length = L * 16 + 1 bytes
    pub fn write_length(&mut self, data: u8) {
        self.sample_length = data;
    }

    fn restart(&mut self) {
        self.current_address = 0xC000 + self.sample_address as u16 * 64;
        self.bytes_remaining = self.sample_length as u16 * 16 + 1;
    }

    // $4015 bit 4: enabling with no bytes left restarts the sample;
    // disabling stops after the current byte drains
    pub fn set_enabled(&mut self, on: bool) {
        if on {
            if self.bytes_remaining == 0 {
                self.restart();
                self.maybe_request();
            }
        } else {
            self.bytes_remaining = 0;
        }
    }

    fn maybe_request(&mut self) {
        if self.sample_buffer.is_none()
            && self.bytes_remaining > 0
            && self.request.is_none()
        {
            self.request = Some(self.current_address);
        }
    }

    // A fetched byte delivered by the DMA path
    pub fn load(&mut self, data: u8) {
        self.sample_buffer = Some(data);
        self.current_address = if self.current_address == 0xFFFF {
            0x8000
        } else {
            self.current_address + 1
        };
        self.bytes_remaining -= 1;
        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.restart();
            } else if self.irq_enabled {
                self.irq_flag = true;
            }
        }
    }

    pub fn clock_timer(&mut self) {
        self.timer -= 1;
        if self.timer == 0 {
            self.timer = self.period;
            self.clock_output();
        }
    }

    fn clock_output(&mut self) {
        if !self.silence {
            if self.shift_reg & 1 != 0 {
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else if self.output_level >= 2 {
                self.output_level -= 2;
            }
        }
        self.shift_reg >>= 1;
        self.bits_remaining -= 1;
        if self.bits_remaining == 0 {
            // Start of a new output cycle: consume the sample buffer, or
            // go silent if the reader hasn't refilled it
            self.bits_remaining = 8;
            match self.sample_buffer.take() {
                Some(byte) => {
                    self.shift_reg = byte;
                    self.silence = false;
                    self.maybe_request();
                }
                None => self.silence = true,
            }
        }
    }

    #[allow(dead_code)]
    pub fn output(&self) -> u8 {
        self.output_level
    }
}
