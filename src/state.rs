use core::Chip8;
use ggez::{
    event::EventHandler,
    graphics::{Color, DrawMode, Mesh},
    input::keyboard::KeyCode,
    *,
};

pub const SIZE: usize = 14;
pub const SPACE: usize = 2;
const FRAME_TIME: f32 = 1000.0 / 60.0;
// Key mapping
// 1 2 3 C -> 1 2 3 4
// 4 5 6 D -> Q W E R
// 7 8 9 E -> A S D F
// A 0 B F -> Z X C V
const KEYCODES: [KeyCode; 16] = [
    KeyCode::X,
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Q,
    KeyCode::W,
    KeyCode::E,
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::Z,
    KeyCode::C,
    KeyCode::Key4,
    KeyCode::R,
    KeyCode::F,
    KeyCode::V,
];

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
        let key = KEYCODES
            .iter()
            .enumerate()
            .map(|(i, &kc)| {
                if ctx.keyboard.is_key_pressed(kc) {
                    1 << i
                } else {
                    0
                }
            })
            .sum();
        self.chip8.step(key);
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
