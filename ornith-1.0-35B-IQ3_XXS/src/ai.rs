use crate::track::Track;
use crate::sprites::ai_car_colors;

pub struct AIState {
    pub z: f32,
    pub speed: f32,
    pub skill: f32,
    pub color: [u8; 3],
    pub id: usize,
}

impl AIState {
    pub fn new(id: usize) -> Self {
        let base_skill = 0.85 + (id as f32 * 0.07).min(0.15); // 0.85 to ~1.0
        let color = ai_car_colors()[id % 5];

        AIState {
            z: 0.0,
            speed: 0.0,
            skill: base_skill,
            color,
            id,
        }
    }

    pub fn update(&mut self, track: &Track, player_z: f32, dt: f32) {
        const BASE_SPEED: f32 = 280.0;
        const ACCEL: f32 = 350.0;
        const MAX_SPEED_MULT: f32 = 1.2;

        // Find upcoming curve severity
        let look_ahead_segs = 20;
        let seg_idx = (self.z / 1.0).floor() as usize % track.segments.len();
        let mut total_curve = 0.0f32;

        for i in 0..look_ahead_segs {
            let idx = (seg_idx + i) % track.segments.len();
            total_curve += track.segments[idx].curve.abs();
        }

        // Slow down for curves - more curve = slower
        let curve_factor = if total_curve > 0.5 {
            (1.0 - total_curve * 0.3).max(0.4)
        } else {
            1.0
        };

        // Target speed based on skill and curve
        let target_speed = BASE_SPEED * self.skill * curve_factor * MAX_SPEED_MULT;

        // Smooth acceleration/deceleration
        if self.speed < target_speed {
            self.speed += ACCEL * self.skill * dt;
        } else {
            self.speed -= ACCEL * 0.5 * dt;
        }

        self.speed = self.speed.max(100.0).min(target_speed * 1.1);

        // Move forward
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

        let x = curr.world_x + (next.world_x - curr.world_x) * frac;
        let y = curr.world_y + (next.world_y - curr.world_y) * frac;

        (x, y)
    }

    pub fn laps_ahead_of(&self, player_z: f32, track_length: f32) -> i32 {
        if self.z > player_z {
            ((self.z - player_z) / track_length).floor() as i32
        } else {
            -((player_z - self.z) / track_length).floor() as i32
        }
    }
}
