use super::car::TOP_SPEED;
use super::track::Track;

const AI_ACCEL: f32 = 750.0;
const AI_BRAKE: f32 = 1400.0;
const LOOKAHEAD_SEGS: usize = 40;

/// A snapshot of any car (player or AI) for separation checks.
#[derive(Clone, Copy)]
pub struct CarSnap {
    pub pos: f32,
    pub x: f32,
    pub speed: f32,
}

/// Target speed for a driver of the given skill approaching the curves ahead
/// of `pos_wrapped`.
pub fn cornering_speed(track: &Track, pos_wrapped: f32, top_speed: f32, skill: f32) -> f32 {
    let base = track.find_segment(pos_wrapped);
    let mut cs = 0.0f32;
    let mut wsum = 0.0f32;
    for i in 0..LOOKAHEAD_SEGS {
        let si = (base + i) % track.segments.len();
        let w = 1.0 - i as f32 / (LOOKAHEAD_SEGS as f32 * 1.4);
        cs += track.segments[si].curve.abs() * w;
        wsum += w;
    }
    cs /= wsum.max(0.001);

    let brake_factor = 1.0 - skill * 0.55;
    top_speed * (1.0 - (cs / 4.2).min(1.0) * 0.85 * brake_factor)
}

pub struct AiCar {
    pub id: usize,
    pub name: &'static str,
    /// Index into assets::ai_cars.
    pub car_sprite: u8,
    pub pos: f32,
    pub x: f32,
    pub speed: f32,
    pub top_speed: f32,
    /// Cornering skill 0..1 (higher carries more speed through curves).
    pub skill: f32,
    /// Preferred lateral line.
    pub base_offset: f32,
    /// Position on the grid at race start (for lap counting / ranking).
    pub grid_pos: f32,
    pub finished: bool,
    pub finish_time: f64,
}

impl AiCar {
    pub fn laps_done(&self, length: f32) -> u32 {
        (((self.pos - self.grid_pos) / length).floor().max(0.0)) as u32
    }

    /// Total distance from the grid (for ranking).
    pub fn travelled(&self) -> f32 {
        (self.pos - self.grid_pos).max(0.0)
    }

    pub fn update(&mut self, dt: f32, track: &Track, others: &[CarSnap]) {
        if self.finished {
            return;
        }

        let v_corner = cornering_speed(track, self.pos.rem_euclid(track.length), self.top_speed, self.skill);

        // Separation: slow for a car just ahead in our lane.
        let mut v_target = v_corner;
        for o in others {
            if (o.pos - self.pos) > 10.0 && (o.pos - self.pos) < 480.0 && (o.x - self.x).abs() < 0.62 {
                v_target = v_target.min(o.speed * 0.93);
            }
        }

        if self.speed < v_target {
            self.speed += AI_ACCEL * dt;
        } else {
            self.speed -= AI_BRAKE * dt;
        }
        self.speed = self.speed.clamp(0.0, self.top_speed);

        // Lateral: steer toward racing line with a gentle wobble.
        let target_x = (self.base_offset + 0.12 * ((self.pos / 2500.0 + self.id as f32 * 2.1).sin()))
            .clamp(-0.8, 0.8);
        let diff = target_x - self.x;
        let max_step = 0.9 * dt * (self.speed / TOP_SPEED).min(1.0);
        self.x += diff.clamp(-max_step, max_step);

        self.pos += self.speed * dt;
    }
}

