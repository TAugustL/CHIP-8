use rand::{Rng, rngs::ThreadRng};
use sdl2::{
    AudioSubsystem,
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    keyboard::{KeyboardState, Scancode},
    pixels::Color,
    rect::Point,
    render::WindowCanvas,
    video::Window,
};
use std::error::Error;

pub const WINDOW_SIZE: (u32, u32) = (1024, 512);
pub const LOGICAL_WINDOW_SIZE: (u32, u32) = (64, 32);
pub const TARGET_IPS: u32 = 700;

pub struct Chip8Context {
    renderer: Renderer,
    memory: [u8; 4096],
    display: [[bool; 64]; 32],
    program_counter: usize,
    i: u16,
    stack: Vec<usize>,
    delay_timer: DTimer,
    sound_timer: STimer,
    register: [u8; 16],
    random_device: ThreadRng,
    keypad: [bool; 16],
}

impl Chip8Context {
    pub fn new(
        renderer: Renderer,
        audio: &AudioSubsystem,
        game_file: Vec<u8>,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Chip8Context {
            renderer,
            memory: init_memory(game_file),
            display: [[false; 64]; 32],
            program_counter: INSTR_OFFSET,
            i: 0,
            stack: Vec::with_capacity(16),
            delay_timer: DTimer::new(),
            sound_timer: STimer::new(audio)?,
            register: [0; 16],
            random_device: rand::thread_rng(),
            keypad: [false; 16],
        })
    }
    const fn start_delay(&mut self, duration: u32) {
        self.delay_timer.time = duration * (TARGET_IPS / 60);
    }
    const fn start_sound(&mut self, duration: u32) {
        self.sound_timer.time = duration * (TARGET_IPS / 60);
    }
    fn process_instructions(&mut self) {
        let instr1 = self.memory[self.program_counter];
        let instr2 = self.memory[self.program_counter + 1];
        let instr = ((instr1 as u16) << 8) + instr2 as u16;
        self.program_counter += 2;
        self.decode_instruction(instr);
        if self.program_counter >= 4096 {
            self.program_counter = INSTR_OFFSET;
        }
    }
    fn decode_instruction(&mut self, opcode: u16) {
        let vx: usize = bit_i(opcode, 1) as usize;
        let vy: usize = bit_i(opcode, 2) as usize;
        let nnn: u16 = opcode & 0xFFF;
        let nn: u8 = (opcode & 0xFF) as u8;
        let n: u8 = (opcode & 0xF) as u8;

        match bit_i(opcode, 0) {
            0x0 => {
                match nnn {
                    // CLEAR SCREEN
                    0x0E0 => self.clear_screen(true),
                    // RETURN FROM SUBROUTINE
                    0x0EE => {
                        self.program_counter = self.stack.pop().unwrap();
                    }
                    _ => (),
                }
            }
            0x1 => {
                // JUMP
                let mem_location = nnn;
                self.program_counter = mem_location as usize;
            }
            0x2 => {
                // SUBROUTINE
                let mem_location = nnn;
                self.stack.push(self.program_counter);
                self.program_counter = mem_location as usize;
            }
            0x3 => {
                // SKIP IF VX == NN
                if self.register[vx] == nn {
                    self.program_counter += 2;
                }
            }
            0x4 => {
                // SKIP IF VX != NN
                if self.register[vx] != nn {
                    self.program_counter += 2;
                }
            }
            0x5 => {
                // SKIP IF VX == VY
                if self.register[vx] == self.register[vy] {
                    self.program_counter += 2;
                }
            }
            0x6 => {
                // SET REGISTER
                self.register[vx] = nn;
            }
            0x7 => {
                // ADD TO REGISTER
                self.register[vx] = ((self.register[vx] as u16 + nn as u16) % (0xFF + 1)) as u8;
            }
            0x8 => {
                match n {
                    0x0 => {
                        // SET
                        self.register[vx] = self.register[vy];
                    }
                    0x1 => {
                        // BINARY OR
                        self.register[vx] |= self.register[vy];
                    }
                    0x2 => {
                        // BINARY AND
                        self.register[vx] &= self.register[vy];
                    }
                    0x3 => {
                        // LOGICAL XOR
                        self.register[vx] ^= self.register[vy];
                    }
                    0x4 => {
                        // ADD
                        if self.register[vx].checked_add(self.register[vy]).is_none() {
                            self.register[0xF] = 1;
                            self.register[vx] = ((self.register[vx] as u16
                                + self.register[vy] as u16)
                                % (0xFF + 1)) as u8;
                        } else {
                            self.register[0xF] = 0;
                            self.register[vx] += self.register[vy];
                        }
                    }
                    0x5 => {
                        // SUBTRACT Y FROM X
                        if self.register[vx] > self.register[vy] {
                            self.register[0xF] = 1;
                        } else {
                            self.register[0xF] = 0;
                        }
                        self.register[vx] -= self.register[vy];
                    }
                    0x7 => {
                        // SUBTRACT X FROM Y
                        if self.register[vy] > self.register[vx] {
                            self.register[0xF] = 1;
                        } else {
                            self.register[0xF] = 0;
                        }
                        self.register[vx] = self.register[vy] - self.register[vx];
                    }
                    0x6 => {
                        // SHIFT RIGHT
                        #[cfg(feature = "alt_shift")]
                        {
                            self.register[vx] = self.register[vy];
                        }

                        let entry = self.register[vx];
                        self.register[vx] >>= 1;

                        let shifted_off: u8 = entry & 0x1;
                        self.register[0xF] = shifted_off;
                    }
                    0xE => {
                        // SHIFT LEFT
                        #[cfg(feature = "alt_shift")]
                        {
                            self.register[vx] = self.register[vy];
                        }
                        let entry = self.register[vx];
                        self.register[vx] <<= 1;

                        let shifted_off: u8 = (entry & 0x80) >> 7;
                        self.register[0xF] = shifted_off;
                    }
                    _ => (),
                }
            }
            0x9 => {
                // SKIP IF VX != VY
                if self.register[vx] != self.register[vy] {
                    self.program_counter += 2;
                }
            }
            0xA => {
                // SET INDEX REGISTER
                self.i = nnn;
            }
            0xB => {
                // JUMP WITH OFFSET
                let mem_location = nnn;
                #[cfg(feature = "alt_jump")]
                {
                    let offset_regx = self.register[vx];
                    self.program_counter = (mem_location + offset_regx as u16) as usize;
                }
                #[cfg(not(feature = "alt_jump"))]
                {
                    let offset_reg0 = self.register[0];
                    self.program_counter = (mem_location + offset_reg0 as u16) as usize;
                }
            }
            0xC => {
                // RANDOM
                let random_number: u8 = self.random_device.gen_range(0x0..0xFF);
                let final_value = random_number & nn;
                self.register[vx] = final_value;
            }
            0xD => {
                // DISPLAY/DRAW
                let x = self.register[vx] % 64;
                let mut curr_y = self.register[vy] % 32;

                self.register[0xF] = 0;
                for byte in self.get_mem_region(self.i as usize, (self.i + (opcode & 0xF)) as usize)
                {
                    if curr_y >= 32 {
                        return;
                    }
                    let mut curr_x = x;
                    for bit in 0..8 {
                        if x + bit >= 64 {
                            break;
                        }

                        let new_pixel = (byte >> (8 - bit - 1)) & 0b1;
                        let old_pixel = self.read_pixel_at(curr_x, curr_y) as u8;

                        if new_pixel == 1 && old_pixel == 0 {
                            self.draw_pixel_at(curr_x, curr_y).unwrap();
                        } else if new_pixel == 1 && old_pixel == 1 {
                            self.register[0xF] = 1;
                            self.remove_pixel_at(curr_x, curr_y).unwrap();
                        }

                        curr_x += 1;
                    }
                    curr_y += 1;
                }
            }
            0xE => match nn {
                0x9E => {
                    // SKIP IF PRESSED
                    if self.keypad[self.register[vx] as usize] {
                        self.program_counter += 2;
                    }
                }
                0xA1 => {
                    // SKIP IF NOT PRESSED
                    if !self.keypad[self.register[vx] as usize] {
                        self.program_counter += 2;
                    }
                }
                _ => (),
            },
            0xF => match nn {
                0x07 => {
                    // READ DELAY
                    self.register[vx] = self.delay_timer.time as u8;
                }
                0x15 => {
                    // START DELAY
                    self.start_delay(self.register[vx] as u32);
                }
                0x18 => {
                    // START SOUND
                    self.start_sound(self.register[vx] as u32);
                }
                0x1E => {
                    // ADD TO INDEX
                    self.i += self.register[vx] as u16;
                    if self.i > 0xFFF {
                        self.register[0xF] = 1;
                    }
                }
                0x0A => {
                    // GET KEY
                    if self.keypad.iter().all(|b| !b) {
                        self.program_counter -= 2;
                    } else {
                        for k_i in 0..self.keypad.len() {
                            if self.keypad[k_i] {
                                self.register[vx] = k_i as u8;
                                break;
                            }
                        }
                    }
                }
                0x29 => {
                    // FONT CHAR
                    self.i = FONT_OFFSET as u16 + 5 * self.register[vx] as u16;
                }
                0x33 => {
                    // BINARY CODED DECIMAL CONVERSION
                    let val: u8 = self.register[vx];
                    let d1: u8 = ((val / 10) / 10) % 10;
                    let d2: u8 = (val / 10) % 10;
                    let d3: u8 = val % 10;
                    self.memory[self.i as usize] = d1;
                    self.memory[self.i as usize + 1] = d2;
                    self.memory[self.i as usize + 2] = d3;
                }
                0x55 => {
                    // STORE REGISTERS IN MEMORY
                    for i in 0..=vx {
                        let val = self.register[i];
                        #[cfg(feature = "alt_store_load")]
                        {
                            self.i += 1;
                            self.memory[self.i as usize] = val;
                        }

                        #[cfg(not(feature = "alt_store_load"))]
                        {
                            self.memory[(self.i + i as u16) as usize] = val;
                        }
                    }
                }
                0x65 => {
                    // STORE MEMORY IN REGISTERS
                    for i in 0..=vx {
                        #[cfg(feature = "alt_store_load")]
                        {
                            self.i += 1;
                            self.register[i] = self.memory[self.i as usize];
                        }
                        #[cfg(not(feature = "alt_store_load"))]
                        {
                            self.register[i] = self.memory[(self.i + i as u16) as usize];
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
    fn get_mem_region(&self, start: usize, end: usize) -> Vec<u8> {
        self.memory[start..end].to_vec()
    }
    fn clear_screen(&mut self, present: bool) {
        self.renderer.canvas.set_draw_color(Color::BLACK);
        self.renderer.canvas.clear();
        if present {
            self.display = [[false; 64]; 32];
            self.renderer.to_be_rendered = vec![];
            self.renderer.canvas.present();
        }
    }
    fn draw_pixel_at(&mut self, x: u8, y: u8) -> Result<(), Box<dyn Error>> {
        self.renderer
            .to_be_rendered
            .push(Point::new(x as i32, y as i32));
        self.renderer.draw()?;
        self.display[y as usize][x as usize] = true;
        Ok(())
    }
    fn remove_pixel_at(&mut self, x: u8, y: u8) -> Result<(), Box<dyn Error>> {
        for (i, pixel) in self.renderer.to_be_rendered.iter().enumerate() {
            if pixel.x == x as i32 && pixel.y == y as i32 {
                self.renderer.to_be_rendered.remove(i);
                self.display[y as usize][x as usize] = false;
                self.clear_screen(false);
                self.renderer.draw()?;
                return Ok(());
            }
        }
        Ok(())
    }
    const fn read_pixel_at(&self, x: u8, y: u8) -> bool {
        self.display[y as usize][x as usize]
    }
    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_instructions();

        self.delay_timer.update();
        self.sound_timer.update();
        self.keypad = [false; 16];

        Ok(())
    }
    pub fn process_keyboard_input(&mut self, keycodes: KeyboardState) {
        for keypress in keycodes.pressed_scancodes() {
            match keypress {
                Scancode::Num1 => self.keypad[0x0] = true,
                Scancode::Num2 => self.keypad[0x1] = true,
                Scancode::Num3 => self.keypad[0x2] = true,
                Scancode::Num4 => self.keypad[0xC] = true,
                Scancode::Q => self.keypad[0x4] = true,
                Scancode::W => self.keypad[0x5] = true,
                Scancode::E => self.keypad[0x6] = true,
                Scancode::R => self.keypad[0xD] = true,
                Scancode::A => self.keypad[0x7] = true,
                Scancode::S => self.keypad[0x8] = true,
                Scancode::D => self.keypad[0x9] = true,
                Scancode::F => self.keypad[0xE] = true,
                Scancode::Y | Scancode::Z => self.keypad[0xA] = true,
                Scancode::X => self.keypad[0x0] = true,
                Scancode::C => self.keypad[0xB] = true,
                Scancode::V => self.keypad[0xF] = true,
                _ => (),
            }
        }
    }
}

struct DTimer {
    time: u32,
}

struct STimer {
    time: u32,
    beep_device: AudioDevice<SquareWave>,
    playing: bool,
}

impl DTimer {
    const fn new() -> Self {
        DTimer { time: 0 }
    }
    const fn update(&mut self) {
        if self.time > 0 {
            self.time -= 1;
        }
    }
}

impl STimer {
    fn new(audio: &AudioSubsystem) -> Result<Self, Box<dyn Error>> {
        let desired_spec = AudioSpecDesired {
            freq: Some(22050),
            channels: Some(1),
            samples: None,
        };
        let beep_device = audio.open_playback(None, &desired_spec, |spec| SquareWave {
            phase_inc: 220.0 / spec.freq as f32,
            phase: 0.0,
            volme: 0.1,
        })?;

        Ok(STimer {
            time: 0,
            beep_device,
            playing: false,
        })
    }
    fn update(&mut self) {
        if self.time > 0 {
            if !self.playing {
                self.playing = true;
                self.beep_device.resume();
            }
            self.time -= 1;
        } else {
            self.playing = false;
            self.beep_device.pause();
        }
    }
}

pub struct Renderer {
    canvas: WindowCanvas,
    to_be_rendered: Vec<Point>,
}

impl Renderer {
    pub fn new(window: Window) -> Result<Self, Box<dyn Error>> {
        let mut canvas = window.into_canvas().accelerated().build()?;
        canvas.set_logical_size(LOGICAL_WINDOW_SIZE.0, LOGICAL_WINDOW_SIZE.1)?;
        let to_be_rendered = Vec::new();

        Ok(Renderer {
            canvas,
            to_be_rendered,
        })
    }

    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        self.canvas.set_draw_color(Color::WHITE);
        for point in &self.to_be_rendered {
            self.canvas.draw_point(*point)?;
        }

        self.canvas.present();
        Ok(())
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volme: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volme
            } else {
                -self.volme
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

const FONT_OFFSET: usize = 0x050;
const INSTR_OFFSET: usize = 0x200;
const FONT: [u8; 80] = [
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

fn init_memory(program_bytes: Vec<u8>) -> [u8; 4096] {
    let mut memory = [0; 4096];

    // Skip 0x000 - 0x1FF

    // Font 0x050 - 0x0A0
    memory[FONT_OFFSET..(FONT.len() + FONT_OFFSET)].copy_from_slice(&FONT[..]);

    // INST 0x200 - 0xFFF
    memory[INSTR_OFFSET..(program_bytes.len() + INSTR_OFFSET)].copy_from_slice(&program_bytes[..]);

    memory
}

const fn bit_i(byte: u16, i: u16) -> u8 {
    ((byte >> (12 - i * 4)) & 0xF) as u8
}
