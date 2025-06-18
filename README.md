# NNES: A Cycle-Accurate NES Emulator
NNES is a work-in-progress, cycle-accurate NES emulator written in Rust. It currently implements the 2A03 CPU core, memory bus, cartridge handling, and 2C02 PPU scaffolding. The goal is to achieve perfect cycle accuracy for NROM (mapper 0) games, with plans to support additional mappers in the future.

## Project Status
- Implemented and tested cycle accuracy of all official 6502 opcodes
- Implemented iNES parser with simple validation
- Implemented robust interrupt handling system
- PPU rendering functionality is currently under development
- Designing a custom controller PCB with an 8 bit shift register

 ## Dependencies
- Rust Toolchain via rustup (tested with 1.87.0)
- Native SDL2 development libraries (e.g. libsdl2-dev 2.0.20 on Ubuntu)

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
├── .gitignore
├── .rustfmt.toml
├── .vscode
│   └── tasks.json
├── Cargo.lock
├── Cargo.toml
├── README.md
├── build.rs
├── palettes
│   └── base.pal
├── scripts
│   └── load-palette.py
├── src
│   ├── cartridge.rs
│   ├── main.rs
│   ├── nnes
│   │   ├── apu
│   │   ├── apu.rs
│   │   ├── cpu
│   │   │   ├── bus
│   │   │   │   └── devices.rs
│   │   │   ├── bus.rs
│   │   │   └── opcodes.rs
│   │   ├── cpu.rs
│   │   ├── ppu
│   │   │   ├── core.rs
│   │   │   └── io.rs
│   │   └── ppu.rs
│   ├── nnes.rs
│   ├── palette.rs
│   └── utils.rs
└── todo.txt
```

## Roadmap
- Complete PPU timing & rendering pipeline
- Add keyboard and NES controller input support
- Implement APU and audio output
- Play Donkey Kong and SMB!
- Implement unofficial CPU opcodes
- Extend mapper support (beyond mapper 0)
