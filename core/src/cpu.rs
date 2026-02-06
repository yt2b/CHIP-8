use crate::{
    display::Display,
    memory::{Memory, PROGRAM_START},
};
use rand::Rng;

macro_rules! combine_nibbles {
    ($($nibble:expr),+) => {
        {
            [$($nibble),+]
                .iter()
                .rev()
                .enumerate()
                .fold(0u32, |acc, (i, &v)| acc | ((v as u32) << (i * 4)))
        }
    };
}

pub struct Cpu {
    v: [u8; 16],
    pc: usize,
    stack: [usize; 16],
    sp: u8,
    i: usize,
    dt: u8,
    st: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            pc: PROGRAM_START,
            stack: [0; 16],
            sp: 0,
            i: 0,
            dt: 0,
            st: 0,
        }
    }

    fn split_byte(&self, byte: u8) -> (u8, u8) {
        (byte >> 4, byte & 0x0F)
    }

    fn skip_if(&mut self, condition: bool) {
        if condition {
            self.pc += 2;
        }
    }

    fn execute_0x_8(&mut self, b: u8, c: u8, d: u8) {
        match d {
            0x0 => self.v[b as usize] = self.v[c as usize],
            0x1 => self.v[b as usize] |= self.v[c as usize],
            0x2 => self.v[b as usize] &= self.v[c as usize],
            0x3 => self.v[b as usize] ^= self.v[c as usize],
            0x4 => {
                let (result, carry) = self.v[b as usize].overflowing_add(self.v[c as usize]);
                self.v[b as usize] = result;
                self.v[0xF] = if carry { 1 } else { 0 };
            }
            0x5 => {
                let (result, carry) = self.v[b as usize].overflowing_sub(self.v[c as usize]);
                self.v[b as usize] = result;
                self.v[0xF] = if carry { 0 } else { 1 };
            }
            0x6 => {
                self.v[0xF] = self.v[b as usize] & 0x1;
                self.v[b as usize] >>= 1;
            }
            0x7 => {
                let (result, borrow) = self.v[c as usize].overflowing_sub(self.v[b as usize]);
                self.v[b as usize] = result;
                self.v[0xF] = if borrow { 0 } else { 1 };
            }
            0xE => {
                self.v[0xF] = (self.v[b as usize] & 0x80) >> 7;
                self.v[b as usize] <<= 1;
            }
            _ => {
                eprintln!("Unknown opcode: 8{:X}{:X}{:X} at 0x{:X}", b, c, d, self.pc);
            }
        }
    }

    fn execute_0x_f(&mut self, b: usize, c: u8, d: u8, memory: &mut Memory) {
        match (c, d) {
            (0x0, 0x7) => self.v[b] = self.dt,
            (0x1, 0x5) => self.dt = self.v[b],
            (0x1, 0x8) => self.st = self.v[b],
            (0x1, 0xE) => self.i += self.v[b] as usize,
            (0x2, 0x9) => self.i = (self.v[b] as usize) * 5,
            (0x3, 0x3) => {
                let value = self.v[b];
                memory.data[self.i] = value / 100;
                memory.data[self.i + 1] = (value % 100) / 10;
                memory.data[self.i + 2] = value % 10;
            }
            (0x5, 0x5) => {
                for idx in 0..=b {
                    memory.data[self.i + idx] = self.v[idx];
                }
            }
            (0x6, 0x5) => {
                for idx in 0..=b {
                    self.v[idx] = memory.data[self.i + idx];
                }
            }
            _ => {
                eprintln!("Unknown opcode: F{:X}{:X}{:X} at 0x{:X}", b, c, d, self.pc);
            }
        }
    }

    pub fn step(&mut self, memory: &mut Memory, display: &mut Display, key: u16) {
        let (a, b) = self.split_byte(memory.data[self.pc]);
        let (c, d) = self.split_byte(memory.data[self.pc + 1]);
        match (a, b, c, d) {
            (0x0, 0x0, 0xE, 0x0) => display.clear(),
            (_, _, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
                return;
            }
            (0x1, _, _, _) => {
                self.pc = combine_nibbles!(b, c, d) as usize;
                return;
            }
            (0x2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc + 2;
                self.sp += 1;
                self.pc = combine_nibbles!(b, c, d) as usize;
                return;
            }
            (0x3, _, _, _) => {
                let vx = self.v[b as usize];
                let nn = combine_nibbles!(c, d) as u8;
                self.skip_if(vx == nn);
            }
            (0x4, _, _, _) => {
                let vx = self.v[b as usize];
                let nn = combine_nibbles!(c, d) as u8;
                self.skip_if(vx != nn);
            }
            (0x5, _, _, 0x0) => {
                let vx = self.v[b as usize];
                let vy = self.v[c as usize];
                self.skip_if(vx == vy);
            }
            (0x6, _, _, _) => {
                let value = combine_nibbles!(c, d) as u8;
                self.v[b as usize] = value;
            }
            (0x7, _, _, _) => {
                let value = combine_nibbles!(c, d) as u8;
                self.v[b as usize] = self.v[b as usize].wrapping_add(value);
            }
            (0x8, _, _, _) => self.execute_0x_8(b, c, d),
            (0x9, _, _, 0x0) => {
                let vx = self.v[b as usize];
                let vy = self.v[c as usize];
                self.skip_if(vx != vy);
            }
            (0xA, _, _, _) => {
                self.i = combine_nibbles!(b, c, d) as usize;
            }
            (0xB, _, _, _) => {
                self.pc = combine_nibbles!(b, c, d) as usize + self.v[0] as usize;
                return;
            }
            (0xC, _, _, _) => {
                let rand_byte: u8 = rand::rng().random();
                let nn = combine_nibbles!(c, d) as u8;
                self.v[b as usize] = rand_byte & nn;
            }
            (0xD, _, _, _) => {
                let x = self.v[b as usize] as usize;
                let y = self.v[c as usize] as usize;
                let collision = display.draw(x, y, &memory.data[self.i..self.i + (d as usize)]);
                self.v[0xF] = if collision { 1 } else { 0 };
            }
            (0xE, _, 0x9, 0xE) => {
                let idx = self.v[b as usize];
                self.skip_if(key & (1 << idx) != 0);
            }
            (0xE, _, 0xA, 0x1) => {
                let idx = self.v[b as usize];
                self.skip_if(key & (1 << idx) == 0);
            }
            (0xF, _, 0x0, 0xA) => {
                if key == 0 {
                    return;
                }
                for i in 0..=0xF {
                    if key & (1 << i) != 0 {
                        self.v[b as usize] = i as u8;
                        break;
                    }
                }
            }
            (0xF, _, _, _) => self.execute_0x_f(b as usize, c, d, memory),
            _ => {
                eprintln!(
                    "Unknown opcode: {:X}{:X}{:X}{:X} at 0x{:X}",
                    a, b, c, d, self.pc
                );
            }
        }
        self.pc += 2;
    }

    pub fn dec_delay_timer(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
    }

    pub fn dec_sound_timer(&mut self) {
        if self.st > 0 {
            self.st -= 1;
        }
    }

    pub fn get_sound_timer(&self) -> u8 {
        self.st
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{display::Display, memory::Memory};
    use super::Cpu;

    fn initialize(program: &[u8]) -> (Cpu, Memory, Display) {
        let cpu = Cpu::new();
        let memory = Memory::new(program);
        let display = Display::new();
        (cpu, memory, display)
    }

    #[test]
    fn test_combine_nibbles() {
        assert_eq!(combine_nibbles!(0xA, 0xB, 0xC), 0xABC);
    }

    #[test]
    fn test_split_byte() {
        let cpu = Cpu::new();
        assert_eq!(cpu.split_byte(0xAB), (0xA, 0xB));
    }

    #[test]
    fn test_return() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x00, 0xEE]);
        cpu.stack[0] = 0x100;
        cpu.sp = 1;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.pc, 0x100);
    }

    #[test]
    fn test_jump() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x12, 0x00]);

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.pc, 0x200);
    }

    #[test]
    fn test_call() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x23, 0x00]);

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.stack[0], 0x202);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.pc, 0x300);
    }

    #[test]
    fn test_skip_if_equal() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x30, 0x42, 0x00, 0x00, 0x30, 0x41]);
        cpu.v[0] = 0x42;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.pc, 0x204);
        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn test_skip_if_not_equal() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x40, 0x41, 0x00, 0x00, 0x40, 0x42]);
        cpu.v[0] = 0x42;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.pc, 0x204);
        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn test_load_value() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x62, 0xFF]);

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[2], 0xFF);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_add_value() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x73, 0x0F]);
        cpu.v[3] = 0xF0;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[3], 0xFF);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_load_register() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x85, 0x30]);
        cpu.v[3] = 0xAB;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[5], 0xAB);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_or_register() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x82, 0x31]);
        cpu.v[2] = 0b10101010;
        cpu.v[3] = 0b11001100;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[2], 0b11101110);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_and_register() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x82, 0x32]);
        cpu.v[2] = 0b10101010;
        cpu.v[3] = 0b11001100;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[2], 0b10001000);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_xor_register() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x82, 0x33]);
        cpu.v[2] = 0b10101010;
        cpu.v[3] = 0b11001100;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[2], 0b01100110);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_add_register_with_carry() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x84, 0x54, 0x86, 0x74]);
        cpu.v[4] = 200;
        cpu.v[5] = 100;
        cpu.v[6] = 1;
        cpu.v[7] = 2;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[4], 44);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x202);
        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[6], 3);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_sub_register_with_borrow() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x84, 0x55, 0x86, 0x75]);
        cpu.v[4] = 50;
        cpu.v[5] = 100;
        cpu.v[6] = 200;
        cpu.v[7] = 150;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[4], 206);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x202);
        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[6], 50);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_right_shift() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x82, 0x06, 0x82, 0x06]);
        cpu.v[2] = 0b00000101;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[2], 0b00000010);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x202);
        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[2], 0b00000001);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_subn_register_with_borrow() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x84, 0x57, 0x86, 0x77]);
        cpu.v[4] = 100;
        cpu.v[5] = 50;
        cpu.v[6] = 150;
        cpu.v[7] = 200;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[4], 206); // 256 - 50 = 206
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x202);
        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[6], 50);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_left_shift() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x82, 0x0E, 0x82, 0x0E]);
        cpu.v[2] = 0b10000001;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[2], 0b00000010);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x202);
        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[2], 0b00000100);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_skip_if_registers_not_equal() {
        let (mut cpu, mut memory, mut display) = initialize(&[0x90, 0x10, 0x00, 0x00, 0x91, 0x20]);
        cpu.v[0] = 0x42;
        cpu.v[1] = 0x43;
        cpu.v[2] = 0x43;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.pc, 0x204);
        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn test_set_index() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xA2, 0xF0]);

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.i, 0x2F0);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_jump_with_offset() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xB2, 0x00]);
        cpu.v[0] = 0x10;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.pc, 0x210);
    }

    #[test]
    fn test_random_and() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xC3, 0x0F]);

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[3] & 0x0F, cpu.v[3]);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_draw_sprite() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xD0, 0x15, 0xD0, 0x15]);
        cpu.v[0] = 1;
        cpu.i = 0x05;

        cpu.step(&mut memory, &mut display, 0);
        for (i, data) in [
            [false, false, true, false],
            [false, true, true, false],
            [false, false, true, false],
            [false, false, true, false],
            [false, true, true, true],
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(display.data[i][1..5], *data);
        }
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x202);
        cpu.i = 0x0A;
        cpu.step(&mut memory, &mut display, 0);
        for (i, data) in [
            [true, true, false, true],
            [false, true, true, true],
            [true, true, false, true],
            [true, false, true, false],
            [true, false, false, false],
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(display.data[i][1..5], *data);
        }
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_skip_if_key_is_pressed() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xE1, 0x9E, 0x00, 0x00, 0xE2, 0x9E]);
        cpu.v[1] = 0x2;
        cpu.v[2] = 0xF;

        cpu.step(&mut memory, &mut display, 0b0100); // Key 2 pressed
        assert_eq!(cpu.pc, 0x204);
        cpu.step(&mut memory, &mut display, 0b0000); // Key F not pressed
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn test_skip_if_key_is_not_pressed() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xE1, 0xA1, 0x00, 0x00, 0xE2, 0xA1]);
        cpu.v[1] = 0x3;
        cpu.v[2] = 0xE;

        cpu.step(&mut memory, &mut display, 0b0000);
        assert_eq!(cpu.pc, 0x204);
        cpu.step(&mut memory, &mut display, 0b0100000000000000);
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn test_load_delay_timer() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xF2, 0x07]);
        cpu.dt = 0x55;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[2], 0x55);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_wait_for_key_press() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xF6, 0x0A]);

        cpu.step(&mut memory, &mut display, 0b0000);
        assert_eq!(cpu.pc, 0x200);
        cpu.step(&mut memory, &mut display, 0b0100);
        assert_eq!(cpu.v[6], 0x02);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_set_delay_timer() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xF6, 0x15]);
        cpu.v[6] = 0xAA;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.dt, 0xAA);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_set_sound_timer() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xFA, 0x18]);
        cpu.v[0xA] = 0xBB;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.st, 0xBB);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_add_to_index() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xF5, 0x1E]);
        cpu.i = 0x300;
        cpu.v[5] = 0x20;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.i, 0x320);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_load_sprite_address() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xF4, 0x29]);
        cpu.v[4] = 0x5;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.i, 0x19);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_bcd_conversion() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xF3, 0x33]);
        cpu.v[3] = 254;
        cpu.i = 0x300;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(memory.data[0x300], 2);
        assert_eq!(memory.data[0x301], 5);
        assert_eq!(memory.data[0x302], 4);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_registers_to_memory() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xF2, 0x55]);
        cpu.v[0] = 0x10;
        cpu.v[1] = 0x20;
        cpu.v[2] = 0x30;
        cpu.i = 0x400;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(memory.data[0x400], 0x10);
        assert_eq!(memory.data[0x401], 0x20);
        assert_eq!(memory.data[0x402], 0x30);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_memory_to_registers() {
        let (mut cpu, mut memory, mut display) = initialize(&[0xF2, 0x65]);
        memory.data[0x500] = 0x11;
        memory.data[0x501] = 0x22;
        memory.data[0x502] = 0x33;
        cpu.i = 0x500;

        cpu.step(&mut memory, &mut display, 0);
        assert_eq!(cpu.v[0], 0x11);
        assert_eq!(cpu.v[1], 0x22);
        assert_eq!(cpu.v[2], 0x33);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_decrement_delay_timer() {
        let mut cpu = Cpu::new();
        cpu.dt = 1;

        cpu.dec_delay_timer();
        assert_eq!(cpu.dt, 0);
        cpu.dec_delay_timer();
        assert_eq!(cpu.dt, 0);
    }

    #[test]
    fn test_decrement_sound_timer() {
        let mut cpu = Cpu::new();
        cpu.st = 1;

        cpu.dec_sound_timer();
        assert_eq!(cpu.st, 0);
        cpu.dec_sound_timer();
        assert_eq!(cpu.st, 0);
    }

    #[test]
    fn test_get_sound_timer() {
        let mut cpu = Cpu::new();
        cpu.st = 42;

        assert_eq!(cpu.get_sound_timer(), 42);
    }
}
