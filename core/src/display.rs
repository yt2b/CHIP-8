pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub struct Display {
    pub data: [[bool; WIDTH]; HEIGHT],
}

impl Display {
    pub fn new() -> Self {
        Self {
            data: [[false; WIDTH]; HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        self.data = [[false; WIDTH]; HEIGHT];
    }

    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        for (row, byte) in sprite.iter().enumerate() {
            for bit in 0..8 {
                if (byte & (0x80 >> bit)) != 0 {
                    let px = (x + bit) % WIDTH;
                    let py = (y + row) % HEIGHT;
                    if self.data[py][px] {
                        collision = true;
                    }
                    self.data[py][px] ^= true;
                }
            }
        }
        collision
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::display::Display;
    use crate::display::{HEIGHT, WIDTH};

    #[test]
    fn test_clear() {
        let mut display = Display::new();
        display.clear();
        assert_eq!(display.data, [[false; WIDTH]; HEIGHT]);
    }

    #[test]
    fn test_draw_no_collision() {
        let mut display = Display::new();
        let sprite = [0xF0, 0x10, 0xF0, 0x80, 0xF0]; // '2' from font set
        let collision = display.draw(0, 0, &sprite);
        assert!(!collision);
        assert_eq!(display.data[0][0..4], [true, true, true, true]);
        assert_eq!(display.data[1][0..4], [false, false, false, true]);
        assert_eq!(display.data[2][0..4], [true, true, true, true]);
        assert_eq!(display.data[3][0..4], [true, false, false, false]);
        assert_eq!(display.data[4][0..4], [true, true, true, true]);
    }

    #[test]
    fn test_draw_with_collision() {
        let mut display = Display::new();
        display.draw(0, 0, &[0xF0]);
        let collision = display.draw(0, 0, &[0x90]);
        assert!(collision);
        assert_eq!(display.data[0][0..4], [false, true, true, false]);
    }

    #[test]
    fn test_draw_wrapping() {
        let mut display = Display::new();
        let collision = display.draw(WIDTH - 2, 0, &[0xF0]);
        assert!(!collision);
        assert_eq!(display.data[0][WIDTH - 2..WIDTH], [true, true]);
        assert_eq!(display.data[0][0..2], [true, true]);
    }
}
