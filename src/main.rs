extern crate sdl2;

use input::Input;
use std::env;
use std::time::Duration;

pub mod cpu;
pub mod input;
pub mod renderer;

const CYCLE_FREQ: u32 = 720;
const TIMER_DECREMENT_FREQ: u32 = 60;

pub fn main() -> Result<(), String> {
    let program_name = env::args().nth(0).unwrap();
    let rom_path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("Usage: {} <rom>", program_name);
            std::process::exit(1);
        }
    };

    let sdl_context = sdl2::init()?;

    let mut renderer = renderer::Sdl2Renderer::new(&sdl_context)?;
    let input = input::Sdl2Input::new(&sdl_context)?;

    let mut cpu = cpu::CPU::new(&mut renderer, &input)?;

    cpu.load_rom(&rom_path)?;

    let mut cycle_count = 0;
    'running: loop {
        cpu.cycle()?;
        cycle_count += 1;

        if cycle_count % (CYCLE_FREQ / TIMER_DECREMENT_FREQ) == 0 {
            cpu.decrement_timers();
            cycle_count = 0;
        }

        input.update_keys();
        input.decrement_keys();

        if input.esc_pressed() {
            break 'running;
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / CYCLE_FREQ));
    }

    Ok(())
}
