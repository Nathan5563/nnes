#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

mod loader;
mod nnes;
mod utils;

use loader::Cartridge;
use nnes::NNES;
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use std::{env, process, thread, time::Duration};

macro_rules! die {
    ($msg:expr) => {
        eprintln!("{}", $msg);
        process::exit(1)
    };
}

// The original NTSC NES palette (64 colours).
pub const NES_PALETTE: [(u8, u8, u8); 64] = [
    (84, 84, 84),
    (0, 30, 116),
    (8, 16, 144),
    (48, 0, 136),
    (68, 0, 100),
    (92, 0, 48),
    (84, 4, 0),
    (60, 24, 0),
    (32, 42, 0),
    (8, 58, 0),
    (0, 64, 0),
    (0, 60, 0),
    (0, 50, 60),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (152, 150, 152),
    (8, 76, 196),
    (48, 50, 236),
    (92, 30, 228),
    (136, 20, 176),
    (160, 20, 100),
    (152, 34, 32),
    (120, 60, 0),
    (84, 90, 0),
    (40, 114, 0),
    (8, 124, 0),
    (0, 118, 40),
    (0, 102, 120),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (236, 238, 236),
    (76, 154, 236),
    (120, 124, 236),
    (176, 98, 236),
    (228, 84, 236),
    (236, 88, 180),
    (236, 106, 100),
    (212, 136, 32),
    (160, 170, 0),
    (116, 196, 0),
    (76, 208, 32),
    (56, 204, 108),
    (56, 180, 204),
    (60, 60, 60),
    (0, 0, 0),
    (0, 0, 0),
    (236, 238, 236),
    (168, 204, 236),
    (188, 188, 236),
    (212, 178, 236),
    (236, 174, 236),
    (236, 174, 212),
    (236, 180, 176),
    (228, 196, 144),
    (204, 210, 120),
    (180, 222, 120),
    (168, 226, 144),
    (152, 226, 180),
    (160, 214, 228),
    (160, 162, 160),
    (0, 0, 0),
    (0, 0, 0),
];

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("nnes", 256 * 2, 240 * 2)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .software()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
        .map_err(|e| e.to_string())?;

    // Create your emulator
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        die!("usage: cargo run -- <path to rom>");
    }
    let rom = match loader::validate(&args[1]) {
        Ok(rom) => rom,
        Err(msg) => {
            die!(msg.as_str());
        }
    };
    let cartridge = Cartridge::new(&rom);
    let nnes = &mut NNES::new(cartridge);
    nnes.reset();

    loop {
        nnes.cpu.step(true);
    }

    // // Master-cycles to tick per frame:
    // // NES CPU runs ~1.7898 MHz, frame rate ~60.1 Hz ⇒ ~29780 CPU ticks/frame.
    // // You tick CPU once per 12 master cycles ⇒ master_cycles_per_frame ≈ 29780 * 12.
    // let master_cycles_per_frame = 29780 * 12;

    // let mut event_pump = sdl.event_pump()?;
    // 'running: loop {
    //     // 1) Advance the emu
    //     for _ in 0..master_cycles_per_frame {
    //         nnes.tick();
    //     }

    //     // 2) Map ppu.output_buffer (u8 indices) → raw RGB bytes
    //     texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
    //         for (i, &palette_idx) in nnes.ppu.output_buffer.iter().enumerate() {
    //             let (r, g, b) = NES_PALETTE[palette_idx as usize];
    //             let base = i * 3;
    //             buffer[base + 0] = r;
    //             buffer[base + 1] = g;
    //             buffer[base + 2] = b;
    //         }
    //     })?;

    //     // 3) Blit and present
    //     canvas.clear();
    //     // scale 2× or 3× if you like
    //     canvas.copy(&texture, None, None)?;
    //     canvas.present();

    //     // 4) Handle input & delay to ~60 Hz
    //     for event in event_pump.poll_iter() {
    //         if let Event::KeyDown { keycode: Some(Keycode::Escape), .. } = event {
    //             break 'running;
    //         }
    //     }
    //     // Roughly 16 ms/frame; vsync=true will also block
    //     thread::sleep(Duration::from_millis(16));
    // }

    Ok(())
}
