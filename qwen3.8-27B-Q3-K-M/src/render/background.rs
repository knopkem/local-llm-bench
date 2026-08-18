use super::framebuffer::Renderer;
use super::{dither_pick, W};
use crate::render::sprite::Sprite;

/// Precomputed sky gradient band palette indices (N+1 entries, top -> horizon).
pub struct SkyTheme {
    pub bands: Vec<u8>,
}

impl SkyTheme {
    /// Build the banded gradient between two palette colors.
    pub fn build(pal: &mut crate::render::framebuffer::Palette, top_rgb: [u8; 3], hor_rgb: [u8; 3], n_bands: usize) -> Self {
        let mut bands = Vec::with_capacity(n_bands + 1);
        for i in 0..=n_bands {
            let t = i as f32 / n_bands as f32; // 0 = top of screen, 1 = horizon
            let c: [u8; 3] = [
                (top_rgb[0] as f32 + (hor_rgb[0] as f32 - top_rgb[0] as f32) * t).round() as u8,
                (top_rgb[1] as f32 + (hor_rgb[1] as f32 - top_rgb[1] as f32) * t).round() as u8,
                (top_rgb[2] as f32 + (hor_rgb[2] as f32 - top_rgb[2] as f32) * t).round() as u8,
            ];
            bands.push(pal.add(c[0], c[1], c[2]));
        }
        SkyTheme { bands }
    }
}

/// Dithered banded sky gradient from row 0 down to `horizon_y` (exclusive).
pub fn render_sky(r: &mut Renderer, horizon_y: i32, sky: &SkyTheme) {
    let n = sky.bands.len() - 1;
    if n <= 0 || horizon_y <= 0 {
        return;
    }
    let hy = horizon_y.min(r.h as i32);
    for y in 0..hy {
        // t: 0 at top of screen -> 1 at horizon
        let t = (y as f32 / hy.max(1) as f32).clamp(0.0, 1.0);
        let bf = t * n as f32;
        let b = bf.floor() as usize;
        let frac = bf - b as f32;
        let lo = sky.bands[b];
        let hi = sky.bands[(b + 1).min(n)];
        for x in 0..W {
            r.set(x as i32, y, if dither_pick(x as i32, y, frac) { hi } else { lo });
        }
    }
}

/// Draw a hill silhouette strip just above `horizon_y`, wrapped horizontally
/// by `offset` (parallax scroll driven by the curves).
pub fn render_hills(r: &mut Renderer, img: &Sprite, offset: i32, horizon_y: i32) {
    let h = img.h;
    for py in 0..h {
        let row_y = horizon_y - h as i32 + py as i32;
        if row_y < 0 || row_y >= horizon_y {
            continue;
        }
        for x in 0..W {
            let sx = ((x as i32 + offset).rem_euclid(W as i32)) as usize;
            if let Some(c) = img.px[py * W + sx] {
                r.set(x as i32, row_y, c);
            }
        }
    }
}
