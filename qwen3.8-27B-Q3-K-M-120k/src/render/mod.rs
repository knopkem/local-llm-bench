pub mod background;
pub mod framebuffer;
pub mod font;
pub mod hud;
pub mod road;
pub mod sprite;

pub use framebuffer::{W, H};
pub use font::draw_text;

/// 4x4 Bayer dither matrix (values /16).
pub const BAYER: [[f32; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];

/// Ordered-dither test: pick the "high" color when t exceeds the local threshold.
#[inline]
pub fn dither_pick(x: i32, y: i32, t: f32) -> bool {
    let bx = x.rem_euclid(4);
    let by = y.rem_euclid(4);
    BAYER[by as usize][bx as usize] / 16.0 < t
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
