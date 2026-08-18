pub struct TrackPoint {
    pub x: f32,
    pub y: f32,
    pub width: f32,
}

pub struct Track {
    pub points: Vec<TrackPoint>,
}

impl Track {
    pub fn new() -> Self {
        let mut points = Vec::new();
        // Simple oval circuit
        for i in 0..100 {
            let angle = i as f32 * 0.0628;
            let radius = 200.0 + (i as f32 % 20.0);
            points.push(TrackPoint {
                x: radius * angle.cos(),
                y: radius * angle.sin(),
                width: 40.0,
            });
        }
        Self { points }
    }
    
    pub fn get_point(&self, index: usize) -> &TrackPoint {
        &self.points[index % self.points.len()]
    }
}
