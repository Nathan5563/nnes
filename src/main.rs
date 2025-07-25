#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

mod cartridge;
mod controller;
mod nnes;
mod palette;
mod utils;

use cartridge::{validate_rom, Cartridge};
use nnes::NNES;
pub use palette::NES_PALETTE;
use sdl2::{
    keyboard::Keycode, pixels::PixelFormatEnum, render::Canvas, video::Window,
    Sdl,
};
use std::{
    env,
    fs::read,
    process,
    thread::sleep,
    time::{Duration, Instant},
};

macro_rules! die {
    ($msg:expr) => {
        eprintln!("{}", $msg);
        process::exit(1)
    };
}

fn init_sdl() -> Result<(Sdl, Canvas<Window>), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("nnes", 256 * 2, 240 * 2)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let canvas = window
        .into_canvas()
        .software()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    Ok((sdl, canvas))
}

fn init_emu() -> NNES {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        die!("usage: cargo run -- <path to rom>");
    }
    let rom = match read(args[1].clone()) {
        Ok(rom) => rom,
        Err(_) => {
            die!("error: invalid path to rom");
        }
    };
    match validate_rom(&rom) {
        Ok(_) => {}
        Err(msg) => {
            die!(msg.as_str());
        }
    };
    let cartridge = Cartridge::new(&rom);
    let mut nnes = NNES::new(cartridge);
    nnes.reset();
    nnes
}

fn main() -> Result<(), String> {
    let (sdl, mut canvas) = init_sdl()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
        .map_err(|e| e.to_string())?;

    let mut nnes = init_emu();

    // NES CPU runs ~1.7898 MHz, frame rate ~60.1 Hz: ~29780 CPU ticks/frame.
    // Tick CPU once per 12 master cycles: 29780 * 12 = ~357360 master cycles per frame.
    let master_cycles_per_frame = 357360;
    let target_frame_duration = Duration::from_millis(1000 / 60);

    let mut event_pump = sdl.event_pump()?;
    'running: loop {
        let frame_start = Instant::now();

        // 0) Update controller state from user input
        nnes.cpu
            .borrow_mut()
            .bus
            .get_joypad_ref()
            .unwrap()
            .update_state(&event_pump);

        // 1) Tick the emulator
        for _ in 0..master_cycles_per_frame {
            nnes.tick();
        }

        // 2) Map ppu.front (u8 indices) -> raw RGB bytes
        texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
            for (i, &palette_idx) in
                nnes.ppu.borrow_mut().front.iter().enumerate()
            {
                let (r, g, b) = NES_PALETTE[palette_idx as usize];
                let base = i * 3;
                buffer[base + 0] = r;
                buffer[base + 1] = g;
                buffer[base + 2] = b;
            }
        })?;

        // 3) Blit and present
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        // 4) Handle input
        for event in event_pump.poll_iter() {
            if let sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } = event
            {
                break 'running;
            }
        }

        // 5) Clamp to 60fps
        let frame_time = frame_start.elapsed();
        if frame_time < target_frame_duration {
            sleep(target_frame_duration - frame_time);
        }
    }

    Ok(())
}
