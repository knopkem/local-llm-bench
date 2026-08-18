use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::track::{Track, ROAD_WIDTH};
use crate::player::PlayerState;
use crate::ai::AIState;
use crate::sprites::{Sprite, SpriteType};

const SW: i32 = 640;
const SH: i32 = 480;
const VSEG: usize = 250;
const FOV: f32 = 0.8;
const CH: f32 = 150.0;

pub struct Renderer { sky: Vec<Color> }

impl Renderer {
    pub fn new() -> Self {
        let mut sky = Vec::with_capacity(240);
        for i in 0..240usize {
            let t = i as f32 / 240.0;
            sky.push(Color::RGB(
                (20.0 + 180.0 * t).round() as u8,
                (10.0 + 60.0 * t).round() as u8,
                (60.0 - 40.0 * t).max(0.0).round() as u8,
            ));
        }
        Renderer { sky }
    }

    pub fn render(
        &mut self, c: &mut Canvas<Window>, trk: &Track, p: &PlayerState,
        ai: &[AIState], sps: &[Sprite], tm: f32, lp: u32, tlp: u32,
    ) {
        let cx = SW as f32 * 0.5;
        let cy = SH as f32 * 0.5;

        for i in 0..240usize {
            c.set_draw_color(self.sky[i]);
            c.fill_rect(Rect::new(0, i as i32, SW as u32, 1)).unwrap();
        }
        c.set_draw_color(Color::RGB(30, 50, 20));
        c.fill_rect(Rect::new(0, 240, SW as u32, SH as u32 - 240)).unwrap();

        let (px, py) = p.position_on_track(trk);
        let pz = p.z;
        let si = ((pz + 1.0) / 1.0).floor() as usize;

        for i in (0..VSEG).rev() {
            let idx = (si + i) % trk.segments.len();
            let s = &trk.segments[idx];
            let mut dz = s.world_z - pz;
            if dz <= 0.0 { dz += trk.total_length; }
            if dz < 2.0 || dz > 2500.0 { continue; }

            let sc = FOV / dz;
            let sx = cx + (s.world_x - px) * sc * SW as f32 * 0.5;
            let sy = cy + (CH - s.world_y + py) * sc * SH as f32 * 0.3;
            let sw = ROAD_WIDTH * 0.5 * sc * SW as f32 * 0.5;

            if i < VSEG - 1 {
                let ni = (si + i + 1) % trk.segments.len();
                let ns = &trk.segments[ni];
                let mut ndz = ns.world_z - pz;
                if ndz <= 0.0 { ndz += trk.total_length; }
                if ndz > 2.0 && ndz < 2500.0 {
                    let nsc = FOV / ndz;
                    let nx = cx + (ns.world_x - px) * nsc * SW as f32 * 0.5;
                    let ny = cy + (CH - ns.world_y + py) * nsc * SH as f32 * 0.3;
                    let nw = ROAD_WIDTH * 0.5 * nsc * SW as f32 * 0.5;
                    self.dseg(c, sx, sy, sw, nx, ny, nw, i);
                }
            }
        }

        let mut sl: Vec<(f32, &Sprite)> = sps.iter()
            .map(|sp| { let mut dz = sp.world_z - pz; if dz <= 0.0 { dz += trk.total_length; } (dz, sp) })
            .filter(|(d, _)| *d > 10.0 && *d < 2000.0).collect();
        sl.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        for (_, sp) in &sl { self.dsp(c, sp, px, py, pz, trk.total_length); }

        let mut al: Vec<(f32, &AIState)> = ai.iter()
            .map(|a| { let mut dz = a.z - pz; if dz <= 0.0 { dz += trk.total_length; } (dz, a) })
            .filter(|(d, _)| *d > 10.0 && *d < 1500.0).collect();
        al.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        for (_, a) in &al { self.dai(c, a, px, py, pz, trk.total_length); }

        self.dpl(c, cx, SH as f32 - 80.0);
        self.dhud(c, p.speed, lp, tlp, tm, ai, pz, trk.total_length);
    }

