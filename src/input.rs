use std::cell::{Cell, RefCell};
use sdl2::{event::Event, keyboard::Scancode, Sdl};

pub trait Input {
    fn esc_pressed(&self) -> bool;
    fn is_key_pressed(&self, key: u8) -> bool;
    fn get_last_press(&self) -> Option<u8>;
}

pub struct Sdl2Input {
    keys: RefCell<[u8; 16]>,
    last_pressed_key: Cell<Option<u8>>,
    event_pump: RefCell<sdl2::EventPump>,
    exit: Cell<bool>,
}

impl Sdl2Input {
    pub fn new(sdl_context: &Sdl) -> Result<Self, String> {
        let event_pump = sdl_context.event_pump()?;

        Ok(Self {
            keys: RefCell::new([0; 16]),
            last_pressed_key: Cell::new(None),
            event_pump: RefCell::new(event_pump),
            exit: Cell::new(false),
        })
    }

    pub fn update_keys(&self) {
        let mut input_char: Option<u8> = None;

        for event in self.event_pump.borrow_mut().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => {
                    self.exit.set(true);
                    return;
                }
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    input_char = match scancode {
                        Scancode::Num1 => Some(0x1),
                        Scancode::Num2 => Some(0x2),
                        Scancode::Num3 => Some(0x3),
                        Scancode::Num4 => Some(0xC),
                        Scancode::Q => Some(0x4),
                        Scancode::W => Some(0x5),
                        Scancode::E => Some(0x6),
                        Scancode::R => Some(0xD),
                        Scancode::A => Some(0x7),
                        Scancode::S => Some(0x8),
                        Scancode::D => Some(0x9),
                        Scancode::F => Some(0xE),
                        Scancode::Z => Some(0xA),
                        Scancode::X => Some(0x0),
                        Scancode::C => Some(0xB),
                        Scancode::V => Some(0xF),
                        _ => None,
                    };
                }
                _ => {}
            }
        }
        if let Some(c) = input_char {
            self.last_pressed_key.set(input_char);

            // value controls key stickiness
            self.keys.borrow_mut()[c as usize] = 16;
        }
    }

    pub fn decrement_keys(&self) {
        for x in self.keys.borrow_mut().iter_mut() {
            if *x > 0 {
                *x -= 1;
            }
        }
    }
}

impl Input for Sdl2Input {
    fn esc_pressed(&self) -> bool {
        self.exit.get()
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        self.keys.borrow_mut()[key as usize] != 0
    }

    fn get_last_press(&self) -> Option<u8> {
        let last_press = self.last_pressed_key.get();
        self.last_pressed_key.set(None);
        last_press
    }
}