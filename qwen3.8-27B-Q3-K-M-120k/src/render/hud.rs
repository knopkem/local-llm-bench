use crate::game::race::{fmt_time, Mode, RaceState, RACE_LAPS};
use super::framebuffer::Renderer;
use super::font::draw_text;
use super::H;

/// Draw the in-race HUD. `white`/`dim` are palette indices for text colors.
pub fn render_hud(r: &mut Renderer, race: &RaceState, white: u8, dim: u8) {
    // Top-left: lap counter + last/best lap.
    let laps = match race.mode {
        Mode::Race => format!("LAP {}/{RACE_LAPS}", race.player_laps_display()),
        Mode::TimeTrial => format!("LAP {}", race.player_laps_display().max(1)),
    };
    draw_text(r, 8, 6, &laps, white, 2);

    if let Some(last) = race.lap_times.last() {
        draw_text(r, 8, 20, &format!("LAST {}", fmt_time(*last)), dim, 1);
    }
    if let Some(best) = race.best_lap {
        draw_text(r, 8, 32, &format!("BEST {}", fmt_time(best)), white, 1);
    }

    // Top-right: race clock.
    let t = fmt_time(race.time);
    draw_text(r, (super::W as i32) - 8 - text_w(&t), 6, &t, white, 2);

    // Bottom-left: position in race mode.
    if race.mode == Mode::Race {
        let pos = race.player_position_display();
        draw_text(r, 8, H as i32 - 14, &format!("P{pos}"), white, 2);
    }

    // Bottom-right: speed readout.
    let mph = race.player.mph();
    let s = format!("{mph} MPH");
    draw_text(r, (super::W as i32) - 8 - text_w(&s), H as i32 - 14, &s, white, 2);

    // Speed bar gauge under the readout.
    let frac = (race.player.speed / crate::game::car::TOP_SPEED).clamp(0.0, 1.0);
    let bw = 96;
    let bx = (super::W as i32) - 8 - bw;
    let by = H as i32 - 34;
    r.rect(bx, by, bx + bw - 1, by + 5, dim);
    r.rect(bx, by, bx + ((bw as f32) * frac).round() as i32 - 1, by + 5, white);

    // Countdown overlay.
    if race.countdown > 0.0 {
        let n = race.countdown.ceil() as i32;
        let label: String = if n >= 4 {
            "READY".into()
        } else {
            n.to_string()
        };
        draw_text(r, (super::W as i32) / 2 - text_w(&label) / 2, H as i32 / 2 - 10, &label, white, 6);
    }

    if race.over {
        let label = "FINISH!";
        draw_text(r, (super::W as i32) / 2 - text_w(&label) / 2, H as i32 / 2 - 10, label, white, 6);
    }
}

fn text_w(s: &str) -> i32 {
    super::font::text_width(s, 2)
}
