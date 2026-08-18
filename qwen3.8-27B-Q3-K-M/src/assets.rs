use std::collections::HashMap;

use crate::game::track::{coastal_circuit, mountain_pass, Track};
use crate::render::background::SkyTheme;
use crate::render::framebuffer::{Palette, W};
use crate::render::road::RoadColors;
use crate::render::sprite::Sprite;

/// Scenery sprite kind ids (match track::scatter_scenery).
pub const KIND_PINE: u8 = 1;
pub const KIND_BUSH: u8 = 2;
pub const KIND_ROCK: u8 = 3;
pub const KIND_SIGN: u8 = 4;
pub const KIND_FLAG: u8 = 5;

/// Screen-width constant (px * scale) per sprite kind, for world->screen sizing.
const K_PINE: f32 = 9000.0;
const K_BUSH: f32 = 7000.0;
const K_ROCK: f32 = 6500.0;
const K_SIGN: f32 = 14000.0;
const K_FLAG: f32 = 11000.0;
/// Cars render ~70px wide at player depth (scale ~= CAM_DEPTH/PLAYER_Z).
pub const K_CAR: f32 = 70_000.0;

pub struct TrackTheme {
    pub road: RoadColors,
    pub sky: SkyTheme,
    /// Distant hill silhouette strip (640 wide).
    pub hills_img: Sprite,
    /// Precomputed palette-cycling variants for each sky band (4 per band).
    pub sky_cycle: Vec<Vec<[u8; 3]>>,
}

pub struct Assets {
    /// Scenery sprites indexed by kind id (index 0 unused).
    pub scenery: Vec<Sprite>,
    pub k_scenery: [f32; 6],
    pub player_car: Sprite,
    pub ai_cars: Vec<Sprite>,
    pub tracks: Vec<(Track, TrackTheme)>,
    /// Palette indices for HUD text.
    pub txt_white: u8,
    pub txt_dim: u8,
}

fn car_map() -> &'static [&'static str] {
    &[
        "........GGGGGGGG........",
        "......GGGGGGGGGGGG......",
        "....BBBBGGGGGGGGGGBB....",
        "..BBBBBBBBBBBBBBBBBBBB..",
        ".BBBBBBBBBBBBBBBBBBBBBB.",
        ".BBRRBBBBBBBBBBBBBBRRBB.",
        ".DDDBBBBBBBBBBBBBBBBDDD.",
        "TTTTBBBBBBBBBBBBBBBBTTTT",
        "TTTTDDDDDDDDDDDDDDDDTTTT",
    ]
}

fn pine_map() -> &'static [&'static str] {
    &[
        "....GG....",
        "...GGGG...",
        "..GGGGGG..",
        "..GGGGGG..",
        ".GGGGGGGG.",
        ".GGGGGGGG.",
        "GGGGGGGGGG",
        "GGGGGGGGGG",
        "....TT....",
        "....TT....",
        "....TT....",
        "....TT....",
        "....TT....",
        "....TT....",
    ]
}

fn bush_map() -> &'static [&'static str] {
    &[
        "...GGGGGG...",
        ".GLLGGGGLLGG",
        "GGGGGGGGGGGG",
        ".GGGGGGGGGG.",
        ".GGGGGGGGGG.",
        "..GGGGGGGG..",
    ]
}

fn rock_map() -> &'static [&'static str] {
    &[
        "....KKkk....",
        "..KKKKkkkk..",
        ".KKKKKKkkkk.",
        "KKKKKKKkkkkk",
        ".kkkkkkkkk.",
        "..kkkkkkkk..",
    ]
}

fn sign_map() -> &'static [&'static str] {
    &[
        "WWWWWWWWWWWWWWWW",
        "WOOOOOOOOOOOOOOW",
        "WOOOOOOOOOOOOOOW",
        "WOOOOOOOOOOOOOOW",
        "WWWWWWWWWWWWWWWW",
        "WOOOOOOOOOOOOOOW",
        "WOOOOOOOOOOOOOOW",
        "WWWWWWWWWWWWWWWW",
        ".......PP.......",
        ".......PP.......",
        ".......PP.......",
    ]
}

