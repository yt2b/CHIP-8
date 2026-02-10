use ggez::{event::EventHandler, *};

pub struct State {
    duration: std::time::Duration,
    chip8: core::Chip8,
}

impl State {
    pub fn new(rom: &[u8]) -> Self {
        Self {
            duration: std::time::Duration::new(0, 0),
            chip8: core::Chip8::new(rom),
        }
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.duration = ctx.time.delta();
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
}