    fn dseg(&self, c: &mut Canvas<Window>, x1: f32, y1: f32, w1: f32, x2: f32, y2: f32, w2: f32, si: usize) {
        let x1i = x1 as i32; let y1i = y1 as i32; let w1i = w1 as i32;
        let x2i = x2 as i32; let y2i = y2 as i32; let w2i = w2 as i32;
        if y1i == y2i { return; }

        c.set_draw_color(if (si / 3) % 2 == 0 { Color::RGB(80, 80, 80) } else { Color::RGB(70, 70, 70) });
        let ys = y2i.min(y1i); let ye = y2i.max(y1i);

        for y in ys..ye {
            let t = if y1i != y2i { (y - ys) as f32 / (ye - ys) as f32 } else { 0.0 };
            let xm = (x1i as f32 * (1.0 - t) + x2i as f32 * t) as i32;
            let wd = ((w1i as f32 * (1.0 - t) + w2i as f32 * t) as i32).max(1);
            let l = (xm - wd).max(0);
            let dw = ((wd * 2).min(SW - l)).max(0);
            if dw > 0 { c.fill_rect(Rect::new(l, y, dw as u32, 1)).ok(); }
        }

        let rm = (si / 2) % 2;
        c.set_draw_color(if rm == 0 { Color::RGB(200, 0, 0) } else { Color::RGB(220, 220, 220) });
        let rw = (w1i as f32 * 0.08).max(2.0) as i32;
        if rw > 0 {
            c.fill_rect(Rect::new((x1i - w1i - rw).max(0), y1i, rw as u32, 1)).ok();
            let rr = x1i + w1i;
            if rr >= 0 { c.fill_rect(Rect::new(rr, y1i, rw as u32, 1)).ok(); }
        }

        if si % 6 < 3 {
            c.set_draw_color(Color::RGB(255, 255, 255));
            let lw = (w1i as f32 * 0.02).max(1.0) as i32;
            if lw > 0 { c.fill_rect(Rect::new((x1i - lw).max(0), y1i, (lw * 2) as u32, 1)).ok(); }
        }
    }

    fn dsp(&self, c: &mut Canvas<Window>, sp: &Sprite, px: f32, py: f32, pz: f32, tl: f32) {
        let mut dz = sp.world_z - pz; if dz <= 0.0 { dz += tl; }
        if dz < 10.0 || dz > 2000.0 { return; }
        let sc = FOV / dz;
        let sx = (SW as f32 * 0.5 + (sp.world_x - px) * sc * SW as f32 * 0.5) as i32;
        let sy = (SH as f32 * 0.5 + (CH - sp.world_y + py) * sc * SH as f32 * 0.3) as i32;
        let sh = (sp.sprite_type.height() * sc * SH as f32 * 0.5) as i32;
        let sw = (sp.sprite_type.width() * sc * SW as f32 * 0.5) as i32;
        if sh < 2 || sw < 2 { return; }

        c.set_draw_color(Color::RGB(sp.sprite_type.color()[0], sp.sprite_type.color()[1], sp.sprite_type.color()[2]));

        match sp.sprite_type {
            SpriteType::TreeLarge | SpriteType::TreeSmall => {
                let tw = (sw as f32 * 0.2).max(2.0) as i32;
                let th = (sh as f32 * 0.4).max(4.0) as i32;
                if tw > 0 && th > 0 { c.fill_rect(Rect::new(sx - tw/2, sy - sh + th/2, tw as u32, th as u32)).ok(); }
                let cw = (sw as f32 * 0.8) as i32;
                for i in 0..sh {
                    let w = (cw as f32 * (1.0 - i as f32 / sh as f32)) as i32;
                    if w > 0 { c.fill_rect(Rect::new(sx - w/2, sy - sh + i, w as u32, 1)).ok(); }
                }
            }
            SpriteType::BarrierRed | SpriteType::BarrierWhite => {
                let h = (sh as f32 * 0.5).max(4.0) as i32;
                if h > 0 { c.fill_rect(Rect::new(sx, sy - h, 4, h as u32)).ok(); }
                if sw > 0 { c.fill_rect(Rect::new(sx - sw/2, sy - sh, sw as u32, 6)).ok(); }
            }
            SpriteType::Bush | SpriteType::Rock => {
                let sz = (sw.min(sh) as f32 * 0.4) as i32;
                if sz > 2 { c.fill_rect(Rect::new(sx - sz/2, sy - sz, sz as u32, (sz * 2 / 3).max(1) as u32)).ok(); }
            }
        }
    }

