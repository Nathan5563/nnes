# NNES: A Cycle-Accurate NES Emulator
NNES is a work-in-progress, cycle-accurate NES emulator written in Rust. It currently implements the 2A03 CPU core, memory bus, cartridge handling, and 2C02 PPU scaffolding. The goal is perfect cycle-accuracy for NROM (mapper 0) games, with future plans to support additional mappers.

## Project Status
- Only tested on *nix environments
- All official CPU opcodes are implemented and passing nestest.nes, including cycle counts
- Basic iNES parser with validation is implemented
- Interrupt handling system is implemented and currently being tested
- PPU struct and associated functionality are currently under development
- Rendering with SDL2 is in the works

 ## Dependencies
- Rust Toolchain via rustup (tested with 1.87.0)
- Native SDL2 development libraries (e.g. libsdl2-dev on Ubuntu)

## Build & Run
### 0. Install dependencies
Rust: https://www.rust-lang.org/tools/install

SDL2 (Ubuntu):
```
sudo apt install libsdl2-dev
```

### 1. Clone the repo
```
git clone git@github.com:Nathan5563/nnes.git
cd nnes
```

### 2. Build in release mode
```
cargo build --release
```

### 3. Run with a ROM file
```
cargo run --release -- path/to/your.nes
```

## Repository Layout
```
nnes
├── Cargo.lock
├── Cargo.toml
├── build.rs
├── palettes/
│   └── smooth.pal
├── scripts/
│   └── load_palette.py
└── src/
    ├── loader.rs
    ├── main.rs
    ├── nnes/
    │   ├── apu/
    │   ├── apu.rs
    │   ├── cpu/
    │   │   ├── bus/
    │   │   │   └── devices.rs
    │   │   ├── bus.rs
    │   │   └── opcodes.rs
    │   ├── cpu.rs
    │   ├── ppu/
    │   │   ├── registers.rs
    │   │   └── render.rs
    │   └── ppu.rs
    ├── nnes.rs
    ├── palette.rs
    └── utils.rs
```

## Roadmap
- Complete PPU timing & rendering pipeline
- Add keyboard and NES controller input support
- Implement APU and audio output
- Play Donkey Kong and SMB!
- Implement unofficial CPU opcodes
- Extend mapper support (beyond mapper 0)
