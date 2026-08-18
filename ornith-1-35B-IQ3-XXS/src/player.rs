use crate::track::Track;

pub struct PlayerState {
    pub z: f32,
    pub x: f32,
    pub speed: f32,
    pub steer: f32,
    pub handbrake: bool,
    pub bounce: f32,
    pub bounce_vel: f32,
}

impl PlayerState {
    pub fn new() -> Self {
        PlayerState {
            z: 0.0,
            x: 0.0,
            speed: 0.0,
            steer: 0.0,
            handbrake: false,
            bounce: 0.0,
            bounce_vel: 0.0,
        }
    }

    pub fn update(&mut self, track: &Track, dt: f32) {
        const MAX_SPEED: f32 = 350.0;
        const ACCEL: f32 = 400.0;
        const BRAKE: f32 = 600.0;
        const DECEL: f32 = 200.0;
        const STEER_SPEED: f32 = 3.0;
        const CENTRIFUGAL: f32 = 0.4;
        const OFF_ROAD_SLOW: f32 = 0.5;
        const ROAD_WIDTH_HALF: f32 = 110.0;

        // Acceleration / braking
        if self.steer != 0.0 {
            // Steering while at speed - slight deceleration for realism
        }

        if self.speed < MAX_SPEED {
            self.speed += ACCEL * dt;
        } else {
            self.speed -= DECEL * dt * 0.5;
        }

        // Apply centrifugal force from curves
        let seg_idx = (self.z / 1.0).floor() as usize;
        if seg_idx < track.segments.len() {
            let seg = &track.segments[seg_idx];
            self.x -= seg.curve * self.speed * CENTRIFUGAL * dt;
        }

        // Steering input
        let steer_input = self.steer;
        self.x += steer_input * STEER_SPEED * (self.speed / MAX_SPEED + 0.3) * dt * 100.0;

        // Off-road slowdown
        if self.x.abs() > ROAD_WIDTH_HALF {
            let off_ratio = (self.x.abs() - ROAD_WIDTH_HALF) / 200.0;
            self.speed *= (1.0 - DECEL * off_ratio * dt).max(0.1);
        }

        // Handbrake
        if self.handbrake {
            self.speed -= BRAKE * 1.5 * dt;
            // Tighter steering when handbraking
            self.x += steer_input * STEER_SPEED * 2.0 * (self.speed / MAX_SPEED + 0.3) * dt * 100.0;
        }

        // Speed clamping
        self.speed = self.speed.max(0.0).min(MAX_SPEED);

        // Bounce from hills
        if seg_idx < track.segments.len() {
            let hill_force = track.segments[seg_idx].hill * self.speed * 0.5;
            self.bounce_vel += hill_force * dt;
        }
        self.bounce_vel *= 0.92; // Damping
        self.bounce += self.bounce_vel * dt;

        // Move forward along track
        self.z += self.speed * dt;

        // Lap wrapping
        if self.z >= track.total_length {
            self.z -= track.total_length;
        } else if self.z < 0.0 {
            self.z += track.total_length;
        }
    }

    pub fn position_on_track(&self, track: &Track) -> (f32, f32) {
        let seg_idx = (self.z / 1.0).floor() as usize % track.segments.len();
        let frac = (self.z / 1.0 - seg_idx as f32).max(0.0).min(1.0);

        let curr = &track.segments[seg_idx];
        let next_idx = (seg_idx + 1) % track.segments.len();
        let next = &track.segments[next_idx];

        let x = curr.world_x + (next.world_x - curr.world_x) * frac + self.x;
        let y = curr.world_y + (next.world_y - curr.world_y) * frac + self.bounce;

        (x, y)
    }
}