    fn dai(&self, c: &mut Canvas<Window>, a: &AIState, _px: f32, py: f32, pz: f32, tl: f32) {
        let mut dz = a.z - pz; if dz <= 0.0 { dz += tl; }
        if dz < 10.0 || dz > 1500.0 { return; }
        let sc = FOV / dz;
        let sx = SW as i32 / 2;
        let sy = (SH as f32 * 0.5 + CH * sc * SH as f32 * 0.3) as i32;
        let cw = (40.0 * sc * SW as f32 * 0.5) as i32;
        let ch = (25.0 * sc * SH as f32 * 0.5) as i32;
        if cw < 3 || ch < 2 { return; }

        c.set_draw_color(Color::RGB(a.color[0], a.color[1], a.color[2]));
        if cw > 0 && ch > 0 {
            c.fill_rect(Rect::new(sx - cw/2, sy - ch, cw as u32, ch as u32)).ok();
            let tw = (cw as f32 * 0.6) as i32;
            let th = (ch as f32 * 0.4) as i32;
            if tw > 0 && th > 0 { c.fill_rect(Rect::new(sx - tw/2, sy - ch - th, tw as u32, th as u32)).ok(); }
        }
    }

    fn dpl(&self, c: &mut Canvas<Window>, x: f32, y: f32) {
        let cw = 60u32; let ch = 35u32; let tw = 35u32; let th = 20u32;
        c.set_draw_color(Color::RGB(220, 30, 30));
        c.fill_rect(Rect::new((x - cw as f32 * 0.5) as i32, y as i32, cw, ch)).ok();
        c.set_draw_color(Color::RGB(180, 20, 20));
        c.fill_rect(Rect::new((x - tw as f32 * 0.5) as i32, y as i32 - th as i32, tw, th)).ok();
        c.set_draw_color(Color::RGB(100, 150, 200));
        c.fill_rect(Rect::new((x - 15.0) as i32, y as i32 - th as i32 + 2, 30, 8)).ok();
        c.set_draw_color(Color::RGB(30, 30, 30));
        let ww = 12u32; let wh = 6u32;
        let lfx = (x - cw as f32 * 0.5 - ww as f32) as i32;
        let rfx = (x + cw as f32 * 0.5) as i32;
        c.fill_rect(Rect::new(lfx.max(0), y as i32, ww, wh)).ok();
        c.fill_rect(Rect::new(rfx, y as i32, ww, wh)).ok();
        c.fill_rect(Rect::new(lfx.max(0), y as i32 + ch as i32 - wh as i32, ww, wh)).ok();
        c.fill_rect(Rect::new(rfx, y as i32 + ch as i32 - wh as i32, ww, wh)).ok();
    }

    fn dhud(&self, c: &mut Canvas<Window>, spd: f32, lp: u32, tlp: u32, tm: f32, ai: &[AIState], pz: f32, tl: f32) {
        let sk = (spd * 1.5) as i32;
        c.set_draw_color(Color::RGB(0, 0, 0));
        c.fill_rect(Rect::new(SW - 180, SH - 70, 170, 60)).ok();
        c.set_draw_color(Color::RGB(255, 255, 255));
        self.dstr(c, &format!("{}", sk), SW - 170, SH as u32 - 60);

        let pos = self.cpos(ai, pz, tl);
        c.set_draw_color(Color::RGB(0, 0, 0));
        c.fill_rect(Rect::new(10, 10, 120, 35)).ok();
        c.set_draw_color(Color::RGB(255, 255, 255));
        self.dstr(c, &format!("P{}/6", pos), 20, 20);

        c.set_draw_color(Color::RGB(0, 0, 0));
        c.fill_rect(Rect::new(SW/2 - 60, 10, 120, 35)).ok();
        c.set_draw_color(Color::RGB(255, 255, 255));
        self.dstr(c, &if tlp > 1 { format!("L{}/{}", lp, tlp) } else { format!("L{}", lp) }, SW/2 - 50, 20);

        c.set_draw_color(Color::RGB(0, 0, 0));
        c.fill_rect(Rect::new(SW - 130, 10, 120, 35)).ok();
        c.set_draw_color(Color::RGB(255, 255, 255));
        self.dstr(c, &format!("{:.2}s", tm), SW - 120, 20);
    }

    fn cpos(&self, ai: &[AIState], pz: f32, tl: f32) -> u32 {
        let mut p = 1u32;
        for a in ai { let az = if a.z >= pz { a.z } else { a.z + tl }; if az > pz { p += 1; } }
        p.min(6)
    }

    fn dstr(&self, c: &mut Canvas<Window>, t: &str, x: i32, y: u32) {
        for (i, _) in t.chars().enumerate() {
            c.fill_rect(Rect::new((x + i as i32 * 8), y as i32, 8, 12)).ok();
        }
    }
}
