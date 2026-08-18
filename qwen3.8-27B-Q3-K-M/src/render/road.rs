use crate::game::track::{Track, SEG_LEN};
use super::framebuffer::Renderer;
use super::{lerp, H, W};

/// Perspective depth (fov ~100 deg).
pub const CAM_DEPTH: f32 = 0.84;
/// Road half-width in world units.
pub const ROAD_WIDTH: f32 = 700.0;
/// Camera height above the road surface.
pub const CAM_HEIGHT: f32 = 1000.0;
/// Distance from camera to the player car plane.
pub const PLAYER_Z: f32 = CAM_HEIGHT * CAM_DEPTH;
/// How many segments ahead are projected.
pub const DRAW_DIST: usize = 120;

/// Horizon line (projection reference), slightly above center for road view.
pub const HORIZON_Y: i32 = ((H as f32) * 0.40).round() as i32;

#[derive(Clone, Copy)]
pub struct RoadColors {
    pub grass_a: u8,
    pub grass_b: u8,
    pub road_a: u8,
    pub road_b: u8,
    pub rumble_a: u8,
    pub rumble_b: u8,
    pub lane: u8,
    pub checker_a: u8,
    pub checker_b: u8,
}

/// Projection data for one drawn segment (near-to-far order).
pub struct SegProj {
    pub index: usize,
    /// Topmost row already covered by nearer segments (sprite clip line).
    pub clip: i32,
    /// Accumulated curve x-offset at the near / far end of this segment.
    pub xoff1: f32,
    pub xoff2: f32,
}

pub struct RoadView {
    pub horizon_y: i32,
    pub cam_y: f32,
    pub segs: Vec<SegProj>,
}

#[inline]
fn project(world_x: f32, world_y: f32, rel_z: f32, cam_x: f32, cam_y: f32) -> (f32, i32, i32) {
    let scale = CAM_DEPTH / rel_z;
    let sx = W as f32 / 2.0 + (W as f32 / 2.0) * scale * (world_x - cam_x);
    let sy = HORIZON_Y as f32 - ((H as f32) / 2.0) * scale * (world_y - cam_y);
    (scale, sx.round() as i32, sy.round() as i32)
}

/// Render the road surface for this frame; returns projection data used by
/// sprite rendering and the horizon position for sky/hills.
pub fn render_road(
    r: &mut Renderer,
    track: &Track,
    pos_wrapped: f32,
    player_x: f32,
    colors: &RoadColors,
) -> RoadView {
    let len = track.segments.len();
    let length = track.length;
    let base = track.find_segment(pos_wrapped);
    let pw = pos_wrapped.rem_euclid(length);
    let base_pct = (pw - base as f32 * SEG_LEN) / SEG_LEN;

    // Camera sits PLAYER_Z behind the car reference point.
    let cam_y = track.elevation_at(pw + PLAYER_Z) + CAM_HEIGHT;

    let mut maxy = H as i32;
    let mut x_acc = 0.0f32;
    let mut dx_acc = -(track.segments[base].curve * base_pct);
    let mut segs: Vec<SegProj> = Vec::with_capacity(DRAW_DIST);

    for n in 0..DRAW_DIST {
        let si = (base + n) % len;
        let seg = &track.segments[si];
        let looped = si < base;

        let z1 = si as f32 * SEG_LEN + if looped { length } else { 0.0 };
        let rel_z1 = z1 - pw;
        let rel_z2 = rel_z1 + SEG_LEN;

        let xoff1 = x_acc;
        let cam_x1 = player_x * ROAD_WIDTH - xoff1;
        let cam_x2 = player_x * ROAD_WIDTH - xoff1 - dx_acc;
        let xoff2 = xoff1 + dx_acc;

        let (_s1, sx1, sy1) = project(0.0, seg.y0, rel_z1, cam_x1, cam_y);
        let (_s2, sx2, sy2) = project(0.0, seg.y1, rel_z2, cam_x2, cam_y);

        x_acc += dx_acc;
        dx_acc += seg.curve;

        if rel_z1 <= CAM_DEPTH || sy2 >= sy1 || sy1 >= maxy {
            continue; // behind camera / backface / hidden by nearer crest
        }

        let clip = maxy;
        let y_top = sy2.max(0);
        let y_bot = sy1.min((H - 1) as i32);
        if y_top <= y_bot {
            draw_strip(r, colors, si, sx1, sy1, sx2, sy2, rel_z1, rel_z2, y_top, y_bot);
        }
        maxy = sy1;

        segs.push(SegProj { index: si, clip, xoff1, xoff2 });
    }

    RoadView { horizon_y: HORIZON_Y, cam_y, segs }
}

