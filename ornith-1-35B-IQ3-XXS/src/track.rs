pub struct TrackPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct Segment {
    pub world_x: f32,
    pub world_y: f32,
    pub world_z: f32,
    pub screen_x: f32,
    pub screen_y: f32,
    pub screen_w: f32,
    pub curve: f32,
    pub hill: f32,
}

pub struct Track {
    pub segments: Vec<Segment>,
    pub total_length: f32,
}

pub const ROAD_WIDTH: f32 = 200.0;
const SEGMENT_LENGTH: f32 = 1.0;

impl Track {
    pub fn new() -> Self {
        let points = vec![
            TrackPoint { x: 0.0, y: 0.0, z: 0.0 },
            TrackPoint { x: 0.0, y: 0.0, z: 800.0 },
            TrackPoint { x: 250.0, y: 0.0, z: 1300.0 },
            TrackPoint { x: 400.0, y: 0.0, z: 1800.0 },
            TrackPoint { x: 250.0, y: 120.0, z: 2300.0 },
            TrackPoint { x: 50.0, y: 100.0, z: 2800.0 },
            TrackPoint { x: -150.0, y: 60.0, z: 3300.0 },
            TrackPoint { x: -350.0, y: 30.0, z: 3800.0 },
            TrackPoint { x: -200.0, y: 0.0, z: 4300.0 },
            TrackPoint { x: 0.0, y: -60.0, z: 4800.0 },
            TrackPoint { x: 150.0, y: -90.0, z: 5300.0 },
            TrackPoint { x: 250.0, y: -70.0, z: 5800.0 },
            TrackPoint { x: 150.0, y: -40.0, z: 6300.0 },
            TrackPoint { x: 0.0, y: 0.0, z: 6800.0 },
        ];

        let segments = Self::generate_segments(&points);
        let total_length = segments.last().map(|s| s.world_z).unwrap_or(0.0);
        Track { segments, total_length }
    }

    fn catmull_rom(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), t: f32) -> (f32, f32) {
        let t2 = t * t;
        let t3 = t2 * t;

        let x = (2.0 * p1.0)
            + (-p0.0 + p2.0) * t
            + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2
            + (-3.0 * p0.0 + 9.0 * p1.0 - 9.0 * p2.0 + p3.0) * t3;
        let x = x * 0.5;

        let y = (2.0 * p1.1)
            + (-p0.1 + p2.1) * t
            + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2
            + (-3.0 * p0.1 + 9.0 * p1.1 - 9.0 * p2.1 + p3.1) * t3;
        let y = y * 0.5;

        (x, y)
    }

    fn generate_segments(points: &[TrackPoint]) -> Vec<Segment> {
        let n = points.len();
        if n < 4 {
            return vec![];
        }

        struct SplineSeg {
            p0: (f32, f32),
            p1: (f32, f32),
            p2: (f32, f32),
            p3: (f32, f32),
            z_start: f32,
            z_end: f32,
        }

        let mut spline_segs = Vec::new();
        for i in 0..(n - 1) as isize {
            let pi = i as usize;
            let p0_idx = ((i - 1 + n as isize) % n as isize) as usize;
            let p3_idx = (pi + 2) % n;

            let p0 = (points[p0_idx].x, points[p0_idx].y);
            let p1 = (points[pi].x, points[pi].y);
            let p2 = (points[(pi + 1) % n].x, points[(pi + 1) % n].y);
            let p3 = (points[p3_idx].x, points[p3_idx].y);

            spline_segs.push(SplineSeg {
                p0,
                p1,
                p2,
                p3,
                z_start: points[pi].z,
                z_end: points[(pi + 1) % n].z,
            });
        }

        let mut segments = Vec::new();
        let mut prev_x = spline_segs[0].p1.0;
        let mut prev_y = spline_segs[0].p1.1;
        let mut prev_z = spline_segs[0].z_start;

        for seg in &spline_segs {
            let steps = ((seg.z_end - seg.z_start) / SEGMENT_LENGTH).ceil() as usize;
            if steps == 0 {
                continue;
            }

            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let (x, y) = Self::catmull_rom(seg.p0, seg.p1, seg.p2, seg.p3, t);
                let z = seg.z_start + (seg.z_end - seg.z_start) * t;

                if segments.len() > 0 {
                    let dx = x - prev_x;
                    let dy = y - prev_y;
                    let dz = z - prev_z;
                    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                    let curve = dx / dist;
                    let hill = dy / dist;

                    segments.push(Segment {
                        world_x: x,
                        world_y: y,
                        world_z: z,
                        screen_x: 0.0,
                        screen_y: 0.0,
                        screen_w: 0.0,
                        curve,
                        hill,
                    });
                }

                prev_x = x;
                prev_y = y;
                prev_z = z;
            }
        }

        segments
    }

    pub fn get_segment_at(&self, z: f32) -> Option<&Segment> {
        let idx = (z / SEGMENT_LENGTH).floor() as usize;
        if idx < self.segments.len() {
            Some(&self.segments[idx])
        } else {
            None
        }
    }

    pub fn lap_length(&self) -> f32 {
        self.total_length
    }
}
