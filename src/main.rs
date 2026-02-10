use crate::state::State;
use anyhow::Result;
use clap::Parser;
use ggez::{conf::WindowSetup, *};
use std::fs::File;
use std::io::Read;
mod state;

#[derive(Parser, Debug)]
struct Args {
    rom_path: String,
}

fn read_rom(path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    if buffer.len() > 0xE00 {
        return Err(anyhow::anyhow!("ROM size exceeds memory limit"));
    }
    Ok(buffer)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let rom = read_rom(&args.rom_path)?;
    let state = State::new(&rom);
    let conf = ggez::conf::Conf::new();
    let (ctx, event_loop) = ggez::ContextBuilder::new("chip8", "")
        .default_conf(conf)
        .window_setup(WindowSetup::default().title("CHIP-8 Emulator"))
        .build()?;
    event::run(ctx, event_loop, state);
}
