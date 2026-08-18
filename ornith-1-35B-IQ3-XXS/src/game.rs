use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::track::Track;
use crate::player::PlayerState;
use crate::ai::AIState;
use crate::sprites::{Sprite, generate_scenery};
use crate::renderer::Renderer;
use crate::sound::SoundEngine;

pub enum GameState {
    Title,
    Countdown(u32),
    Racing,
    Results,
    Pause,
}

pub struct Game {
    state: GameState,
    track: Track,
    player: PlayerState,
    ai_cars: Vec<AIState>,
    sprites: Vec<Sprite>,
    renderer: Renderer,
    sound_engine: SoundEngine,
    race_time: f32,
    lap: u32,
    total_laps: u32,
    countdown_timer: f32,
    keys_pressed: Vec<i32>,
}

impl Game {
    pub fn new(_sdl_ctx: &sdl2::Sdl, _canvas: &mut Canvas<Window>) -> Self {
        let track = Track::new();
        let sprites = generate_scenery(track.total_length);
        
        let mut ai_cars = Vec::new();
        for i in 0..5 {
            ai_cars.push(AIState::new(i));
        }

        let renderer = Renderer::new();
        let sound_engine = SoundEngine::new();

        Game {
            state: GameState::Title,
            track,
            player: PlayerState::new(),
            ai_cars,
            sprites,
            renderer,
            sound_engine,
            race_time: 0.0,
            lap: 1,
            total_laps: 3,
            countdown_timer: 0.0,
            keys_pressed: Vec::new(),
        }
    }

    pub fn run(&mut self, canvas: &mut Canvas<Window>, sdl_ctx: &sdl2::Sdl) {
        let mut last_time = std::time::Instant::now();
        
        loop {
            let now = std::time::Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32();
            last_time = now;

            // Handle events
            if let Ok(mut event_pump) = sdl_ctx.event_pump() {
                self.handle_events(canvas, &mut event_pump);
            }

            // Update game state
            match self.state {
                GameState::Title => {
                    self.update_title(canvas);
                }
                GameState::Countdown(_) => {
                    self.countdown_timer += dt;
                    if self.countdown_timer >= 1.0 {
                        self.countdown_timer = 0.0;
                        // Could play beep sound here
                    }
                }
                GameState::Racing => {
                    self.update_racing(dt, canvas);
                }
                GameState::Results => {
                    self.update_results(canvas);
                }
                GameState::Pause => {
                    self.update_pause(canvas);
                }
            }

            // Render current state
            self.render(canvas);

            // Present frame
            canvas.present();

            // Cap at ~60 FPS
            let target = std::time::Duration::from_millis(16);
            if now.elapsed() < target {
                std::thread::sleep(target - now.elapsed());
            }
        }
    }

