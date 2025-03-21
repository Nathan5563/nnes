#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

mod nnes;
mod types;
mod trace;

use nnes::NNES;
use nnes::memory::Mem;
use nnes::bus::Bus;
use nnes::rom::Rom;

use rand::Rng;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::TextureAccess;
use std::time::{Duration, Instant};
use std::env;

static KEY_W: u8 = 0x77;
static KEY_A: u8 = 0x61;
static KEY_S: u8 = 0x73;
static KEY_D: u8 = 0x64;

fn color(byte: u8) -> Color {
    match byte {
        0 => Color::BLACK,
        1 => Color::WHITE,
        2 => Color::RGB(255, 0, 0),       // Red
        3 => Color::RGB(0, 255, 255),     // Cyan
        4 => Color::RGB(128, 0, 128),     // Purple
        5 => Color::RGB(0, 128, 0),       // Green
        6 => Color::RGB(0, 0, 255),       // Blue
        7 => Color::RGB(255, 255, 0),     // Yellow
        8 => Color::RGB(255, 165, 0),     // Orange
        9 => Color::RGB(165, 42, 42),     // Brown
        0xA => Color::RGB(255, 192, 203), // Light Red
        0xB => Color::RGB(64, 64, 64),    // Dark Grey
        0xC => Color::RGB(128, 128, 128), // Grey
        0xD => Color::RGB(144, 238, 144), // Light Green
        0xE => Color::RGB(173, 216, 230), // Light Blue
        0xF => Color::RGB(211, 211, 211), // Light Grey
        _ => Color::BLACK,
    }
}

fn read_screen_state(nnes: &NNES, frame: &mut [u8; 32 * 32 * 3]) -> bool {
    let mut updated = false;
    for (i, addr) in (0x0200..=0x05FF).enumerate() {
        let color_byte: u8 = nnes.memory_read_u8(addr);
        let pixel_color: Color = color(color_byte);
        let idx: usize = i * 3;
        if frame[idx] != pixel_color.r
            || frame[idx + 1] != pixel_color.g
            || frame[idx + 2] != pixel_color.b
        {
            frame[idx] = pixel_color.r;
            frame[idx + 1] = pixel_color.g;
            frame[idx + 2] = pixel_color.b;
            updated = true;
        }
    }
    updated
}

fn handle_user_input(nnes: &mut NNES, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => {
                nnes.memory_write_u8(0xff, KEY_W);
            }
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                nnes.memory_write_u8(0xff, KEY_A);
            }
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => {
                nnes.memory_write_u8(0xff, KEY_S);
            }
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => {
                nnes.memory_write_u8(0xff, KEY_D);
            }
            _ => {}
        }
    }
}

fn main() {
    let sdl_context: sdl2::Sdl = sdl2::init().unwrap();

    let video_subsystem: sdl2::VideoSubsystem = sdl_context.video().unwrap();
    let mut event_pump: EventPump = sdl_context.event_pump().unwrap();

    let window: sdl2::video::Window = video_subsystem
        .window("Snake Game", 320, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> = window
        .into_canvas()
        .software() /* Wow that took a while to debug */
        .present_vsync()
        .build()
        .unwrap();

    let texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext> =
        canvas.texture_creator();

    let mut texture: sdl2::render::Texture<'_> = texture_creator
        .create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, 32, 32)
        .unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <path to rom>");
        std::process::exit(1);
    }
    let file_path: &str = &args[1];
    let bytes: Vec<u8> = std::fs::read(file_path).unwrap();
    let rom: Rom = Rom::new(&bytes).unwrap();
    let bus: Bus = Bus::new(rom);
    let mut emu: NNES = NNES::new(bus);
    emu.reset_state_snake();

    let mut screen_state: [u8; 3072] = [0 as u8; 32 * 3 * 32];
    let mut rng: rand::prelude::ThreadRng = rand::rng();
    let mut last_frame: Instant = Instant::now();

    emu.run_callback(move |emu: &mut NNES| {
        handle_user_input(emu, &mut event_pump);
        emu.memory_write_u8(0xfe, rng.random_range(1..16));
        if read_screen_state(&emu, &mut screen_state) {
            let pitch: usize = 32 * 3;
            texture.update(None, &screen_state, pitch).unwrap();
            canvas.clear();
            canvas
                .copy(&texture, None, Some(Rect::new(0, 0, 32 * 10, 32 * 10)))
                .unwrap();
            canvas.present();
        }
        let frame_time = Duration::from_micros(100);
        while last_frame.elapsed() < frame_time {
            std::thread::yield_now();
        }
        last_frame = Instant::now();
    });
}

#[cfg(test)]
mod test {
    use std::env;
    use super::*;

    use trace::trace;

    #[test]
    fn nestest() {
        let args: Vec<String> = env::args().collect();
        if args.len() != 2 {
            eprintln!("Usage: cargo run -- <path to rom>");
            std::process::exit(1);
        }
        let file_path: &str = &args[1];
        let bytes: Vec<u8> = std::fs::read(file_path).unwrap();
        let rom: Rom = Rom::new(&bytes).unwrap();
        let bus: Bus = Bus::new(rom);
        let mut emu: NNES = NNES::new(bus);
        emu.set_program_counter(0xC000);
        emu.run_callback(move |emu: &mut NNES| {
            trace(emu);
        });
    }
}