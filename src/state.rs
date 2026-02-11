use core::Chip8;
use ggez::{
    event::EventHandler,
    graphics::{Color, DrawMode, Mesh},
    *,
};

pub const SIZE: usize = 14;
pub const SPACE: usize = 2;
const FRAME_TIME: f32 = 1000.0 / 60.0;

pub struct State {
    chip8: Chip8,
    elapsed_ms: f32,
    is_first_frame: bool,
    gray: Color,
}

impl State {
    pub fn new(rom: &[u8]) -> Self {
        Self {
            chip8: Chip8::new(rom),
            elapsed_ms: 0.0,
            is_first_frame: true,
            gray: Color::from_rgb_u32(0x101010),
        }
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.chip8.step(0);
        if !self.is_first_frame {
            self.elapsed_ms += ctx.time.delta().as_secs_f32() * 1000.0;
            while self.elapsed_ms >= FRAME_TIME {
                self.chip8.dec_delay_timer();
                self.chip8.dec_sound_timer();
                self.elapsed_ms -= FRAME_TIME;
            }
        } else {
            self.is_first_frame = false;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        let mut mb = graphics::MeshBuilder::new();
        let display = self.chip8.get_display();
        for (y, row) in display.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                let rect = graphics::Rect::new(
                    (x * (SIZE + SPACE)) as f32,
                    (y * (SIZE + SPACE)) as f32,
                    SIZE as f32,
                    SIZE as f32,
                );
                let color = if pixel { Color::GREEN } else { self.gray };
                mb.rectangle(DrawMode::fill(), rect, color)?;
            }
        }
        let mesh = Mesh::from_data(ctx, mb.build());
        canvas.draw(&mesh, graphics::DrawParam::default());
        canvas.finish(ctx)
    }
}
