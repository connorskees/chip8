use std::{convert::TryFrom, fs, io, path::Path};

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rand::Rng;

const WIDTH: usize = 640;
const HEIGHT: usize = 320;

const TRUE_WIDTH: usize = 64;
const TRUE_HEIGHT: usize = 32;

struct Emulator {
    memory: [u8; 4096],
    registers: [u8; 16],
    index_register: u16,
    program_counter: u16,
    bitmap: Vec<u8>,
    big_bitmap: Vec<u32>,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: u16,
    keys: [bool; 16],
    should_draw: bool,
    window: Window,
}

impl Emulator {
    pub fn init(window: Window) -> Self {
        let mut memory = [0; 4096];

        let sprites: [[u8; 5]; 16] = [
            [0xF0, 0x90, 0x90, 0x90, 0xF0],
            [0x20, 0x60, 0x20, 0x20, 0x70],
            [0xF0, 0x10, 0xF0, 0x80, 0xF0],
            [0xF0, 0x10, 0xF0, 0x10, 0xF0],
            [0x90, 0x90, 0xF0, 0x10, 0x10],
            [0xF0, 0x80, 0xF0, 0x10, 0xF0],
            [0xF0, 0x80, 0xF0, 0x90, 0xF0],
            [0xF0, 0x10, 0x20, 0x40, 0x40],
            [0xF0, 0x90, 0xF0, 0x90, 0xF0],
            [0xF0, 0x90, 0xF0, 0x10, 0xF0],
            [0xF0, 0x90, 0xF0, 0x90, 0x90],
            [0xE0, 0x90, 0xE0, 0x90, 0xE0],
            [0xF0, 0x80, 0x80, 0x80, 0xF0],
            [0xE0, 0x90, 0x90, 0x90, 0xE0],
            [0xF0, 0x80, 0xF0, 0x80, 0xF0],
            [0xF0, 0x80, 0xF0, 0x80, 0x80],
        ];

        let mut i = 0;
        for sprite in &sprites {
            for ch in sprite {
                memory[i] = *ch;
                i += 1;
            }
        }

        Self {
            program_counter: 0x200,
            index_register: 0,
            stack_pointer: 0,
            memory,
            registers: [0; 16],
            bitmap: vec![0; TRUE_WIDTH * TRUE_HEIGHT],
            big_bitmap: vec![0; WIDTH * HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            keys: [false; 16],
            should_draw: false,
            window,
        }
    }

    pub fn load_file(&mut self, path: &Path) -> io::Result<()> {
        self.set_bytes(0x200, &fs::read(path)?);
        Ok(())
    }

    fn set_bytes(&mut self, offset: usize, bytes: &[u8]) {
        debug_assert!(bytes.len() + offset <= self.memory.len());
        self.memory
            .get_mut(offset..(offset + bytes.len()))
            .unwrap()
            .copy_from_slice(bytes)
    }

    pub fn do_game_loop(&mut self) {
        while self.window.is_open() {
            self.emulate_cycle();

            if self.should_draw {
                self.draw_graphics();
                self.should_draw = false;
            }

            if let Some(keys) = self.window.get_keys() {
                self.set_keys(&keys);
            }
        }
    }

    fn draw_graphics(&mut self) {
        for y in 0..HEIGHT {
            let y_coord = y / 10;
            let offset = y * WIDTH;
            for x in 0..WIDTH {
                let index = y_coord * TRUE_WIDTH + (x / 10);
                let pixel = self.bitmap[index];
                self.big_bitmap[offset + x] = match pixel {
                    0 => 0,
                    1 => 0xffffff,
                    _ => unreachable!(),
                };
            }
        }

        self.window
            .update_with_buffer(&self.big_bitmap, WIDTH, HEIGHT)
            .unwrap();
    }

    fn set_keys(&mut self, keys: &[Key]) {
        self.keys = [false; 16];
        for key in keys.iter().map(integer_from_key) {
            if let Some(k) = key {
                self.keys[usize::from(k)] = true;
            }
        }
    }

    fn get_key(&mut self) -> u8 {
        loop {
            self.window.update();
            if let Some(keys) = self.window.get_keys_pressed(KeyRepeat::Yes) {
                if keys.is_empty() {
                    continue;
                }
                if let Some(key) = integer_from_key(&keys[0]) {
                    return key;
                }
            }
        }
    }

    fn get_register_x_mut(&mut self, opcode: u16) -> &mut u8 {
        self.registers
            .get_mut(usize::from(opcode & 0x0F00) >> 8)
            .unwrap()
    }

    fn get_register_x(&mut self, opcode: u16) -> u8 {
        *self
            .registers
            .get(usize::from(opcode & 0x0F00) >> 8)
            .unwrap()
    }

    fn get_register_y(&mut self, opcode: u16) -> u8 {
        *self
            .registers
            .get(usize::from(opcode & 0x00F0) >> 4)
            .unwrap()
    }

    fn emulate_cycle(&mut self) {
        let opcode = u16::from(self.memory[self.program_counter as usize]) << 8
            | u16::from(self.memory[self.program_counter as usize + 1]);

        // println!("{:X}", opcode);

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00EE => todo!("{:#X}", opcode),
                0x00E0 => todo!("{:#X}", opcode),
                _ => todo!("{:#X}", opcode),
            },
            0x1000 => {
                self.program_counter = opcode & 0x0FFF;
                return;
            }
            0x2000 => todo!("{:#X}", opcode),
            0x3000 => {
                let register = self.get_register_x(opcode);

                let value = u8::try_from(opcode & 0x00FF).unwrap();

                if register == value {
                    self.program_counter += 2;
                }
            }
            0x4000 => {
                let register = self.get_register_x(opcode);

                let value = u8::try_from(opcode & 0x00FF).unwrap();

                if register != value {
                    self.program_counter += 2;
                }
            }
            0x5000 => todo!("{:#X}", opcode),
            0x6000 => {
                let register_value = u8::try_from(opcode & 0x00FF).unwrap();
                *self.get_register_x_mut(opcode) = register_value;
            }
            0x7000 => {
                let register = self.get_register_x_mut(opcode);
                let value = opcode & 0x00FF;

                *register = ((*register as u16 + value) & 0x00FF) as u8;
            }
            0x8000 => match opcode & 0x000F {
                0x0000 => todo!("{:#X}", opcode),
                0x0001 => todo!("{:#X}", opcode),
                0x0002 => {
                    let register_y = self.get_register_y(opcode);

                    let register_x = self.get_register_x_mut(opcode);

                    *register_x &= register_y;
                }
                0x0003 => todo!("{:#X}", opcode),
                0x0004 => {
                    let register_y = self.get_register_y(opcode);

                    let register_x = self.get_register_x_mut(opcode);

                    let wrapped = register_x.wrapping_add(register_y);

                    if register_x.checked_add(register_y).is_some() {
                        *register_x = wrapped;
                        self.registers[0xf] = 1;
                    } else {
                        *register_x = wrapped;
                        self.registers[0xf] = 0;
                    }
                }
                0x0005 => todo!("{:#X}", opcode),
                0x0006 => todo!("{:#X}", opcode),
                0x0007 => todo!("{:#X}", opcode),
                0x000E => todo!("{:#X}", opcode),
                _ => unreachable!("{:#X}", opcode),
            },
            0x9000 => todo!("{:#X}", opcode),
            0xA000 => self.index_register = opcode & 0x0FFF,
            0xB000 => todo!("{:#X}", opcode),
            0xC000 => {
                let mut rng = rand::thread_rng();

                let random_number: u8 = rng.gen();

                let value = opcode & 0x00FF;

                *self.get_register_x_mut(opcode) = (value & u16::from(random_number)) as u8;
            }
            0xD000 => {
                let x = self.get_register_x(opcode);
                let y = self.get_register_y(opcode);
                let height = usize::from(opcode & 0x000F);

                self.draw(x as usize, y as usize, height);
            }
            0xE000 => match opcode & 0x00FF {
                0x009E => {
                    let register = self.get_register_x(opcode);

                    let key_exists = self.keys[usize::from(register)];

                    if key_exists {
                        self.program_counter += 2;
                    }
                }
                0x00A1 => {
                    let register = self.get_register_x(opcode);

                    let key_exists = self.keys[usize::from(register)];

                    if !key_exists {
                        self.program_counter += 2;
                    }
                }
                _ => unreachable!("{:#X}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x000A => {
                    let key = self.get_key();

                    let register = self.get_register_x_mut(opcode);

                    *register = key;
                }
                0x0015 => todo!("{:#X}", opcode),
                0x0018 => {
                    let x = self.get_register_x(opcode);

                    self.sound_timer = x;
                }
                0x001E => todo!("{:#X}", opcode),
                0x0029 => self.index_register = self.get_register_x(opcode) as u16 * 5,
                0x0033 => {
                    let register = self.get_register_x(opcode);

                    *self.memory.get_mut(self.index_register as usize).unwrap() = register / 100;
                    *self
                        .memory
                        .get_mut(self.index_register as usize + 1)
                        .unwrap() = (register % 100) / 10;
                    *self
                        .memory
                        .get_mut(self.index_register as usize + 2)
                        .unwrap() = register % 10;
                }
                0x0055 => todo!("{:#X}", opcode),
                0x0065 => {
                    let register_idx = (opcode as usize & 0x0F00) >> 8;
                    self.registers
                        .get_mut(0..=register_idx)
                        .unwrap()
                        .copy_from_slice(
                            &self.memory[self.index_register as usize
                                ..=(self.index_register as usize + register_idx)],
                        )
                }
                _ => unreachable!("{:#X}", opcode),
            },
            _ => unreachable!("{:#X}", opcode),
        }

