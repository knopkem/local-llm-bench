pub mod ai;
pub mod car;
pub mod race;
pub mod track;

/// Fixed simulation timestep (60 Hz).
pub const STEP: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, Debug)]
pub struct InputState {
    pub throttle: bool,
    pub brake: bool,
    pub left: bool,
    pub right: bool,
}

impl InputState {
    pub fn none() -> Self {
        InputState {
            throttle: false,
            brake: false,
            left: false,
            right: false,
        }
    }
}
