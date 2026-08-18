use crate::track::Track;
use crate::car::Car;
use crate::input::InputState;

pub struct GameState {
    pub track: Track,
    pub player: Car,
    pub opponents: Vec<Car>,
    pub lap: u32,
    pub total_laps: u32,
}

impl GameState {
    pub fn new() -> Self {
        let track = Track::new();
        let player = Car::new_player();
        let mut opponents = Vec::new();
        for i in 0..5 {
            opponents.push(Car::new_ai(i));
        }
        Self {
            track,
            player,
            opponents,
            lap: 1,
            total_laps: 3,
        }
    }
    pub fn update(&mut self, input: &InputState) {
        self.player.update_with_input(Some(input));
        for opponent in &mut self.opponents {
            opponent.update_with_input(None);
        }
    }
}
