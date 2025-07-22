use sdl2::keyboard::Scancode;

pub struct Joypad {
    pub active: u8,
    pub state: u8,
}

impl Joypad {
    pub fn update_state(&mut self, event_pump: &sdl2::EventPump) {
        let keystate = event_pump.keyboard_state();

        self.active = 0;

        if keystate.is_scancode_pressed(Scancode::D) {
            self.active |= 0b0000_0001;
        }
        if keystate.is_scancode_pressed(Scancode::A) {
            self.active |= 0b0000_0010;
        }
        if keystate.is_scancode_pressed(Scancode::S) {
            self.active |= 0b0000_0100;
        }
        if keystate.is_scancode_pressed(Scancode::W) {
            self.active |= 0b0000_1000;
        }
        if keystate.is_scancode_pressed(Scancode::I) {
            self.active |= 0b0001_0000; // Start
        }
        if keystate.is_scancode_pressed(Scancode::U) {
            self.active |= 0b0010_0000; // Select
        }
        if keystate.is_scancode_pressed(Scancode::K) {
            self.active |= 0b0100_0000; // B
        }
        if keystate.is_scancode_pressed(Scancode::J) {
            self.active |= 0b1000_0000; // A
        }
    }
}
