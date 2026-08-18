use macroquad::prelude::*;
use ::rand::{thread_rng, Rng};

const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;
const ROAD_WIDTH: f32 = 2000.0;
const SEGMENT_LENGTH: f32 = 200.0;
const COLORS: [Color; 2] = [Color::new(0.4, 0.4, 0.4, 1.0), Color::new(0.5, 0.5, 0.5, 1.0)];
const GRASS_COLORS: [Color; 2] = [Color::new(0.1, 0.6, 0.1, 1.0), Color::new(0.1, 0.5, 0.1, 1.0)];

struct Segment {
    index: usize,
    curve: f32,
}

struct Car {
    z: f32,
    x: f32, // -1 to 1 relative to road center
    speed: f32,
    color: Color,
}

struct GameState {
    segments: Vec<Segment>,
    track_length: usize,
    player: Car,
    opponents: Vec<Car>,
    camera_depth: f32,
}

impl GameState {
    fn new() -> Self {
        let mut segments = Vec::new();
        let track_length = 1000;
        for i in 0..track_length {
            let curve = if (i > 100 && i < 300) || (i > 600 && i < 800) {
                (i as f32 * 0.05).sin() * 2.0
            } else if i > 400 && i < 500 {
                1.5
            } else if i > 800 && i < 900 {
                -1.5
            } else {
                0.0
            };
            segments.push(Segment { index: i, curve });
        }

        let mut opponents = Vec::new();
        for i in 0..5 {
            opponents.push(Car {
                z: (i + 1) as f32 * 2000.0,
                x: thread_rng().gen_range(-0.5..0.5),
                speed: 180.0 + thread_rng().gen_range(0.0..40.0),
                color: RED,
            });
        }

        GameState {
            segments,
            track_length,
            player: Car { z: 0.0, x: 0.0, speed: 0.0, color: BLUE },
            opponents,
            camera_depth: 0.8,
        }
    }

    fn update(&mut self) {
        if is_key_down(KeyCode::Up) {
            self.player.speed += 2.0;
        } else if is_key_down(KeyCode::Down) {
            self.player.speed -= 5.0;
        } else {
            self.player.speed -= 1.0;
        }

        if is_key_down(KeyCode::Left) {
            self.player.x -= 0.02 * (self.player.speed / 200.0).max(0.1);
        }
        if is_key_down(KeyCode::Right) {
            self.player.x += 0.02 * (self.player.speed / 200.0).max(0.1);
        }

        self.player.speed = self.player.speed.clamp(0.0, 300.0);
        self.player.z += self.player.speed;

        for opp in &mut self.opponents {
            opp.z += opp.speed;
            if opp.z > self.track_length as f32 * SEGMENT_LENGTH {
                opp.z = 0.0;
            }
        }

        if self.player.z > self.track_length as f32 * SEGMENT_LENGTH {
            self.player.z -= self.track_length as f32 * SEGMENT_LENGTH;
        }
    }

    fn project(&self, world_x: f32, world_y: f32, relative_z: f32) -> Vec2 {
        if relative_z <= 0.0 { return Vec2::ZERO; }
        let scale = self.camera_depth / relative_z;
        Vec2::new(
            (SCREEN_WIDTH / 2.0) + (world_x * scale * SCREEN_WIDTH / 2.0),
            (SCREEN_HEIGHT / 2.0) + (world_y * scale * SCREEN_HEIGHT / 2.0),
        )
    }

    fn draw(&self) {
        clear_background(SKYBLUE);

        let start_index = (self.player.z / SEGMENT_LENGTH) as usize;
        
        // Pre-calculate offsets for the visible segments relative to player's position
        let mut offsets = Vec::with_capacity(201);
        let mut current_x = 0.0;
        let mut current_curve = 0.0;
        
        for i in 0..201 {
            let segment_idx = (start_index + i) % self.track_length;
            current_curve += self.segments[segment_idx].curve;
            current_x += current_curve;
            offsets.push(current_x);
        }

        // Draw road segments from back to front (Painter's Algorithm)
        for i in (0..200).rev() {
            let segment_idx = (start_index + i) % self.track_length;
            let relative_z = (start_index as f32 * SEGMENT_LENGTH) + (i as f32 * SEGMENT_LENGTH) - self.player.z;

            if relative_z <= 0.0 { continue; }

            let p1 = self.project(offsets[i], 0.0, relative_z);
            let p2 = self.project(offsets[i+1], 0.0, relative_z + SEGMENT_LENGTH);

            let w1 = (self.camera_depth / relative_z) * ROAD_WIDTH;
            let w2 = (self.camera_depth / (relative_z + SEGMENT_LENGTH)) * ROAD_WIDTH;

            draw_rectangle(0.0, p1.y, SCREEN_WIDTH, p2.y - p1.y, GRASS_COLORS[segment_idx % 2]);

            let left = p1.x - w1 / 2.0;
            let right = p1.x + w1 / 2.0;
            let next_left = p2.x - w2 / 2.0;
            let next_right = p2.x + w2 / 2.0;

            draw_triangle(Vec2::new(left, p1.y), Vec2::new(right, p1.y), Vec2::new(next_left, p2.y), COLORS[segment_idx % 2]);
            draw_triangle(Vec2::new(right, p1.y), Vec2::new(next_right, p2.y), Vec2::new(next_left, p2.y), COLORS[segment_idx % 2]);
        }

        // Draw Opponents
        for opp in &self.opponents {
            let relative_z = opp.z - self.player.z;
            if relative_z > 0.0 && relative_z < 10000.0 {
                let scale = self.camera_depth / relative_z;
                
                // Find the offset for this opponent's Z position
                let opp_relative_idx = (relative_z / SEGMENT_LENGTH) as usize;
                let world_x_offset = if opp_relative_idx < offsets.len() {
                    offsets[opp_relative_idx]
                } else {
                    // Approximate offset for far away cars by using the last known offset
                    offsets[offsets.len()-1]
                };

                let screen_x = (SCREEN_WIDTH / 2.0) + (opp.x * ROAD_WIDTH + world_x_offset) * scale * SCREEN_WIDTH / 2.0;
                let screen_y = (SCREEN_HEIGHT / 2.0) + (0.0 - 1500.0) * scale * SCREEN_HEIGHT / 2.0;
                let size = 100.0 * scale * 5.0;
                draw_rectangle(screen_x - size/2.0, screen_y - size, size, size, opp.color);
            }
        }

        // Draw Player Car (Fixed position)
        let p_size = 100.0;
        draw_rectangle(SCREEN_WIDTH / 2.0 - p_size / 2.0, SCREEN_HEIGHT - p_size - 20.0, p_size, p_size, self.player.color);

        // HUD
        draw_text(&format!("Speed: {:.0}", self.player.speed), 20.0, 30.0, 30.0, WHITE);
    }
}

#[macroquad::main("Lotus Racer")]
async fn main() {
    let mut game = GameState::new();

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
