use crate::cpu::Cpu;
use crate::display::{Display, HEIGHT, WIDTH};
use crate::memory::{MEMORY_LEN, Memory};
mod cpu;
mod display;
mod memory;

pub const DISPLAY_WIDTH: usize = WIDTH;
pub const DISPLAY_HEIGHT: usize = HEIGHT;

pub struct Chip8 {
    pub cpu: Cpu,
    pub memory: Memory,
    pub display: Display,
}

impl Chip8 {
    pub fn new(program: &[u8]) -> Self {
        Self {
            cpu: Cpu::new(),
            memory: Memory::new(program),
            display: Display::new(),
        }
    }

    pub fn step(&mut self, key: u16) {
        self.cpu.step(&mut self.memory, &mut self.display, key);
    }

    pub fn get_memory(&self) -> &[u8; MEMORY_LEN] {
        &self.memory.data
    }

    pub fn get_display(&self) -> &[[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT] {
        &self.display.data
    }

    pub fn dec_delay_timer(&mut self) {
        self.cpu.dec_delay_timer();
    }

    pub fn dec_sound_timer(&mut self) {
        self.cpu.dec_sound_timer();
    }

    pub fn get_sound_timer(&self) -> u8 {
        self.cpu.get_sound_timer()
    }
}
