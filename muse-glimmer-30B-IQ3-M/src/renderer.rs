use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub fn render(canvas: &mut Canvas<Window>, game_state: &crate::game::GameState) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    
    // Sky
    canvas.set_draw_color(Color::RGB(100, 150, 200));
    canvas.fill_rect(Some(sdl2::rect::Rect::new(0, 0, 640, 240))).unwrap();
    
    // Road perspective with proper fill
    canvas.set_draw_color(Color::RGB(80, 80, 80));
    for i in 0..120 {
        let y = 240 + i * 3;
        if y >= 480 { break; }
        let width = 300 - i * 2;
        if width > 10 {
            canvas.fill_rect(Some(sdl2::rect::Rect::new(
                320 - width / 2,
                y as i32,
                width as u32,
                3
            ))).unwrap();
        }
    }
    
    // Road lines
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for i in 0..60 {
        let y = 240 + i * 8;
        if y < 480 {
            canvas.fill_rect(Some(sdl2::rect::Rect::new(320, y as i32, 2, 2))).unwrap();
        }
    }
    
    // Draw cars
    let player_z = game_state.player.z;
    for car in std::iter::once(&game_state.player).chain(game_state.opponents.iter()) {
        let dz = car.z - player_z;
        if dz >= 0.0 && dz < 500.0 {
            let scale = 1.0 / (dz * 0.01 + 1.0);
            let screen_y = 240 + (dz as i32) / 2;
            let lane_offset = car.lane * 40;
            let screen_x = 320 + lane_offset;
            
            if screen_y >= 240 && screen_y < 480 && scale > 0.1 {
                let size = (scale * 20.0).max(4.0) as u32;
                if car.is_player {
                    canvas.set_draw_color(Color::RGB(255, 0, 0));
                } else {
                    canvas.set_draw_color(Color::RGB(0, 200, 255));
                }
                canvas.fill_rect(Some(sdl2::rect::Rect::new(
                    screen_x as i32 - size as i32 / 2,
                    screen_y - size as i32 / 2,
                    size,
                    size
                ))).unwrap();
            }
        }
    }
    
    // HUD
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.fill_rect(Some(sdl2::rect::Rect::new(10, 10, 100, 20))).unwrap();
    
    // Speed display
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let speed_text = format!("SPEED: {:.0}", game_state.player.speed);
    // Simple placeholder - actual text rendering would need font library
}

