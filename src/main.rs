#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

mod loader;
mod nnes;
mod utils;

use loader::Cartridge;
use nnes::NNES;
use std::{env, process};

macro_rules! die {
    ($msg:expr) => {
        eprintln!("{}", $msg);
        process::exit(1)
    };
}

fn main() {
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
    nnes.cpu.reset();
    nnes.cpu.pc = 0xc000;
    loop {
        nnes.cpu.step(true);
    }
}
