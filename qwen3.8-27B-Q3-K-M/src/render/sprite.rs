use std::collections::HashMap;

use super::framebuffer::Renderer;

/// A pre-rendered sprite: palette-indexed pixels, None = transparent.
#[derive(Clone)]
pub struct Sprite {
    pub w: usize,
    pub h: usize,
    pub px: Vec<Option<u8>>,
}

impl Sprite {
    /// Build a sprite from an ASCII char map. `colors` maps each character to a
    /// palette index; '.' (or any unmapped char) is transparent.
    pub fn from_map(map: &[&str], colors: &HashMap<char, u8>) -> Self {
        let h = map.len();
        let w = map[0].chars().count();
        let mut px = Vec::with_capacity(w * h);
        for (i, row) in map.iter().enumerate() {
            // Clamp ragged rows to the first-row width so indexing stays aligned.
            for ch in row.chars().take(w) {
                if let Some(&c) = colors.get(&ch) {
                    px.push(Some(c));
                } else {
                    px.push(None);
                }
            }
            while px.len() < (i + 1) * w {
                px.push(None);
            }
        }
        Sprite { w, h, px }
    }

    /// Remap one source char to a different palette index (sprite variants).
    pub fn remap(&self, from: u8, to: u8) -> Self {
        let px = self.px.iter().map(|p| match p {
            Some(c) if *c == from => Some(to),
            other => *other,
        }).collect();
        Sprite { w: self.w, h: self.h, px }
    }

    /// Generate a sprite procedurally via a per-pixel callback.
    pub fn generate(w: usize, h: usize, f: impl Fn(usize, usize) -> Option<u8>) -> Self {
        let mut px = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                px.push(f(x, y));
            }
        }
        Sprite { w, h, px }
    }
}

/// Blit a sprite at (dx, dy) with nearest-neighbor scale.
/// `clip_y`: pixels on rows >= clip_y are skipped (hill clipping). Pass -1 for none.
pub fn blit_scaled(r: &mut Renderer, s: &Sprite, dx: i32, dy: i32, scale: f32, clip_y: i32) {
    if scale <= 0.0 {
        return;
    }
    let dw = (s.w as f32 / scale).ceil() as i32;
    let dh = (s.h as f32 / scale).ceil() as i32;
    for py in 0..dh {
        let row_y = dy + py;
        if clip_y >= 0 && row_y >= clip_y {
            continue;
        }
        let sy = (py as f32 * scale) as usize;
        if sy >= s.h {
            continue;
        }
        for px_ in 0..dw {
            let sx = (px_ as f32 * scale) as usize;
            if sx >= s.w {
                continue;
            }
            if let Some(c) = s.px[sy * s.w + sx] {
                r.set(dx + px_, row_y, c);
            }
        }
    }
}
