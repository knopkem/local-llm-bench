mod app;
mod assets;
mod audio;
mod game;
mod png;
mod render;

use std::collections::HashSet;

use app::{App, Key};
use assets::Assets;
use audio::AudioSys;
use game::{InputState, STEP};
use render::framebuffer::Renderer;
use sdl2::keyboard::Keycode;
use render::{H, W};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(i) = args.iter().position(|a| a == "--shot") {
        let n = args.get(i + 1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(400);
        let track = arg_val(&args, "--track").unwrap_or(0).min(1);
        shot(n, track);
    } else if let Some(i) = args.iter().position(|a| a == "--sim") {
        let secs = args.get(i + 1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(60);
        sim(secs, arg_val(&args, "--track").unwrap_or(0).min(1));
    } else {
        windowed();
    }
}

fn arg_val(args: &[String], flag: &str) -> Option<usize> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).and_then(|s| s.parse::<usize>().ok())
}

/// Headless render check: simulate N steps (autopilot), save one frame as PNG.
fn shot(n_steps: u32, track_idx: usize) {
    let (assets, pal) = Assets::build();
    let mut r = Renderer::with_palette(W, H, pal);
    let (track, _) = &assets.tracks[track_idx];
    let mut scene = app::Scene::attract(track, track_idx);
    let input = InputState { throttle: true, brake: false, left: false, right: false };
    for _ in 0..n_steps {
        scene.update(STEP, &input, track);
    }
    scene.render(&mut r, &assets);
    let rgba = r.rgba();
    png::save_png("/tmp/lotus_shot.png", &rgba, W, H).expect("png write");
    println!("wrote /tmp/lotus_shot.png ({} steps, track {})", n_steps, track_idx);
}

/// Headless simulation check: run a full autopilot race, print results.
fn sim(secs: u32, track_idx: usize) {
    let (assets, _pal) = Assets::build();
    let (track, _) = &assets.tracks[track_idx];
    let mut race = game::race::RaceState::new(track, game::race::Mode::Race, track_idx, 1);
    race.autopilot = true;
    let input = InputState { throttle: true, brake: false, left: false, right: false };
    for _ in 0..secs * 60 {
        if race.over {
            break;
        }
        race.update(STEP, &input, track);
    }
    println!("track {} finished at {:.2}s ({} laps)", track.name, race.time, race.lap_times.len());
    for (id, name, t) in &race.results {
        println!("  P{} {:<14} {}", id + 1, name, game::race::fmt_time(*t));
    }
    if let Some(b) = race.best_lap {
        println!("  best lap {}", game::race::fmt_time(b));
    }
}

fn windowed() {
    let (assets, pal) = Assets::build();
    let mut app = App::new(assets, pal);
    let mut audio = match AudioSys::init() {
        Ok(a) => Some(a),
        Err(e) => {
            eprintln!("audio unavailable: {e}");
            None
        }
    };

    let sdl = sdl2::init().expect("sdl init");
    let video = sdl.video().expect("video subsystem");
    let window = video
        .window("LOTUS ESPRIT TURBO CHALLENGE", (W * 2) as u32, (H * 2) as u32)
        .position_centered()
        .build()
        .expect("window");
    let mut canvas = window.into_canvas().build().expect("canvas");
    // Render at native 640x256 and let SDL upscale with nearest-neighbor.
    canvas.set_logical_size(W as u32, H as u32).expect("logical size");
    let mut texture = canvas
        .create_texture(
            sdl2::pixels::PixelFormatEnum::RGBA32,
            sdl2::render::TextureAccess::Streaming,
            W as u32,
            H as u32,
        )
        .expect("texture");
    texture.set_scale_mode(sdl2::render::ScaleMode::Nearest);

    let mut pump = sdl.event_pump().expect("event pump");
    let mut held: HashSet<Keycode> = HashSet::new();
    let mut acc: f32 = 0.0;
    let mut last = std::time::Instant::now();
    let mut music_track: Option<usize> = None;

    'run: loop {
        for event in pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => break 'run,
                Event::KeyDown { keycode: Some(k), repeat, .. } => {
                    held.insert(k);
                    if !repeat {
                        if k == Keycode::M {
                            if let Some(a) = audio.as_mut() {
                                a.toggle_mute();
                            }
                        } else if let Some(key) = map_key(k) {
                            if app.handle_key(key) {
                                break 'run;
                            }
                        }
                    }
                }
                Event::KeyUp { keycode: Some(k), .. } => {
                    held.remove(&k);
                }
                _ => {}
            }
        }

        let now = std::time::Instant::now();
        let mut ft = now.duration_since(last).as_secs_f32();
        last = now;
        if ft > 0.1 {
            ft = 0.1;
        }
        acc += ft;

        while acc >= STEP {
            let input = input_from(&held);
            app.update(STEP, &input);
            let ev = app.take_events();
            if let Some(a) = audio.as_ref() {
                if ev.beep.is_some() {
                    a.play_beep();
                }
                if ev.go {
                    a.play_go();
                }
                if ev.bump {
                    a.play_bump();
                }
                match app.music_track() {
                    Some(ti) => {
                        a.set_engine(app.player_speed_frac(), input.throttle);
                        if music_track != Some(ti) {
                            a.start_music(ti);
                            music_track = Some(ti);
                        }
                    }
                    None => a.engine_off(),
                }
            }
            acc -= STEP;
        }

        app.render();
        let rgba = app.rgba();
        let _ = texture.with_lock(None, |buf, pitch| {
            let row = W * 4;
            for y in 0..H {
                buf[y * pitch..y * pitch + row].copy_from_slice(&rgba[y * row..(y + 1) * row]);
            }
        });
        canvas.copy(&texture, None, None).expect("draw");
        canvas.present();
    }
}

fn map_key(k: Keycode) -> Option<Key> {
    Some(match k {
        Keycode::RETURN => Key::Enter,
        Keycode::ESCAPE => Key::Esc,
        Keycode::TAB | Keycode::X => Key::Mode,
        Keycode::P => Key::Pause,
        _ => return None,
    })
}

fn input_from(held: &HashSet<Keycode>) -> InputState {
    let has = |ks: &[Keycode]| ks.iter().any(|k| held.contains(k));
    InputState {
        throttle: has(&[Keycode::UP, Keycode::W]),
        brake: has(&[Keycode::DOWN, Keycode::S]),
        left: has(&[Keycode::LEFT, Keycode::A]),
        right: has(&[Keycode::RIGHT, Keycode::D]),
    }
}
