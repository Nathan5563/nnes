use sdl2::keyboard::Scancode;

pub struct Joypad {
    pub active: u8,
    pub state: u8,
}

pub struct Controllers {
    pub pad1: Joypad,
    pub pad2: Joypad,
}

impl Controllers {
    pub fn update_state(&mut self, event_pump: &sdl2::EventPump) {
        let keystate = event_pump.keyboard_state();

        self.pad1.active = 0;

        if keystate.is_scancode_pressed(Scancode::D) {
            self.pad1.active |= 0b0000_0001;
        }
        if keystate.is_scancode_pressed(Scancode::A) {
            self.pad1.active |= 0b0000_0010;
        }
        if keystate.is_scancode_pressed(Scancode::S) {
            self.pad1.active |= 0b0000_0100;
        }
        if keystate.is_scancode_pressed(Scancode::W) {
            self.pad1.active |= 0b0000_1000;
        }
        if keystate.is_scancode_pressed(Scancode::I) {
            self.pad1.active |= 0b0001_0000; // Start
        }
        if keystate.is_scancode_pressed(Scancode::U) {
            self.pad1.active |= 0b0010_0000; // Select
        }
        if keystate.is_scancode_pressed(Scancode::K) {
            self.pad1.active |= 0b0100_0000; // B
        }
        if keystate.is_scancode_pressed(Scancode::J) {
            self.pad1.active |= 0b1000_0000; // A
        }

        self.pad2.active = 0;

        if keystate.is_scancode_pressed(Scancode::Right) {
            self.pad2.active |= 0b0000_0001;
        }
        if keystate.is_scancode_pressed(Scancode::Left) {
            self.pad2.active |= 0b0000_0010;
        }
        if keystate.is_scancode_pressed(Scancode::Down) {
            self.pad2.active |= 0b0000_0100;
        }
        if keystate.is_scancode_pressed(Scancode::Up) {
            self.pad2.active |= 0b0000_1000;
        }
        if keystate.is_scancode_pressed(Scancode::Return) {
            self.pad2.active |= 0b0001_0000; // Start
        }
        if keystate.is_scancode_pressed(Scancode::RShift) {
            self.pad2.active |= 0b0010_0000; // Select
        }
        if keystate.is_scancode_pressed(Scancode::Comma) {
            self.pad2.active |= 0b0100_0000; // B
        }
        if keystate.is_scancode_pressed(Scancode::Period) {
            self.pad2.active |= 0b1000_0000; // A
        }
    }
}
