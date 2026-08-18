use std::fs;
use std::path::PathBuf;

use crate::assets::{Assets, K_CAR};
use crate::game::car::TOP_SPEED;
use crate::game::race::{fmt_time, Events, Mode, RaceState};
use crate::game::track::{Track, SEG_LEN};
use crate::game::InputState;
use crate::render::background::{render_hills, render_sky};
use crate::render::framebuffer::Renderer;
use crate::render::font::text_width;
use crate::render::hud::render_hud;
use crate::render::road::{project_sprite, render_road, HORIZON_Y};
use crate::render::sprite::blit_scaled;
use crate::render::{draw_text, H, W};

/// Discrete key actions (mapped from platform keys in main).
#[derive(Clone, Copy, PartialEq)]
pub enum Key {
    Up, Down, Left, Right, Enter, Esc, Mode, Pause,
}

/// A running race/attract scene: simulation state + hill parallax offset.
pub struct Scene {
    pub race: RaceState,
    hill_off: f32,
}

impl Scene {
    /// Attract mode: all cars self-drive (used as the menu backdrop).
    pub fn attract(track: &Track, track_idx: usize) -> Self {
        Scene { race: RaceState::attract(track, track_idx), hill_off: 0.0 }
    }

    pub fn new_race(track: &Track, mode: Mode, track_idx: usize, difficulty: u8) -> Self {
        Scene { race: RaceState::new(track, mode, track_idx, difficulty), hill_off: 0.0 }
    }

    pub fn update(&mut self, dt: f32, input: &InputState, track: &Track) -> Events {
        let ev = self.race.update(dt, input, track);
        if !self.race.over && self.race.countdown <= 0.0 {
            let sr = (self.race.player.speed / TOP_SPEED).clamp(0.0, 1.0);
            let curve = track.curve_at(self.race.player.pos_wrapped(track.length));
            self.hill_off -= curve * 3.0 * sr;
        }
        ev
    }

    /// Render the full scene: sky, hills, road, sprites, HUD.
    pub fn render(&mut self, r: &mut Renderer, assets: &Assets) {
        let (track, theme) = &assets.tracks[self.race.track_idx];
        let pw = self.race.player.pos_wrapped(track.length);

        render_sky(r, HORIZON_Y, &theme.sky);
        render_hills(r, &theme.hills_img, self.hill_off as i32, HORIZON_Y);

        // Pre-fill the horizon gap: far segments converge a few rows below
        // HORIZON_Y; paint distant ground so no black band shows through.
        r.rect(0, HORIZON_Y, (W - 1) as i32, HORIZON_Y + 16, theme.road.grass_b);

        let view = render_road(r, track, pw, self.race.player.x, &theme.road);

        // Scenery sprites, far to near (painter's algorithm).
        for sp in view.segs.iter().rev() {
            let seg = &track.segments[sp.index];
            if seg.sprites.is_empty() {
                continue;
            }
            for s in &seg.sprites {
                let Some((scale, sx, sy)) = project_sprite(track, &view, pw, self.race.player.x, sp.index, 0.5, s.offset) else {
                    continue;
                };
                let spr = &assets.scenery[s.kind as usize];
                let dw = (assets.k_scenery[s.kind as usize] * scale).max(1.0);
                let ss = spr.w as f32 / dw;
                let dh = ((spr.h as f32) * dw / spr.w as f32).ceil() as i32;
                blit_scaled(r, spr, sx - (dw / 2.0).round() as i32, sy - dh + 1, ss, sp.clip);
            }
        }

        // AI cars, far to near.
        for ai in self.race.ais.iter().rev() {
            let aw = ai.pos.rem_euclid(track.length);
            let seg_i = track.find_segment(aw);
            if !view.segs.iter().any(|s| s.index == seg_i) {
                continue;
            }
            let t = (aw - seg_i as f32 * SEG_LEN) / SEG_LEN;
            let Some((scale, sx, sy)) = project_sprite(track, &view, pw, self.race.player.x, seg_i, t, ai.x) else {
                continue;
            };
            let clip = view.segs.iter().find(|s| s.index == seg_i).map(|s| s.clip).unwrap_or(-1);
            let spr = &assets.ai_cars[ai.car_sprite as usize];
            let dw = (K_CAR * scale).max(1.0);
            let ss = spr.w as f32 / dw;
            let dh = ((spr.h as f32) * dw / spr.w as f32).ceil() as i32;
            blit_scaled(r, spr, sx - (dw / 2.0).round() as i32, sy - dh + 1, ss, clip);
        }

        // Player car: fixed screen position, nudged by steering direction.
        {
            let spr = &assets.player_car;
            let dw: f32 = 74.0;
            let ss = spr.w as f32 / dw;
            let dh = ((spr.h as f32) * dw / spr.w as f32).ceil() as i32;
            let dx = W as i32 / 2 - (dw / 2.0).round() as i32 + (self.race.steer_dir * 5.0).round() as i32;
            blit_scaled(r, spr, dx, H as i32 - dh - 6, ss, -1);
        }

        render_hud(r, &self.race, assets.txt_white, assets.txt_dim);
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State { Title, Select, Race, Results }

struct SelOpts {
    track_idx: usize,
    mode: Mode,
    difficulty: u8,
}

pub struct App {
    pub assets: Assets,
    pub r: Renderer,
    /// Attract-mode backdrop that always runs behind the menus.
    bg: Scene,
    scene: Option<Scene>,
    state: State,
    sel: SelOpts,
    paused: bool,
    frame: u32,
    best_laps: [Option<f64>; 2],
    pending: Events,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SaveData {
    best_laps: [Option<f64>; 2],
}

fn save_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".lotus_racer.json")
}

impl App {
    pub fn new(assets: Assets, pal: crate::render::framebuffer::Palette) -> Self {
        let r = Renderer::with_palette(W, H, pal);
        let (track0, _) = &assets.tracks[0];
        let bg = Scene::attract(track0, 0);
        let best_laps = fs::read_to_string(save_path())
            .ok()
            .and_then(|s| serde_json::from_str::<SaveData>(&s).ok())
            .map(|d| d.best_laps)
            .unwrap_or([None, None]);
        App {
            assets,
            r,
            bg,
            scene: None,
            state: State::Title,
            sel: SelOpts { track_idx: 0, mode: Mode::Race, difficulty: 1 },
            paused: false,
            frame: 0,
            best_laps,
            pending: Events::default(),
        }
    }

