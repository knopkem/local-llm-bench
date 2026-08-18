use super::track::Track;
use super::InputState;

pub const TOP_SPEED: f32 = 2000.0;
const ACCEL: f32 = 950.0;
const BRAKE: f32 = 1700.0;
const COAST: f32 = 340.0;
const REVERSE_ACCEL: f32 = 300.0;
const MAX_REVERSE: f32 = -260.0;
const OFFROAD_LIMIT: f32 = 480.0;
const OFFROAD_DECEL: f32 = 2600.0;
const STEER_RATE: f32 = 1.9;
const CENTRIFUGAL: f32 = 0.55;

pub struct PlayerCar {
    /// Unbounded distance travelled along the track (world units).
    pub pos: f32,
    /// Lateral position in road half-widths (-1..1 on the road).
    pub x: f32,
    pub speed: f32,
}

impl PlayerCar {
    pub fn new(grid_pos: f32) -> Self {
        PlayerCar { pos: grid_pos, x: 0.0, speed: 0.0 }
    }

    pub fn pos_wrapped(&self, length: f32) -> f32 {
        self.pos.rem_euclid(length)
    }

    /// Display speed in "mph".
    pub fn mph(&self) -> i32 {
        (self.speed.abs() * 0.075).round() as i32
    }

    pub fn update(&mut self, dt: f32, input: &InputState, track: &Track) {
        let on_road = self.x.abs() <= 1.0;

        // Longitudinal.
        if input.throttle {
            self.speed += ACCEL * dt;
        } else if input.brake {
            if self.speed > 40.0 {
                self.speed -= BRAKE * dt;
            } else {
                self.speed = (self.speed - REVERSE_ACCEL * dt).max(MAX_REVERSE);
            }
        } else if self.speed > 0.0 {
            self.speed = (self.speed - COAST * dt).max(0.0);
        }

        // Off-road drag.
        if !on_road && self.speed > OFFROAD_LIMIT {
            self.speed = (self.speed - OFFROAD_DECEL * dt).max(OFFROAD_LIMIT);
        }
        self.speed = self.speed.clamp(MAX_REVERSE, TOP_SPEED);

        // Lateral: steering scaled by speed.
        let sr = (self.speed.abs() / TOP_SPEED).min(1.0);
        let steer = (input.right as u8) as f32 - (input.left as u8) as f32;
        self.x += steer * STEER_RATE * sr * dt;

        // Centrifugal force pushes outward on curves.
        let curve = track.curve_at(self.pos_wrapped(track.length));
        self.x -= curve * CENTRIFUGAL * sr * sr * dt;

        self.x = self.x.clamp(-2.6, 2.6);

        self.pos += self.speed * dt;
    }
}

/// Grid slot position (just before the start line).
pub fn grid_pos(length: f32, slot: usize) -> f32 {
    length - 700.0 - slot as f32 * 260.0
}

impl PlayerCar {
    /// Simple AI driving used for attract mode / demos.
    pub fn autopilot(&mut self, dt: f32, track: &Track) {
        let v_target = super::ai::cornering_speed(track, self.pos.rem_euclid(track.length), TOP_SPEED, 0.7);

        if self.speed < v_target {
            self.speed += ACCEL * dt;
        } else {
            self.speed -= BRAKE * dt;
        }
        self.speed = self.speed.clamp(0.0, TOP_SPEED);

        // Steer toward the center line.
        let max_step = 1.2 * dt * (self.speed / TOP_SPEED).min(1.0);
        self.x += (-self.x).clamp(-max_step, max_step);

        self.pos += self.speed * dt;
    }
}