fn draw_strip(
    r: &mut Renderer,
    colors: &RoadColors,
    si: usize,
    sx1: i32, sy1: i32, // near end (p1)
    sx2: i32, sy2: i32, // far end (p2)
    rel_z1: f32,
    rel_z2: f32,
    y_top: i32,
    y_bot: i32,
) {
    let hw1 = (W as f32 / 2.0) * (CAM_DEPTH / rel_z1) * ROAD_WIDTH;
    let hw2 = (W as f32 / 2.0) * (CAM_DEPTH / rel_z2) * ROAD_WIDTH;

    let light = (si / 2) % 2 == 0;
    let grass_c = if light { colors.grass_a } else { colors.grass_b };
    let road_c = if light { colors.road_a } else { colors.road_b };
    let rumble_c = if light { colors.rumble_a } else { colors.rumble_b };

    for y in y_top..=y_bot {
        let t = if sy1 == sy2 {
            0.5
        } else {
            ((y - sy2) as f32 / (sy1 - sy2) as f32).clamp(0.0, 1.0)
        };
        let cx = lerp(sx2 as f32, sx1 as f32, t);
        let hw = lerp(hw2, hw1, t);
        let le = (cx - hw).round() as i32;
        let re = (cx + hw).round() as i32;

        r.hline(y, 0, W as i32 - 1, grass_c);

        if si < 3 {
            // Start/finish checker band.
            let span = (re - le).max(8) as f32;
            let block = (span / 8.0).max(4.0) as i32;
            let mut x = le;
            let mut bi = 0usize;
            while x < re {
                let xe = (x + block).min(re);
                let c = if (bi + y as usize) % 2 == 0 { colors.checker_a } else { colors.checker_b };
                r.hline(y, x.max(le), xe, c);
                x += block;
                bi += 1;
            }
        } else {
            let rw = (hw * 0.15).max(1.0) as i32;
            r.hline(y, le, le + rw - 1, rumble_c);
            r.hline(y, re - rw + 1, re, rumble_c);
            r.hline(y, le + rw, re - rw, road_c);
            if si % 4 < 2 {
                let dw = (hw * 0.03).max(1.0) as i32;
                let cxi = cx.round() as i32;
                r.hline(y, cxi - dw / 2, cxi + dw / 2, colors.lane);
            }
        }
    }
}

/// Project a sprite anchored at segment `seg_i`, fraction `t` along it, with
/// lateral `offset` (in road half-widths). Returns (scale, screen x/y) or None
/// if the point is not in the visible/drawn range.
pub fn project_sprite(
    track: &Track,
    view: &RoadView,
    pos_wrapped: f32,
    player_x: f32,
    seg_i: usize,
    t: f32,
    offset: f32,
) -> Option<(f32, i32, i32)> {
    let sp = view.segs.iter().find(|s| s.index == seg_i)?;

    let length = track.length;
    let z_w = seg_i as f32 * SEG_LEN + t * SEG_LEN;
    let mut rel_z = z_w - pos_wrapped.rem_euclid(length);
    if rel_z < 0.0 {
        rel_z += length;
    }
    if rel_z <= CAM_DEPTH || rel_z > DRAW_DIST as f32 * SEG_LEN {
        return None;
    }

    let seg = &track.segments[seg_i];
    let xoff = lerp(sp.xoff1, sp.xoff2, t);
    let cam_x = player_x * ROAD_WIDTH - xoff;
    let world_y = seg.y0 + (seg.y1 - seg.y0) * t;

    let (scale, sx, sy) = project(offset * ROAD_WIDTH, world_y, rel_z, cam_x, view.cam_y);
    Some((scale, sx, sy))
}
