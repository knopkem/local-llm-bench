use super::ai::{AiCar, CarSnap};
use super::car::{grid_pos, PlayerCar, TOP_SPEED};
use super::track::Track;
use super::InputState;

pub const RACE_LAPS: u32 = 3;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Race,
    TimeTrial,
}

/// One-shot events produced by a simulation step (for SFX / UI).
#[derive(Default, Clone, Copy)]
pub struct Events {
    /// Player crossed the start line with this lap time.
    pub player_lap: Option<f64>,
    /// Countdown tick to announce (3, 2, 1) or None.
    pub beep: Option<u8>,
    /// The countdown just ended ("GO").
    pub go: bool,
    /// A car finished the race this step.
    pub finish: bool,
    /// Player bumped into an AI car.
    pub bump: bool,
}

pub struct RaceState {
    pub track_idx: usize,
    pub mode: Mode,
    pub player: PlayerCar,
    pub ais: Vec<AiCar>,
    /// Countdown remaining (controls locked while > 0).
    pub countdown: f64,
    /// Race clock in seconds (runs after the countdown).
    pub time: f64,
    last_lap_start: f64,
    pub lap_times: Vec<f64>,
    pub best_lap: Option<f64>,
    /// Results: (car id 0 = player, name, total time) in finishing order.
    pub results: Vec<(usize, &'static str, f64)>,
    pub over: bool,
    /// Cached display values (updated each step).
    pub pos_rank: usize,
    pub laps_done: u32,
    /// When true the player car is driven by autopilot (attract mode).
    pub autopilot: bool,
    bump_cd: f64,
    /// Last steering direction -1..1 (for the car sprite offset).
    pub steer_dir: f32,
}

const DRIVER_NAMES: [&'static str; 3] = ["H. SURTEES", "A. DE CESARIS", "M. ANDRETTI"];

impl RaceState {
    /// difficulty: 0 easy, 1 normal, 2 hard.
    pub fn new(track: &Track, mode: Mode, track_idx: usize, difficulty: u8) -> Self {
        let grid = grid_pos(track.length, 3); // player starts last (P4)
        let mut ais = Vec::new();
        for i in 0..3 {
            let top_scale = match difficulty {
                0 => 0.72 + i as f32 * 0.05,
                1 => 0.86 + i as f32 * 0.04,
                _ => 0.97 + i as f32 * 0.02,
            };
            let skill = match difficulty {
                0 => 0.35 + i as f32 * 0.1,
                1 => 0.6 + i as f32 * 0.08,
                _ => 0.82 + i as f32 * 0.05,
            };
            ais.push(AiCar {
                id: i + 1,
                name: DRIVER_NAMES[i],
                car_sprite: i as u8,
                pos: grid_pos(track.length, i),
                x: [-0.45, 0.0, 0.45][i],
                speed: 0.0,
                top_speed: TOP_SPEED * top_scale.min(1.02),
                skill: skill.min(0.95),
                base_offset: [-0.3, 0.0, 0.3][i],
                grid_pos: grid_pos(track.length, i),
                finished: false,
                finish_time: 0.0,
            });
        }

        RaceState {
            track_idx,
            mode,
            player: PlayerCar::new(grid),
            ais,
            countdown: 3.5,
            time: 0.0,
            last_lap_start: 0.0,
            lap_times: Vec::new(),
            best_lap: None,
            results: Vec::new(),
            over: false,
            pos_rank: 4,
            laps_done: 0,
            autopilot: false,
            bump_cd: 0.0,
            steer_dir: 0.0,
        }
    }

    /// Attract-mode scene: all cars (including the player slot) self-drive.
    pub fn attract(track: &Track, track_idx: usize) -> Self {
        let mut s = RaceState::new(track, Mode::Race, track_idx, 1);
        s.countdown = 0.0;
        s.autopilot = true;
        s
    }

    pub fn player_position_display(&self) -> usize {
        self.pos_rank
    }

    pub fn player_laps_display(&self) -> u32 {
        self.laps_done
    }

    /// Simulate one step. Returns events for SFX/UI.
    pub fn update(&mut self, dt: f32, input: &InputState, track: &Track) -> Events {
        let mut ev = Events::default();
        if self.over {
            return ev;
        }

        // Countdown phase: cars hold on the grid.
        if self.countdown > 0.0 {
            let before = self.countdown.ceil() as u8;
            self.countdown -= dt as f64;
            let after = self.countdown.max(0.0).ceil() as u8;
            if after < before && after <= 3 {
                ev.beep = Some(after);
            }
            if self.countdown <= 0.0 {
                ev.go = true;
            }
            return ev;
        }

        self.time += dt as f64;

        // Player.
        let laps_before = ((self.player.pos - grid_pos(track.length, 3)) / track.length).floor().max(0.0) as u32;
        if self.autopilot {
            self.steer_dir = (-self.player.x).signum();
            self.player.autopilot(dt, track);
        } else {
            self.steer_dir = (input.right as u8) as f32 - (input.left as u8) as f32;
            self.player.update(dt, input, track);
        }
        let laps_after = ((self.player.pos - grid_pos(track.length, 3)) / track.length).floor().max(0.0) as u32;
        if laps_after > laps_before {
            let lap_time = self.time - self.last_lap_start;
            self.last_lap_start = self.time;
            self.lap_times.push(lap_time);
            if self.best_lap.map_or(true, |b| lap_time < b) {
                self.best_lap = Some(lap_time);
            }
            ev.player_lap = Some(lap_time);
        }

        // AI.
        let snaps: Vec<CarSnap> = std::iter::once(CarSnap {
            pos: self.player.pos,
            x: self.player.x,
            speed: self.player.speed,
        })
        .chain(self.ais.iter().map(|a| CarSnap {
            pos: a.pos,
            x: a.x,
            speed: a.speed,
        }))
        .collect();

        for ai in &mut self.ais {
            if !ai.finished && ai.laps_done(track.length) >= RACE_LAPS {
                ai.finished = true;
                ai.finish_time = self.time;
                ev.finish = true;
            }
            let others: Vec<CarSnap> = snaps.iter().filter(|s| s.pos != ai.pos || s.x != ai.x).copied().collect();
            ai.update(dt, track, &others);
        }

        // Player-AI bumps (rear-end / side contact).
        self.bump_cd -= dt as f64;
        if self.bump_cd <= 0.0 && !self.autopilot {
            for ai in &mut self.ais {
                let d = ai.pos - self.player.pos;
                if (10.0..320.0).contains(&d)
                    && (ai.x - self.player.x).abs() < 0.45
                    && self.player.speed > ai.speed + 60.0
                {
                    self.player.speed *= 0.86;
                    ai.speed = (ai.speed * 1.05).min(ai.top_speed);
                    ev.bump = true;
                    self.bump_cd = 1.2;
                    break;
                }
            }
        }

        // Player finishing the race.
        if self.mode == Mode::Race {
            let player_laps = ((self.player.pos - grid_pos(track.length, 3)) / track.length).floor() as u32;
            if player_laps >= RACE_LAPS && !self.results.iter().any(|(id, _, _)| *id == 0) {
                self.results.push((0, "YOU", self.time));
                ev.finish = true;
            }
        }

        // Collect finished AI results in finishing order.
        let mut done: Vec<&AiCar> = self.ais.iter().filter(|a| a.finished).collect();
        done.sort_by(|a, b| a.finish_time.partial_cmp(&b.finish_time).unwrap_or(std::cmp::Ordering::Equal));
        for ai in done {
            if !self.results.iter().any(|(id, _, _)| *id == ai.id) {
                self.results.push((ai.id, ai.name, ai.finish_time));
            }
        }

        // Race ends once everyone has finished (or 60s after the winner as a
        // safety cap); cars still on track are ranked below as DNF.
        if self.mode == Mode::Race && !self.results.is_empty() {
            let all_done = self.ais.iter().all(|a| a.finished) && self.results.iter().any(|(id, _, _)| *id == 0);
            let winner_time = self.results[0].2;
            if all_done || self.time - winner_time > 60.0 {
                for ai in &self.ais {
                    if !ai.finished && !self.results.iter().any(|(id, _, _)| *id == ai.id) {
                        self.results.push((ai.id, ai.name, f64::MAX));
                    }
                }
                self.over = true;
            }
        }

        // Cache display values.
        let grid3 = grid_pos(track.length, 3);
        self.laps_done = ((self.player.pos - grid3) / track.length).floor().max(0.0) as u32;
        let pt = (self.player.pos - grid3).max(0.0);
        let mut ahead = 0usize;
        for ai in &self.ais {
            if (ai.pos - ai.grid_pos).max(0.0) > pt {
                ahead += 1;
            }
        }
        self.pos_rank = ahead + 1;

        ev
    }

}

/// Format seconds as M:SS.cc (or --:-- for DNF).
pub fn fmt_time(t: f64) -> String {
    if t >= f64::MAX / 2.0 {
        return "--:--".to_string();
    }
    let m = (t / 60.0) as u32;
    let s = ((t % 60.0) * 100.0).round() as i32;
    format!("{m}:{s:04}")
}
