use crate::{input::Input,renderer::Renderer};
use std::fs;

pub struct CPU<'a, I, R>
where
    I: 'a + Input,
    R: 'a + Renderer,
{
    ram: [u8; 4096],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    v: [u8; 16],
    video_ram: [[u8; 64]; 32],
    renderer: &'a mut R,
    input: &'a I,
}

impl<'a, I, R> CPU<'a, I, R>
where
    I: 'a + Input,
    R: 'a + Renderer,
{
    pub fn new(renderer: &'a mut R, input: &'a I) -> Result<Self, String> {
        let mut cpu = Self {
            ram: [0; 4096],
            pc: 0x200,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            v: [0; 16],
            video_ram: [[0; 64]; 32],
            renderer,
            input,
        };
        cpu.load_font_sprites();
        Ok(cpu)
    }

    fn load_font_sprites(&mut self) {
        let font_sprites: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for (idx, val) in font_sprites.iter().enumerate() {
            self.ram[idx + 0x50] = *val;
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), String> {
        let rom_bytes: Vec<u8> = fs::read(path).map_err(|e| e.to_string())?;
        for (idx, val) in rom_bytes.iter().enumerate() {
            self.ram[idx + 0x200] = *val;
        }
        Ok(())
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn cycle(&mut self) -> Result<(), String> {
        // fetch
        let next_inst =
            u16::from_be_bytes([self.ram[self.pc as usize], self.ram[(self.pc + 1) as usize]]);
        self.pc += 2;

        // execute
        self.execute_instruction(next_inst)?;

        Ok(())
    }

    fn execute_instruction(&mut self, inst: u16) -> Result<(), String> {
        let first_nibble = inst / 0x1000 % 0x10;
        let x = (inst / 0x0100 % 0x10) as usize;
        let y = (inst / 0x0010 % 0x10) as usize;
        let n = (inst % 0x10) as u8;
        let nn = (inst % 0x100) as u8;
        let nnn = inst % 0x1000;

        match first_nibble {
            0x0 => match inst {
                // 00E0: Clear Screen
                0x00E0 => {
                    for i in 0..64 {
                        for j in 0..32 {
                            self.video_ram[j][i] = 0;
                        }
                    }
                    self.renderer.render_screen_ram(self.video_ram)?;
                }
                // 00EE: Return from a subroutine
                // Pops the last address from the stack, and sets pc to it
                0x00EE => {
                    self.pc = self.stack.pop().unwrap();
                }
                _ => {}
            },
            // 1NNN: Jump to memory location
            // Sets pc to NNN
            0x1 => {
                self.pc = nnn;
            }
            // 2NNN: Call subroutine at memory location NNN.
            // Pushes the current pc onto the stack, and sets pc to NNN.
            0x2 => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            // 3XNN: Skip th next instruction if VX is equal to NN.
            // Increments pc by 2 if VX and NN are equal.
            0x3 => {
                if self.v[x] == nn as u8 {
                    self.pc += 2;
                }
            }
            // 4XNN: Skip the next instruction if VX is not equal to NN.
            // Increments pc by 2 if VX and NN are not equal.
            0x4 => {
                if self.v[x] != nn as u8 {
                    self.pc += 2;
                }
            }
            // 5XY0: Skip the next instruction if VX and VY are equal.
            // Increments pc by 2 if VX and VY are equal.
            0x5 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            // 6XNN: Set the register VX to the value NN.
            0x6 => {
                self.v[x] = nn as u8;
            }
            // 7XNN: Add the value NN to VX.
            0x7 => {
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            0x8 => match n {
                // 8XY0: Set the register VX to the value of VY
                0x0 => {
                    self.v[x] = self.v[y];
                }
                // 8XY1: Set VX to the bitwise OR of VX and VY
                0x1 => {
                    self.v[x] |= self.v[y];
                }
                // 8XY2: Set VX to the bitwise AND of VX and VY
                0x2 => {
                    self.v[x] &= self.v[y];
                }
                // 8XY3: Set VX to the bitwise XOR of VX and VY
                0x3 => {
                    self.v[x] ^= self.v[y];
                }
                // 8XY4: Set VX to the sum of VX and VY.
                // Carry flag is set if there is an overflow.
                0x4 => {
                    let r = self.v[x] as u16 + self.v[y] as u16;
                    self.v[x] = r as u8;
                    self.v[0xF] = (r > 0xFF) as u8;
                }
                // 8XY5: Set VX to the difference between VX and VY.
                // Carry flag is set if there is no borrow.
                0x5 => {
                    let vf = (self.v[x] > self.v[y]) as u8;
                    self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                    self.v[0xF] = vf;
                }
                // 8XY6: Right shift VX in place.
                // Vf is set to the bit that was shifted out.
                0x6 => {
                    let vf = self.v[x] & 1;
                    self.v[x] >>= 1;
                    self.v[0xF] = vf;
                }
                // 8XY7: Set VX to the difference between VY and VX.
                // Carry flag is set if there is no borrow.
                0x7 => {
                    let vf = (self.v[y] > self.v[x]) as u8;
                    self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                    self.v[0xF] = vf;
                }
                // 8XYE: Left shift VX in place.
                // Vf is set the bit that was shifted out.
                0xE => {
                    let vf = self.v[x] >> 7;
                    self.v[x] <<= 1;
                    self.v[0xF] = vf;
                }
                _ => {}
            },
            // 9XY0: Skip the next instructin if VX and VY are NOT equal.
            // Increments pc by 2 if VX and VY are unequal.
            0x9 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            // ANNN: Set the index register to the value nnn
            // Sets i to NNN;
            0xA => {
                self.i = nnn;
            }
            // BNNN: Jump with offset
            // Jumps to address NNN, plus the value in register V0.
            // Sets pc to NNN plus value in V0.
            // Sets pc to NNN plus value in V0.
            0xB => {
                self.pc = nnn + self.v[0] as u16;
                // CNNN: Random
                // Generates a random number, binary ANDs it with the value nn, and puts the result in VX.
            }
            // Generates a random number, binary ANDs it with the value nn, and puts the result in VX.
            0xC => {
                let random = rand::random::<u8>();
                // DXYN: Display
                // Draws an N pixels tall sprite from memory location held in the index register,
                // at the X coordinate in VX and Y coordinate in VY.
                self.v[x] = random & nn;
            }
            // Draws an N pixels tall sprite from memory location held in the index register, at the
            // horizontal coordinate in VX and Y coordinate in VY.
            0xD => {
                let x = (self.v[x] % 64) as usize;
                let y = (self.v[y] % 32) as usize;

                self.v[0xF] = 0;

                for i in 0..n {
                    let y_pos = y + i as usize;
                    if y_pos > 31 {
                        break;
                    }
                    let sprite_row_addr = self.i + i as u16;
                    let sprite_row = self.ram[sprite_row_addr as usize];
                    for j in 0..8 {
                        let x_pos = x + j;
                        if x_pos > 63 {
                            break;
                        }
                        let bit_at_index = (sprite_row << j) >> 7;
                        self.v[0xF] = self.video_ram[y_pos][x_pos];
                        self.video_ram[y_pos][x_pos] ^= bit_at_index;
                    }
                }
                self.renderer.render_screen_ram(self.video_ram)?;
            }
            0xE => match nn {
                // EX9E: Skip next instruction if the key corresponding to the value in VX is pressed.
                // Increments pc by 2 if the key corresponding to the value in VX is pressed.
                0x9E => {
                    if self.input.is_key_pressed(self.v[x]) {
                        self.pc += 2;
                    }
                }
                // EXA1: Skip next instruction if the key corresponding to the value in VX is NOT pressed.
                // Increments pc by 2 if the key corresponding to the value in VX is not pressed.
                0xA1 => {
                    if !(self.input.is_key_pressed(self.v[x])) {
                        self.pc += 2;
                    }
                }
                _ => {}
            },
            0xF => match nn {
                // FX07: Set VX to the current value of the delay timer.
                0x07 => {
                    self.v[x] = self.delay_timer;
                }
                0x0A => {
                    if let Some(key) = self.input.get_last_press() {
                        self.v[x] = key;
                    } else {
                        self.pc -= 2;
                    }
                }
                // FX15: Set the delay timer to the value in VX,
                0x15 => {
                    self.delay_timer = self.v[x];
                }
                // FX18: Set the sound timer to the value in VX,
                0x18 => {
                    self.sound_timer = self.v[x];
                }
                // FX1E: Add the value in VX to the index register
                0x1E => {
                    self.i += self.v[x] as u16;
                    if self.i > 0xFFF {
                        self.i %= 0x1000;
                        self.v[0xF] = 1;
                    }
                }
                // FX29: Font character
                // Sets the index register is set to the address of the hexadecimal character in VX.
                0x29 => {
                    let char = (self.v[x] & 0xF) as u16;
                    self.i = 0x50 + char * 5;
                }
                // FX33: Binary-coded decimal conversion
                // Takes the number in VX (which is an 8-bit number, so ranges from 0 to 255), and
                // converts it into three decimal digits, storing these digits in memory at the address
                // in the index register. The first digit is stored at the address in I, the second digit
                // is stored in address I + 1, and the third at address I + 2
                0x33 => {
                    for i in 0..3 {
                        let digit = self.v[x] / 10u8.pow(2 - i) % 10;
                        self.ram[self.i as usize + i as usize] = digit;
                    }
                }
                // FX55: Store memory
                // Stores the value of each register from V0 to VX in memory in successive memory
                // addresses, starting from the address in I till I + x.
                0x55 => {
                    for i in 0..(x + 1) {
                        let idx = self.i as usize + i;
                        self.ram[idx] = self.v[i];
                    }
                }
                // FX65: Load memory
                // Same as FX55, but loads values stored at the memory addresses into the
                // registers instead.
                0x65 => {
                    for i in 0..(x + 1) {
                        let idx = self.i as usize + i;
                        self.v[i] = self.ram[idx];
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }
}
