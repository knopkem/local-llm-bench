use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

mod game;
mod renderer;
mod track;
mod car;
mod ai;
mod input;
mod audio;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Lotus Esprit Turbo Challenge Style", 640, 480)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut game_state = game::GameState::new();
    let mut input_state = input::InputState::new();
    
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(k), .. } => {
                    match k {
                        Keycode::Up | Keycode::W => input_state.accelerate = true,
                        Keycode::Down | Keycode::S => input_state.brake = true,
                        Keycode::Left | Keycode::A => input_state.steer_left = true,
                        Keycode::Right | Keycode::D => input_state.steer_right = true,
                        _ => {}
                    }
                }
                Event::KeyUp { keycode: Some(k), .. } => {
                    match k {
                        Keycode::Up | Keycode::W => input_state.accelerate = false,
                        Keycode::Down | Keycode::S => input_state.brake = false,
                        Keycode::Left | Keycode::A => input_state.steer_left = false,
                        Keycode::Right | Keycode::D => input_state.steer_right = false,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        game_state.update(&input_state);
        renderer::render(&mut canvas, &game_state);
        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }
}
