use crate::state::State;
use anyhow::Result;
use ggez::*;

mod state;

fn main() -> Result<()> {
    let state = State::new();
    let conf = ggez::conf::Conf::new();
    let (ctx, event_loop) = ggez::ContextBuilder::new("chip8", "")
        .default_conf(conf)
        .build()?;
    event::run(ctx, event_loop, state);
}
