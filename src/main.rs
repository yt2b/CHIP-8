use crate::state::State;
use anyhow::Result;
use ggez::{conf::WindowSetup, *};

mod state;

fn main() -> Result<()> {
    let state = State::new();
    let conf = ggez::conf::Conf::new();
    let (ctx, event_loop) = ggez::ContextBuilder::new("chip8", "")
        .default_conf(conf)
        .window_setup(WindowSetup::default().title("CHIP-8 Emulator"))
        .build()?;
    event::run(ctx, event_loop, state);
}