        self.program_counter += 2;

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // println!("Beep!");
            }
            self.sound_timer -= 1;
        }
    }

    /// Draws a sprite at coordinate (x, y) that has a width of 8 pixels
    /// and a height of N pixels. Each row of 8 pixels is read as bit-coded
    /// starting from memory location I; I value doesn't change after the
    /// execution of this instruction. As described above, VF is set to 1 if
    /// any screen pixels are flipped from set to unset when the sprite is
    /// drawn, and to 0 if that doesn't happen
    fn draw(&mut self, x: usize, y: usize, height: usize) {
        let mut changed = false;

        for yline in 0..height {
            let pixel = self.memory[self.index_register as usize + yline];
            for xline in 0..8 {
                if (pixel & (0x80 >> xline)) != 0 {
                    if self.bitmap[(x + xline + ((y + yline) * 64))] == 1 {
                        changed = true;
                    }
                    self.bitmap[x + xline + ((y + yline) * 64)] ^= 1;
                }
            }
        }

        self.registers[0xf] = if changed { 1 } else { 0 };
        self.should_draw = true;
    }
}

fn integer_from_key(key: &Key) -> Option<u8> {
    match key {
        Key::Key1 => Some(0x1),
        Key::Key2 => Some(0x2),
        Key::Key3 => Some(0x3),
        Key::Key4 => Some(0xC),

        Key::Q => Some(0x4),
        Key::W => Some(0x5),
        Key::E => Some(0x6),
        Key::R => Some(0xD),

        Key::A => Some(0x7),
        Key::S => Some(0x8),
        Key::D => Some(0x9),
        Key::F => Some(0xE),

        Key::Z => Some(0xA),
        Key::X => Some(0x0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),
        _ => None,
    }
}

fn main() -> io::Result<()> {
    let window = Window::new("chip8", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    let mut emulator = Emulator::init(window);
    emulator.load_file("Breakout [Carmelo Cortez, 1979].ch8".as_ref())?;

    emulator.do_game_loop();

    Ok(())
}
