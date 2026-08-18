pub struct Sprite {
    pub world_x: f32,
    pub world_y: f32,
    pub world_z: f32,
    pub sprite_type: SpriteType,
}

#[derive(Clone, Copy)]
pub enum SpriteType {
    TreeLarge,
    TreeSmall,
    BarrierRed,
    BarrierWhite,
    Bush,
    Rock,
}

pub struct CarSprite {
    pub color: [u8; 3],
    pub width: u16,
    pub height: u16,
}

impl SpriteType {
    pub fn width(&self) -> f32 {
        match self {
            SpriteType::TreeLarge => 60.0,
            SpriteType::TreeSmall => 40.0,
            SpriteType::BarrierRed | SpriteType::BarrierWhite => 80.0,
            SpriteType::Bush => 50.0,
            SpriteType::Rock => 35.0,
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            SpriteType::TreeLarge => 100.0,
            SpriteType::TreeSmall => 70.0,
            SpriteType::BarrierRed | SpriteType::BarrierWhite => 30.0,
            SpriteType::Bush => 40.0,
            SpriteType::Rock => 25.0,
        }
    }

    pub fn color(&self) -> [u8; 3] {
        match self {
            SpriteType::TreeLarge | SpriteType::TreeSmall => [0, 120, 0],
            SpriteType::BarrierRed => [200, 0, 0],
            SpriteType::BarrierWhite => [200, 200, 200],
            SpriteType::Bush => [0, 100, 0],
            SpriteType::Rock => [100, 100, 100],
        }
    }

    pub fn is_barrier(&self) -> bool {
        matches!(self, SpriteType::BarrierRed | SpriteType::BarrierWhite)
    }
}

pub fn generate_scenery(track_length: f32) -> Vec<Sprite> {
    let mut sprites = Vec::new();

    // Place trees along the track at regular intervals
    for i in 0..200 {
        let z = (i as f32 / 200.0) * track_length;
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let distance = 250.0 + (i as f32 % 100.0) as f32 * 2.0;

        sprites.push(Sprite {
            world_x: side * distance,
            world_y: 0.0,
            world_z: z,
            sprite_type: if i % 3 == 0 { SpriteType::TreeLarge } else { SpriteType::TreeSmall },
        });
    }

    // Place barriers along curves
    for i in 0..100 {
        let z = (i as f32 / 100.0) * track_length;
        let side = if i % 4 < 2 { -1.0 } else { 1.0 };
        let distance = 130.0;

        sprites.push(Sprite {
            world_x: side * distance,
            world_y: 0.0,
            world_z: z,
            sprite_type: if i % 2 == 0 { SpriteType::BarrierRed } else { SpriteType::BarrierWhite },
        });
    }

    // Place bushes and rocks sporadically
    for i in 0..50 {
        let z = (i as f32 / 50.0) * track_length + 100.0;
        let side = if i % 3 == 0 { -1.0 } else { 1.0 };
        let distance = 300.0 + (i as f32 % 50.0) as f32 * 3.0;

        sprites.push(Sprite {
            world_x: side * distance,
            world_y: 0.0,
            world_z: z,
            sprite_type: if i % 2 == 0 { SpriteType::Bush } else { SpriteType::Rock },
        });
    }

    sprites
}

pub fn player_car_color() -> [u8; 3] {
    [220, 30, 30] // Red
}

pub fn ai_car_colors() -> [[u8; 3]; 5] {
    [
        [30, 100, 220],   // Blue
        [30, 200, 80],    // Green
        [240, 200, 30],   // Yellow
        [220, 220, 220],  // White
        [240, 140, 20],   // Orange
    ]
}
