/// Length of one track segment in world units.
pub const SEG_LEN: f32 = 200.0;

#[derive(Clone, Copy)]
pub struct SegmentSprite {
    /// Lateral offset: -1..1 is on the road, beyond that is off-road.
    pub offset: f32,
    /// Sprite kind id (see assets::Kind).
    pub kind: u8,
}

#[derive(Clone)]
pub struct Segment {
    pub index: usize,
    pub curve: f32,
    /// Elevation at segment start / end.
    pub y0: f32,
    pub y1: f32,
    pub sprites: Vec<SegmentSprite>,
}

#[derive(Clone)]
pub struct Track {
    pub name: &'static str,
    pub segments: Vec<Segment>,
    /// Total track length in world units.
    pub length: f32,
    // Theme palette indices (assigned by assets).
    pub sky_top: u8,
    pub sky_horizon: u8,
    pub hill_far: u8,
    pub hill_near: u8,
}

impl Track {
    pub fn find_segment(&self, pos_wrapped: f32) -> usize {
        let p = pos_wrapped.rem_euclid(self.length);
        ((p / SEG_LEN) as usize).rem_euclid(self.segments.len())
    }

    /// Elevation at a wrapped track position.
    pub fn elevation_at(&self, pos_wrapped: f32) -> f32 {
        let p = pos_wrapped.rem_euclid(self.length);
        let i = (p / SEG_LEN) as usize;
        let seg = &self.segments[i.min(self.segments.len() - 1)];
        let t = (p - i as f32 * SEG_LEN) / SEG_LEN;
        seg.y0 + (seg.y1 - seg.y0) * t
    }

    /// Curve of the segment containing a wrapped position.
    pub fn curve_at(&self, pos_wrapped: f32) -> f32 {
        self.segments[self.find_segment(pos_wrapped)].curve
    }
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

pub struct TrackBuilder {
    segs: Vec<Segment>,
    y: f32,
}

impl TrackBuilder {
    pub fn new() -> Self {
        TrackBuilder {
            segs: Vec::new(),
            y: 0.0,
        }
    }

    /// Add `n` segments with the given curve (eased in/out over the first and
    /// last third) and a total elevation change of `dy`.
    pub fn road(&mut self, n: usize, curve: f32, dy: f32) {
        let per = dy / n as f32;
        for i in 0..n {
            let t = i as f32 / n as f32;
            let e = if t < 1.0 / 3.0 {
                smoothstep(t * 3.0)
            } else if t > 2.0 / 3.0 {
                smoothstep((1.0 - t) * 3.0)
            } else {
                1.0
            };
            let y0 = self.y;
            self.y += per;
            self.segs.push(Segment {
                index: 0, // assigned at build time
                curve: curve * e,
                y0,
                y1: self.y,
                sprites: Vec::new(),
            });
        }
    }

    pub fn build(mut self, name: &'static str) -> Track {
        // Close the elevation loop so laps wrap seamlessly.
        if self.y.abs() > 0.5 {
            let n = ((self.y.abs() / 25.0).ceil()).max(1.0) as usize;
            let per = -self.y / n as f32;
            for _ in 0..n {
                let y0 = self.y;
                self.y += per;
                self.segs.push(Segment {
                    index: 0,
                    curve: 0.0,
                    y0,
                    y1: self.y,
                    sprites: Vec::new(),
                });
            }
        }
        for (i, s) in self.segs.iter_mut().enumerate() {
            s.index = i;
        }
        let total_len = self.segs.len() as f32 * SEG_LEN;
        Track {
            name,
            segments: self.segs,
            length: total_len,
            sky_top: 0,
            sky_horizon: 0,
            hill_far: 0,
            hill_near: 0,
        }
    }
}

/// Deterministic LCG for scenery placement.
pub struct Lcg {
    state: u32,
}

impl Lcg {
    pub fn new(seed: u32) -> Self {
        Lcg { state: seed.max(1) }
    }
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        (self.state >> 8) as f32 / 16777216.0
    }
}

/// Scatter scenery sprites along the track (deterministic per seed).
pub fn scatter_scenery(track: &mut Track, seed: u32) {
    let mut rng = Lcg::new(seed);
    for seg in track.segments.iter_mut().skip(6) {
        if rng.next_f32() < 0.42 {
            let r = rng.next_f32();
            let kind = if r < 0.45 {
                1 // pine tree
            } else if r < 0.7 {
                2 // bush
            } else if r < 0.9 {
                3 // rock
            } else {
                4 // sign
            };
            let side = if rng.next_f32() < 0.5 { -1.0 } else { 1.0 };
            seg.sprites.push(SegmentSprite {
                offset: side * (1.3 + rng.next_f32() * 2.0),
                kind,
            });
        }
    }
    // Flag poles flanking the start line.
    if track.segments.len() > 2 {
        track.segments[0].sprites.push(SegmentSprite { offset: -1.7, kind: 5 });
        track.segments[0].sprites.push(SegmentSprite { offset: 1.7, kind: 5 });
        track.segments[1].sprites.push(SegmentSprite { offset: -1.7, kind: 5 });
        track.segments[1].sprites.push(SegmentSprite { offset: 1.7, kind: 5 });
    }
}

/// Track 1: flat coastal circuit with flowing S-curves.
pub fn coastal_circuit() -> Track {
    let mut b = TrackBuilder::new();
    b.road(40, 0.0, 0.0); // start straight
    b.road(28, 2.2, 10.0);
    b.road(16, -1.5, 0.0);
    b.road(30, 0.0, 0.0); // mid straight
    b.road(24, -2.8, -10.0);
    b.road(20, 1.8, 0.0);
    b.road(36, 0.0, 0.0); // back straight
    b.road(26, 3.4, 15.0); // fast right kink uphill
    b.road(18, -2.0, 0.0);
    b.road(22, 0.0, 0.0);
    b.road(30, -3.0, -15.0);
    b.road(24, 2.4, 0.0);
    b.road(34, 0.0, 0.0); // final straight
    let mut t = b.build("COASTAL CIRCUIT");
    scatter_scenery(&mut t, 1234);
    t
}

/// Track 2: mountain pass with elevation and hairpins.
pub fn mountain_pass() -> Track {
    let mut b = TrackBuilder::new();
    b.road(30, 0.0, 0.0); // start straight
    b.road(24, 2.6, 40.0);
    b.road(14, -4.5, 20.0); // hairpin left uphill
    b.road(20, 0.0, 30.0);
    b.road(22, -2.2, -20.0);
    b.road(16, 4.8, 10.0); // hairpin right
    b.road(26, 0.0, 40.0); // climb straight
    b.road(20, -3.2, 0.0);
    b.road(18, 2.0, -30.0);
    b.road(24, 0.0, -50.0); // descent
    b.road(26, 3.8, -20.0);
    b.road(16, -4.2, 0.0); // hairpin left downhill
    b.road(28, 0.0, 20.0);
    b.road(22, -2.6, 30.0);
    b.road(30, 0.0, 10.0); // final climb to line
    let mut t = b.build("MOUNTAIN PASS");
    scatter_scenery(&mut t, 98765);
    t
}
