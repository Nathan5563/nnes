#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

mod loader;
mod nnes;

use std::{env, process};
use loader::Cartridge;
use nnes::NNES;

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
        Err(msg) => { die!(msg.as_str()); },
    };
    
    let cartridge = Cartridge::new(&rom);
    let nnes = NNES::new(cartridge);

    println!("Hello, world!");
}