    /// Handle a discrete key action. Returns true if the app should quit.
    pub fn handle_key(&mut self, k: Key) -> bool {
        match self.state {
            State::Title => match k {
                Key::Enter => self.state = State::Select,
                Key::Esc => return true,
                _ => {}
            },
            State::Select => match k {
                Key::Up | Key::Down => self.sel.track_idx ^= 1,
                Key::Left => self.sel.difficulty = (self.sel.difficulty + 2) % 3,
                Key::Right => self.sel.difficulty = (self.sel.difficulty + 1) % 3,
                Key::Mode => {
                    self.sel.mode = match self.sel.mode {
                        Mode::Race => Mode::TimeTrial,
                        Mode::TimeTrial => Mode::Race,
                    };
                }
                Key::Enter => {
                    let (track, _) = &self.assets.tracks[self.sel.track_idx];
                    self.scene = Some(Scene::new_race(track, self.sel.mode, self.sel.track_idx, self.sel.difficulty));
                    self.state = State::Race;
                    self.paused = false;
                }
                Key::Esc => self.state = State::Title,
                _ => {}
            },
            State::Race => match k {
                Key::Pause => self.paused = !self.paused,
                Key::Esc => {
                    self.scene = None;
                    self.state = State::Select;
                }
                _ => {}
            },
            State::Results => {
                if matches!(k, Key::Enter | Key::Esc) {
                    self.scene = None;
                    self.state = State::Select;
                }
            }
        }
        false
    }

    /// Advance the simulation by dt. The attract backdrop always runs.
    pub fn update(&mut self, dt: f32, input: &InputState) {
        let t0 = &self.assets.tracks[0].0;
        let _ = self.bg.update(dt, &InputState::none(), t0);

        if self.state == State::Race && !self.paused {
            let Some(scene) = self.scene.as_mut() else { return };
            let (track, _) = &self.assets.tracks[scene.race.track_idx];
            let ev = scene.update(dt, input, track);
            if let Some(lt) = ev.player_lap {
                let i = scene.race.track_idx;
                if self.best_laps[i].map_or(true, |b| lt < b) {
                    self.best_laps[i] = Some(lt);
                    fs::write(save_path(), serde_json::to_string(&SaveData { best_laps: self.best_laps }).unwrap()).ok();
                }
            }
            if ev.finish && scene.race.over {
                self.state = State::Results;
            }
            self.pending = ev;
        }
    }

    /// Drain the events produced by the last update (for SFX in main).
    pub fn take_events(&mut self) -> Events {
        let e = self.pending;
        self.pending = Events::default();
        e
    }

    /// Period-correct palette cycling on the active track's sky bands.
    fn cycle_sky(&mut self, track_idx: usize) {
        let v = ((self.frame / 6) % 4) as usize;
        let theme = &self.assets.tracks[track_idx].1;
        for i in 0..theme.sky.bands.len() {
            self.r.pal.set(theme.sky.bands[i] as usize, theme.sky_cycle[i][v]);
        }
    }