fn flag_map() -> &'static [&'static str] {
    &[
        "WBWB",
        "BWBW",
        "WBWB",
        "BWBW",
        ".P..",
        ".P..",
        ".P..",
        ".P..",
        ".P..",
        ".P..",
        ".P..",
        ".P..",
        ".P..",
        ".P..",
        ".P..",
        ".P..",
    ]
}

fn gen_hills(far_c: u8, near_c: u8) -> Sprite {
    const HH: usize = 20;
    Sprite::generate(W, HH, |x, y| {
        let xa = x as f32;
        let far_h = (8.0 + 3.0 * ((std::f32::consts::TAU * 2.0 * xa / W as f32).sin())
            + 2.0 * ((std::f32::consts::TAU * 5.0 * xa / W as f32 + 1.7).sin()))
            .round() as usize;
        let near_h = (5.0 + 2.0 * ((std::f32::consts::TAU * 3.0 * xa / W as f32 + 0.8).sin())
            + 1.5 * ((std::f32::consts::TAU * 7.0 * xa / W as f32 + 2.9).sin()))
            .round() as usize;
        if y >= HH - far_h.min(HH) {
            Some(if y >= HH - near_h.min(HH) { near_c } else { far_c })
        } else {
            None
        }
    })
}

impl Assets {
    /// Build all sprites/tracks and the final 256-color palette.
    pub fn build() -> (Self, Palette) {
        let mut pal = Palette::new();

        // Base colors.
        let black = pal.add(15, 15, 15);
        let white = pal.add(245, 245, 245);
        let road_a = pal.add(155, 155, 160);
        let road_b = pal.add(125, 125, 130);
        let rumble_red = pal.add(205, 45, 40);
        let dim = pal.add(170, 170, 175);

        // Track themes.
        let mut tracks: Vec<(Track, TrackTheme)> = Vec::new();
        for (track, grass_a, grass_b, sky_top, sky_hor, hill_far_rgb, hill_near_rgb) in [
            (
                coastal_circuit(),
                [85u8, 165, 75],
                [65, 135, 58],
                [30, 80, 180],
                [170, 200, 230],
                [60, 90, 140],
                [90, 120, 160],
            ),
            (
                mountain_pass(),
                [150, 128, 70],
                [120, 100, 55],
                [70, 90, 160],
                [200, 180, 160],
                [80, 70, 110],
                [110, 95, 130],
            ),
        ] {
            let ga = pal.add(grass_a[0], grass_a[1], grass_a[2]);
            let gb = pal.add(grass_b[0], grass_b[1], grass_b[2]);
            let hf = pal.add(hill_far_rgb[0], hill_far_rgb[1], hill_far_rgb[2]);
            let hn = pal.add(hill_near_rgb[0], hill_near_rgb[1], hill_near_rgb[2]);
            let sky = SkyTheme::build(&mut pal, sky_top, sky_hor, 12);
            const NB: usize = 12;
            let mut sky_cycle = Vec::with_capacity(NB + 1);
            for i in 0..=NB {
                let t = i as f32 / NB as f32;
                let base: [f32; 3] = [
                    sky_top[0] as f32 + (sky_hor[0] as f32 - sky_top[0] as f32) * t,
                    sky_top[1] as f32 + (sky_hor[1] as f32 - sky_top[1] as f32) * t,
                    sky_top[2] as f32 + (sky_hor[2] as f32 - sky_top[2] as f32) * t,
                ];
                let mut variants = Vec::new();
                for (dr, dg, db) in [(0.0f32, 0.0, 0.0), (6.0, 2.0, -4.0), (-6.0, -2.0, 4.0), (3.0, 5.0, 2.0)] {
                    variants.push([
                        (base[0] + dr).clamp(0.0, 255.0) as u8,
                        (base[1] + dg).clamp(0.0, 255.0) as u8,
                        (base[2] + db).clamp(0.0, 255.0) as u8,
                    ]);
                }
                sky_cycle.push(variants);
            }
            let hills_img = gen_hills(hf, hn);
            let road = RoadColors {
                grass_a: ga,
                grass_b: gb,
                road_a,
                road_b,
                rumble_a: rumble_red,
                rumble_b: white,
                lane: white,
                checker_a: white,
                checker_b: black,
            };
            tracks.push((track, TrackTheme { road, sky, hills_img, sky_cycle }));
        }

        // Car colors.
        let glass = pal.add(140, 190, 220);
        let tire = black;
        let tail = pal.add(255, 60, 50);
        let player_body = pal.add(30, 60, 160);
        let player_dark = pal.add(18, 34, 92);

        let mut player_map_colors: HashMap<char, u8> = HashMap::new();
        player_map_colors.insert('G', glass);
        player_map_colors.insert('B', player_body);
        player_map_colors.insert('D', player_dark);
        player_map_colors.insert('T', tire);
        player_map_colors.insert('R', tail);
        let player_car = Sprite::from_map(car_map(), &player_map_colors);

        let ai_bodies: [([u8; 3], [u8; 3]); 4] = [
            ([200, 50, 40], [120, 26, 22]),
            ([230, 180, 40], [140, 108, 24]),
            ([60, 170, 70], [34, 100, 40]),
            ([225, 225, 225], [140, 140, 140]),
        ];
        let ai_cars: Vec<Sprite> = ai_bodies
            .iter()
            .map(|(b, d)| {
                let mut m: HashMap<char, u8> = HashMap::new();
                m.insert('G', glass);
                m.insert('B', pal.add(b[0], b[1], b[2]));
                m.insert('D', pal.add(d[0], d[1], d[2]));
                m.insert('T', tire);
                m.insert('R', tail);
                Sprite::from_map(car_map(), &m)
            })
            .collect();

        // Scenery.
        let mut pine_c = HashMap::new();
        pine_c.insert('G', pal.add(30, 90, 45));
        pine_c.insert('T', pal.add(120, 80, 40));
        let pine = Sprite::from_map(pine_map(), &pine_c);

        let mut bush_c = HashMap::new();
        bush_c.insert('G', pal.add(70, 140, 60));
        bush_c.insert('L', pal.add(105, 172, 82));
        let bush = Sprite::from_map(bush_map(), &bush_c);

        let mut rock_c = HashMap::new();
        rock_c.insert('K', pal.add(120, 120, 125));
        rock_c.insert('k', pal.add(88, 88, 94));
        let rock = Sprite::from_map(rock_map(), &rock_c);

        let mut sign_c = HashMap::new();
        sign_c.insert('W', white);
        sign_c.insert('O', pal.add(230, 140, 40));
        sign_c.insert('P', pal.add(70, 70, 75));
        let sign = Sprite::from_map(sign_map(), &sign_c);

        let mut flag_c = HashMap::new();
        flag_c.insert('W', white);
        flag_c.insert('B', black);
        flag_c.insert('P', pal.add(90, 90, 95));
        let flag = Sprite::from_map(flag_map(), &flag_c);

        let mut scenery = vec![Sprite { w: 1, h: 1, px: vec![] }]; // index 0 unused
        scenery.push(pine);
        scenery.push(bush);
        scenery.push(rock);
        scenery.push(sign);
        scenery.push(flag);

        let k_scenery = [0.0, K_PINE, K_BUSH, K_ROCK, K_SIGN, K_FLAG];

        (
            Assets {
                scenery,
                k_scenery,
                player_car,
                ai_cars,
                tracks,
                txt_white: white,
                txt_dim: dim,
            },
            pal,
        )
    }
}

