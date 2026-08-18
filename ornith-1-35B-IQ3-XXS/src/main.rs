mod track;
mod player;
mod ai;
mod sprites;
mod renderer;
mod sound;
mod game;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Set window attributes for retro feel
    let window = video_subsystem.window("Ornith Racer - Lotus Style Pseudo-3D Racer", 640, 480)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // Wait a moment for window to appear
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut game = game::Game::new(&sdl_context, &mut canvas);
    game.run(&mut canvas, &sdl_context);
}
