use std::collections::HashMap;

pub const W: usize = 640;
pub const H: usize = 256;

/// 256-color palette with dedup on insert (period-correct constraint).
#[derive(Clone)]
pub struct Palette {
    colors: Vec<[u8; 3]>,
    map: HashMap<[u8; 3], u8>,
}

impl Palette {
    pub fn new() -> Self {
        Palette {
            colors: Vec::new(),
            map: HashMap::new(),
        }
    }

    /// Register a color, returning its palette index (deduped).
    pub fn add(&mut self, r: u8, g: u8, b: u8) -> u8 {
        let c = [r, g, b];
        if let Some(&i) = self.map.get(&c) {
            return i;
        }
        debug_assert!(self.colors.len() < 256, "palette overflow");
        let i = self.colors.len() as u8;
        self.colors.push(c);
        self.map.insert(c, i);
        i
    }

    pub fn get(&self, i: usize) -> [u8; 3] {
        self.colors[i.min(self.colors.len() - 1)]
    }

    /// Overwrite a palette entry (used for color cycling).
    pub fn set(&mut self, i: usize, c: [u8; 3]) {
        if i < self.colors.len() {
            self.map.remove(&self.colors[i]);
            self.colors[i] = c;
            self.map.insert(c, i as u8);
        }
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }
}

/// Software framebuffer: palette-indexed pixels + the palette itself.
pub struct Renderer {
    pub w: usize,
    pub h: usize,
    pub pal: Palette,
    idx: Vec<u8>,
}

impl Renderer {
    pub fn new(w: usize, h: usize) -> Self {
        Renderer {
            w,
            h,
            pal: Palette::new(),
            idx: vec![0; w * h],
        }
    }

    /// Create a renderer with a pre-built palette (from assets).
    pub fn with_palette(w: usize, h: usize, pal: Palette) -> Self {
        Renderer {
            w,
            h,
            pal,
            idx: vec![0; w * h],
        }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: u8) {
        if (x as usize) < self.w && (y as usize) < self.h {
            self.idx[y as usize * self.w + x as usize] = c;
        }
    }

    pub fn clear(&mut self, c: u8) {
        self.idx.fill(c);
    }

    /// Horizontal line at row y from x0..=x1 (clamped to screen).
    #[inline]
    pub fn hline(&mut self, y: i32, x0: i32, x1: i32, c: u8) {
        if y < 0 || (y as usize) >= self.h {
            return;
        }
        let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        let a = lo.max(0);
        let b = hi.min((self.w - 1) as i32);
        if a > b {
            return;
        }
        let row = y as usize * self.w;
        for x in a..=b {
            self.idx[row + x as usize] = c;
        }
    }

    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u8) {
        for y in y0.max(0)..=(y1.min((self.h - 1) as i32)) {
            self.hline(y, x0, x1, c);
        }
    }

    /// Convert indexed framebuffer to an RGBA byte buffer (for upload / PNG export).
    pub fn rgba(&self) -> Vec<u8> {
        let mut out = vec![0u8; self.w * self.h * 4];
        for i in 0..(self.w * self.h) {
            let c = self.pal.get(self.idx[i] as usize);
            out[i * 4] = c[0];
            out[i * 4 + 1] = c[1];
            out[i * 4 + 2] = c[2];
            out[i * 4 + 3] = 255;
        }
        out
    }
}