    fn handle_events(&mut self, canvas: &mut Canvas<Window>, event_pump: &mut sdl2::EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    std::process::exit(0);
                }
                Event::KeyDown { keycode, .. } => {
                    if let Some(k) = keycode {
                        self.keys_pressed.push(*k);
                        
                        match *k {
                            27 => { // ESCAPE
                                match self.state {
                                    GameState::Racing | GameState::Countdown(_) => {
                                        self.state = GameState::Pause;
                                    }
                                    GameState::Pause => {
                                        self.state = GameState::Racing;
                                    }
                                    _ => {}
                                }
                            }
                            13 | 32 => { // RETURN or SPACE
                                match self.state {
                                    GameState::Title => {
                                        self.start_race();
                                    }
                                    GameState::Results => {
                                        self.reset_game();
                                        self.state = GameState::Title;
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(k) = keycode {
                        self.keys_pressed.retain(|&key| key != *k);
                    }
                }
                _ => {}
            }
        }
    }

    fn start_race(&mut self) {
        self.player = PlayerState::new();
        for ai in &mut self.ai_cars {
            *ai = AIState::new(ai.id);
        }
        self.race_time = 0.0;
        self.lap = 1;
        self.state = GameState::Countdown(3);
        self.countdown_timer = 0.0;
    }

    fn reset_game(&mut self) {
        self.player = PlayerState::new();
        for ai in &mut self.ai_cars {
            *ai = AIState::new(ai.id);
        }
        self.race_time = 0.0;
        self.lap = 1;
    }

    fn update_title(&mut self, canvas: &mut Canvas<Window>) {
        // Title screen rendering handled in render()
    }

    fn update_racing(&mut self, dt: f32, canvas: &mut Canvas<Window>) {
        // Handle input
        let mut steer = 0.0f32;
        let mut accelerating = false;
        let mut braking = false;
        let mut handbrake = false;

        for key in &self.keys_pressed {
            match *key {
                273 | 119 => accelerating = true, // Up arrow or W
                274 | 115 => braking = true,      // Down arrow or S
                276 | 97 => steer -= 1.0,         // Left arrow or A
                275 | 100 => steer += 1.0,        // Right arrow or D
                32 => handbrake = true,           // SPACE
                _ => {}
            }
        }

        // Update player
        self.player.steer = steer;
        self.player.handbrake = handbrake;
        if accelerating {
            self.player.speed += 500.0 * dt;
        } else if braking {
            self.player.speed -= 400.0 * dt;
        } else {
            self.player.speed -= 100.0 * dt; // Natural deceleration
        }
        self.player.speed = self.player.speed.max(0.0).min(400.0);

        self.player.update(&self.track, dt);

        // Update AI
        for ai in &mut self.ai_cars {
            ai.update(&self.track, self.player.z, dt);
        }

        // Update race time
        self.race_time += dt;

        // Check lap completion
        let prev_lap_z = self.lap as f32 * self.track.total_length;
        if self.player.z < prev_lap_z {
            self.lap += 1;
            
            if self.lap > self.total_laps {
                self.state = GameState::Results;
            }
        }

        // Update sound
        self.sound_engine.update(self.player.speed, self.race_time);
    }

    fn update_results(&mut self, canvas: &mut Canvas<Window>) {
        // Results screen rendering handled in render()
    }

    fn update_pause(&mut self, canvas: &mut Canvas<Window>) {
        // Pause menu rendering handled in render()
    }

    fn render(&mut self, canvas: &mut Canvas<Window>) {
        match self.state {
            GameState::Title => {
                self.render_title(canvas);
            }
            GameState::Countdown(_) => {
                self.render_countdown(canvas);
            }
            GameState::Racing => {
                self.renderer.render(
                    canvas,
                    &self.track,
                    &self.player,
                    &self.ai_cars,
                    &self.sprites,
                    self.race_time,
                    self.lap,
                    self.total_laps,
                );
            }
            GameState::Results => {
                self.render_results(canvas);
            }
            GameState::Pause => {
                self.render_pause(canvas);
            }
        }
    }

    fn render_title(&mut self, canvas: &mut Canvas<Window>) {
        // Dark background
        canvas.set_draw_color(Color::RGB(10, 10, 30));
        canvas.clear();

        // Title text (simplified - just rectangles for now)
        let title_y = 150;
        
        // "ORNITH RACER" in large block letters
        canvas.set_draw_color(Color::RGB(255, 200, 0));
        for i in 0..40 {
            canvas.fill_rect(Rect::new(200 + i * 12, title_y, 8, 30)).ok();
        }

        // Subtitle
        canvas.set_draw_color(Color::RGB(200, 200, 200));
        for i in 0..30 {
            canvas.fill_rect(Rect::new(250 + i * 8, title_y + 60, 4, 15)).ok();
        }

        // Instructions
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for i in 0..20 {
            canvas.fill_rect(Rect::new(280 + i * 6, 350, 3, 12)).ok();
        }

        // Blinking "Press Enter" text
        let blink = (self.race_time as u32 % 60) < 30;
        if blink {
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            for i in 0..15 {
                canvas.fill_rect(Rect::new(300 + i * 8, 400, 5, 10)).ok();
            }
        }
    }

    fn render_countdown(&mut self, canvas: &mut Canvas<Window>) {
        // Render the race scene
        self.renderer.render(
            canvas,
            &self.track,
            &self.player,
            &self.ai_cars,
            &self.sprites,
            self.race_time,
            self.lap,
            self.total_laps,
        );

        // Draw countdown number
        let count = 3u32 - (self.countdown_timer as u32 % 3);
        if count > 0 {
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            // Simple number rendering (would need proper font for real implementation)
            let size = 100;
            let x = 320 - size / 2;
            let y = 240 - size / 2;
            canvas.fill_rect(Rect::new(x, y, size as u32, size as u32)).ok();
        }
    }

    fn render_results(&mut self, canvas: &mut Canvas<Window>) {
        // Dark background
        canvas.set_draw_color(Color::RGB(10, 10, 30));
        canvas.clear();

        // Results text
        canvas.set_draw_color(Color::RGB(255, 200, 0));
        for i in 0..30 {
            canvas.fill_rect(Rect::new(220 + i * 10, 100, 6, 25)).ok();
        }

        // Player position
        let player_pos = self.calculate_final_position();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        
        // Time display
        let time_text = format!("Time: {:.2}s", self.race_time);
        // Would need proper text rendering here
        
        // "Press Enter to restart"
        let blink = (self.race_time as u32 % 60) < 30;
        if blink {
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            for i in 0..20 {
                canvas.fill_rect(Rect::new(280 + i * 6, 400, 3, 12)).ok();
            }
        }
    }

    fn render_pause(&mut self, canvas: &mut Canvas<Window>) {
        // Render the race scene (dimmed)
        self.renderer.render(
            canvas,
            &self.track,
            &self.player,
            &self.ai_cars,
            &self.sprites,
            self.race_time,
            self.lap,
            self.total_laps,
        );

        // Dim overlay
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 128));
        canvas.fill_rect(Rect::new(0, 0, 640, 480)).ok();

        // "PAUSED" text
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for i in 0..25 {
            canvas.fill_rect(Rect::new(250 + i * 10, 200, 7, 20)).ok();
        }

        // "Press ESC to resume"
        let blink = (self.race_time as u32 % 60) < 30;
        if blink {
            canvas.set_draw_color(Color::RGB(200, 200, 200));
            for i in 0..25 {
                canvas.fill_rect(Rect::new(240 + i * 5, 280, 3, 10)).ok();
            }
        }
    }

    fn calculate_final_position(&self) -> u32 {
        let mut position = 1;
        
        for ai in &self.ai_cars {
            if ai.z > self.player.z {
                position += 1;
            } else if (ai.z + self.track.total_length) > self.player.z {
                position += 1;
            }
        }

        position.min(6)
    }
}
