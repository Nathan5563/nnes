use sdl2::{event::Event, keyboard::Keycode};

pub struct Joypad {
    pub active: u8,
    pub state: u8,
}

impl Joypad {
    pub fn update_state(&mut self, event: &Event) {
        if let sdl2::event::Event::KeyDown {
            keycode: Some(key), ..
        } = event
        {
            match key {
                &Keycode::D => self.active |= 0b0000_0001,
                &Keycode::A => self.active |= 0b0000_0010,
                &Keycode::S => self.active |= 0b0000_0100,
                &Keycode::W => self.active |= 0b0000_1000,
                &Keycode::U => self.active |= 0b0001_0000, // A
                &Keycode::I => self.active |= 0b0010_0000, // B
                &Keycode::J => self.active |= 0b0100_0000, // Start
                &Keycode::K => self.active |= 0b1000_0000, // Select
                _ => {}
            }
        }
        if let sdl2::event::Event::KeyUp { 
            keycode: Some(key), ..
        } = event
        {
            match key {
                &Keycode::D => self.active &= !0b0000_0001,
                &Keycode::A => self.active &= !0b0000_0010,
                &Keycode::S => self.active &= !0b0000_0100,
                &Keycode::W => self.active &= !0b0000_1000,
                &Keycode::U => self.active &= !0b0001_0000, // A
                &Keycode::I => self.active &= !0b0010_0000, // B
                &Keycode::J => self.active &= !0b0100_0000, // Start
                &Keycode::K => self.active &= !0b1000_0000, // Select
                _ => {}
            }
        }
    }
}
