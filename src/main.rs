use std::{convert::TryFrom, fs, io, path::Path};

use rand::Rng;

struct Emulator {
    opcode: u16,
    memory: [u8; 4096],
    registers: [u8; 16],
    index_register: u16,
    program_counter: u16,
    bitmap: [bool; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: u16,
    key: [bool; 16],
    should_draw: bool,
}

impl Emulator {
    pub fn init() -> Self {
        Self {
            program_counter: 0x200,
            opcode: 0,
            index_register: 0,
            stack_pointer: 0,
            memory: [0; 4096],
            registers: [0; 16],
            bitmap: [false; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            key: [false; 16],
            should_draw: false,
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
        loop {
            self.emulate_cycle();

            if self.should_draw {
                self.draw_graphics();
            }

            self.set_keys(&[]);
        }
    }

    fn draw_graphics(&mut self) {
        todo!()
    }

    fn set_keys(&mut self, _keys: &[u8]) {
        // noop
    }

    fn get_key(&mut self) -> u8 {
        b'A'
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

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00EE => todo!("{:#X}", opcode),
                0x00E0 => todo!("{:#X}", opcode),
                _ => todo!("{:#X}", opcode),
            },
            0x1000 => {
                let address = opcode & 0x0FFF;

                self.program_counter = address;
            }
            0x2000 => todo!("{:#X}", opcode),
            0x3000 => {
                let register = self.registers[usize::from(opcode & 0x0F00) >> 8];

                let value = u8::try_from(opcode & 0x00FF).unwrap();

                if register == value {
                    self.program_counter += 2;
                }
            }
            0x4000 => {
                let register = self.registers[usize::from(opcode & 0x0F00) >> 8];

                let value = u8::try_from(opcode & 0x00FF).unwrap();

                if register != value {
                    self.program_counter += 2;
                }
            }
            0x5000 => todo!("{:#X}", opcode),
            0x6000 => {
                let register = usize::from(opcode & 0x0F00) >> 8;
                let register_value = u8::try_from(opcode & 0x00FF).unwrap();

                self.registers[register] = register_value;
            }
            0x7000 => {
                let register = self.get_register_x_mut(opcode);
                let value = opcode & 0x00FF;

                *register = (*register as u16 + value) as u8;
            }
            0x8000 => match opcode & 0x000F {
                0x0001 => todo!("{:#X}", opcode),
                0x0002 => {
                    let register_y = *self
                        .registers
                        .get(usize::from(opcode & 0x00F0) >> 4)
                        .unwrap();

                    let register_x = self.get_register_x_mut(opcode);

                    *register_x = *register_x | register_y;
                }
                0x0003 => todo!("{:#X}", opcode),
                0x0004 => {
                    let register_y = self.get_register_y(opcode);

                    let register_x = self.get_register_x_mut(opcode);

                    *register_x = register_x.wrapping_add(register_y);
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
                let x = usize::from(opcode & 0x0F00) >> 8;
                let y = usize::from(opcode & 0x00F0) >> 4;
                let height = usize::from(opcode & 0x000F);

                self.draw(x, y, height);
            }
            0xE000 => match opcode & 0x00FF {
                0x009E => {
                    let register = self.get_register_x(opcode);

                    let key_exists = self.key[usize::from(register)];

                    if key_exists {
                        self.program_counter += 2;
                    }
                }
                0x00A1 => {
                    let register = self.get_register_x(opcode);

                    let key_exists = self.key[usize::from(register)];

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
                0x0018 => todo!("{:#X}", opcode),
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
                println!("Beep!");
            }
            self.sound_timer -= 1;
        }
    }

    /// Draws a sprite at coordinate (x, y) that has a width of 8 pixels
    /// and a height of N pixels. Each row of 8 pixels is read as bit-coded
    /// starting from memory location I; I value doesn’t change after the
    /// execution of this instruction. As described above, VF is set to 1 if
    /// any screen pixels are flipped from set to unset when the sprite is
    /// drawn, and to 0 if that doesn’t happen
    fn draw(&mut self, x: usize, y: usize, height: usize) {}
}

fn main() -> io::Result<()> {
    let mut emulator = Emulator::init();
    emulator.load_file("Breakout [Carmelo Cortez, 1979].ch8".as_ref())?;

    emulator.do_game_loop();

    Ok(())
}
