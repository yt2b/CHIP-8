use ggez::{event::EventHandler, *};

pub struct State {
    duration: std::time::Duration,
}

impl State {
    pub fn new() -> Self {
        Self {
            duration: std::time::Duration::new(0, 0),
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