    pub fn render(&mut self) {
        self.frame += 1;
        let in_race = self.state == State::Race && self.scene.is_some();
        let track_idx = if in_race {
            self.scene.as_ref().unwrap().race.track_idx
        } else {
            0
        };
        self.cycle_sky(track_idx);

        if in_race {
            self.scene.as_mut().unwrap().render(&mut self.r, &self.assets);
            if self.paused {
                let r = &mut self.r;
                r.rect(0, H as i32 / 2 - 16, (W - 1) as i32, H as i32 / 2 + 16, 0);
                draw_text(r, W as i32 / 2 - text_width("PAUSED", 4) / 2, H as i32 / 2 - 8, "PAUSED", self.assets.txt_white, 4);
            }
        } else {
            self.bg.render(&mut self.r, &self.assets);
            match self.state {
                State::Title => self.draw_title(),
                State::Select => self.draw_select(),
                State::Results => self.draw_results(),
                _ => {}
            }
        }
    }

    fn draw_title(&mut self) {
        let r = &mut self.r;
        let white = self.assets.txt_white;
        let dim = self.assets.txt_dim;
        r.rect(0, 48, (W - 1) as i32, 116, 0);
        draw_text(r, W as i32 / 2 - text_width("LOTUS ESPRIT", 5) / 2, 58, "LOTUS ESPRIT", white, 5);
        draw_text(r, W as i32 / 2 - text_width("TURBO CHALLENGE", 2) / 2, 96, "TURBO CHALLENGE", dim, 2);
        if (self.frame / 20) % 2 == 0 {
            draw_text(r, W as i32 / 2 - text_width("PRESS ENTER", 2) / 2, 140, "PRESS ENTER", white, 2);
        }
        draw_text(r, 8, H as i32 - 26, "ARROWS/WASD DRIVE   ENTER OK   ESC BACK", dim, 1);
    }

    fn draw_select(&mut self) {
        let r = &mut self.r;
        let white = self.assets.txt_white;
        let dim = self.assets.txt_dim;
        let x0 = W as i32 / 2 - 150;
        let y0 = 64;
        r.rect(x0 - 1, y0 - 1, x0 + 301, y0 + 129, white);
        r.rect(x0, y0, x0 + 300, y0 + 128, 0);

        draw_text(r, x0 + 16, y0 + 14, "TRACK", dim, 1);
        for (i, name) in ["COASTAL CIRCUIT", "MOUNTAIN PASS"].iter().enumerate() {
            let cur = if i == self.sel.track_idx { "> " } else { "  " };
            draw_text(r, x0 + 16, y0 + 28 + (i as i32) * 14, &format!("{cur}{name}"), white, 1);
        }

        let mode_s = match self.sel.mode { Mode::Race => "RACE", Mode::TimeTrial => "TIME TRIAL" };
        draw_text(r, x0 + 16, y0 + 62, &format!("MODE   {mode_s}"), white, 1);
        let diff_s = ["EASY", "NORMAL", "HARD"][self.sel.difficulty as usize];
        draw_text(r, x0 + 16, y0 + 76, &format!("DIFFICULTY   {diff_s}"), white, 1);

        if (self.frame / 20) % 2 == 0 {
            draw_text(r, x0 + 16, y0 + 104, "ENTER = START", white, 1);
        }
    }

    fn draw_results(&mut self) {
        let Some(scene) = &self.scene else { return };
        let r = &mut self.r;
        let white = self.assets.txt_white;
        let dim = self.assets.txt_dim;
        let x0 = W as i32 / 2 - 150;
        let y0 = 48;
        r.rect(x0 - 1, y0 - 1, x0 + 301, y0 + 161, white);
        r.rect(x0, y0, x0 + 300, y0 + 160, 0);

        draw_text(r, W as i32 / 2 - text_width("RESULTS", 3) / 2, y0 + 10, "RESULTS", white, 3);
        for (i, (_, name, t)) in scene.race.results.iter().enumerate() {
            let line = format!("{:>2}. {:<14} {}", i + 1, name, fmt_time(*t));
            draw_text(r, x0 + 16, y0 + 34 + (i as i32) * 14, &line, white, 1);
        }
        if let Some(b) = scene.race.best_lap {
            draw_text(r, x0 + 16, y0 + 92, &format!("BEST LAP {}", fmt_time(b)), dim, 1);
        }
        if (self.frame / 20) % 2 == 0 {
            draw_text(r, W as i32 / 2 - text_width("ENTER = CONTINUE", 1) / 2, y0 + 140, "ENTER = CONTINUE", white, 1);
        }
    }

    /// Player speed as a fraction of top speed (for engine pitch).
    pub fn player_speed_frac(&self) -> f32 {
        let s = self.scene.as_ref().map(|s| s.race.player.speed).unwrap_or(0.0);
        (s / TOP_SPEED).clamp(0.0, 1.0)
    }

    /// Track index of the active race scene (for music selection), if any.
    pub fn music_track(&self) -> Option<usize> {
        if self.state == State::Race {
            Some(self.scene.as_ref()?.race.track_idx)
        } else {
            None
        }
    }

    /// RGBA bytes of the current frame (for texture upload / PNG export).
    pub fn rgba(&self) -> Vec<u8> {
        self.r.rgba()
    }
}
