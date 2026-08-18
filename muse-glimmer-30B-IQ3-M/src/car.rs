use crate::input::InputState;

pub struct Car {
    pub z: f32,
    pub lane: i32,
    pub speed: f32,
    pub angle: f32,
    pub is_player: bool,
    pub id: usize,
}

impl Car {
    pub fn new_player() -> Self {
        Self {
            z: 0.0,
            lane: 0,
            speed: 0.0,
            angle: 0.0,
            is_player: true,
            id: 0,
        }
    }
    
    pub fn new_ai(id: usize) -> Self {
        Self {
            z: (id as f32 * 200.0),
            lane: (id % 3) as i32 - 1,
            speed: 80.0 + (id as f32 * 5.0),
            angle: 0.0,
            is_player: false,
            id,
        }
    }
    
    pub fn update(&mut self) {
        self.z += self.speed * 0.1;
        if self.z > 2000.0 {
            self.z = -500.0;
        }
    }
    
    pub fn update_player(&mut self, input: &InputState) {
        if input.accelerate {
            self.speed += 2.0;
            if self.speed > 200.0 { self.speed = 200.0; }
        }
        if input.brake {
            self.speed -= 3.0;
            if self.speed < 0.0 { self.speed = 0.0; }
        }
        if input.steer_left && self.lane > -1 {
            self.lane -= 1;
        }
        if input.steer_right && self.lane < 1 {
            self.lane += 1;
        }
        self.z += self.speed * 0.1;
    }
    
    pub fn update_with_input(&mut self, input: Option<&InputState>) {
        if self.is_player {
            if let Some(inp) = input {
                self.update_player(inp);
            }
        } else {
            self.update();
        }
    }
}
