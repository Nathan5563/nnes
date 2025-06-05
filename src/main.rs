#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

mod cartridge;
mod nnes;
mod utils;

use cartridge::{validate_rom, Cartridge};
use nnes::NNES;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    render::{Canvas, Texture},
    video::Window,
    Sdl,
};
use std::{env, process, thread, time::Duration};

macro_rules! die {
    ($msg:expr) => {
        eprintln!("{}", $msg);
        process::exit(1)
    };
}

mod palette;
pub use palette::NES_PALETTE;

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
    let rom = match validate_rom(&args[1]) {
        Ok(rom) => rom,
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

    // // Trace CPU execution
    // loop {
    //     nnes.cpu.step(true);
    // }

    // // Draw NES_PALETTE to screen
    // let mut event_pump = sdl.event_pump()?;
    // 'running: loop {
    //     texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //         for y in 0..240 {
    //             for x in 0..256 {
    //                 // Divide into an 8×8 grid of blocks, each 32×30px
    //                 let block_x = x / 32;
    //                 let block_y = y / 30;
    //                 let palette_idx = (block_y * 8 + block_x) as usize;
    //                 let (r, g, b) = NES_PALETTE[palette_idx];

    //                 let offset = y * pitch + x * 3;
    //                 buffer[offset] = r;
    //                 buffer[offset + 1] = g;
    //                 buffer[offset + 2] = b;
    //             }
    //         }
    //     })?;

    //     canvas.clear();
    //     canvas.copy(&texture, None, None)?;
    //     canvas.present();

    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit { .. } => {
    //                 break 'running;
    //             }
    //             _ => {}
    //         }
    //     }

    //     thread::sleep(Duration::from_millis(16));
    // }

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
