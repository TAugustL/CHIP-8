use chip_8::{Chip8Context, Renderer};
use sdl2::{event::Event, keyboard::Keycode};
use std::{env::args, error::Error, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;
    let window = video_subsystem
        .window("CHIP-8", chip_8::WINDOW_SIZE.0, chip_8::WINDOW_SIZE.1)
        .position_centered()
        .vulkan()
        .build()?;
    let mut event_pump = sdl_context.event_pump()?;

    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("No CHIP-8 file supplied as an argument!");
        return Ok(());
    }
    let file = std::fs::read(&args[1]).expect("Invalid file path!");

    let renderer = Renderer::new(window)?;
    let mut chip_8_context = Chip8Context::new(renderer, &audio_subsystem, file)?;

    'running: loop {
        chip_8_context.process_keyboard_input(event_pump.keyboard_state());
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::ESCAPE),
                    ..
                } => break 'running,
                _ => (),
            }
        }

        chip_8_context.update()?;
        sleep(Duration::from_nanos(
            (1_000_000_000 / chip_8::TARGET_IPS) as u64,
        ));
    }

    Ok(())
}
