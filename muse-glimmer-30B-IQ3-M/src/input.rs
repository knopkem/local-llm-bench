pub struct InputState {
    pub accelerate: bool,
    pub brake: bool,
    pub steer_left: bool,
    pub steer_right: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            accelerate: false,
            brake: false,
            steer_left: false,
            steer_right: false,
        }
    }
}
