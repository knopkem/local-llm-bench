/// Procedural PCM synthesis (period-style chiptune / sample feel).
/// All buffers are mono f32 in [-1, 1].

fn square(p: f32) -> f32 {
    if p.fract() < 0.5 { 1.0 } else { -1.0 }
}

pub fn midi_freq(m: f32) -> f32 {
    440.0 * 2f32.powf((m - 69.0) / 12.0)
}

/// Add a square-wave note with a simple envelope into `out` at time t0 (seconds).
fn add_note(out: &mut Vec<f32>, sr: f32, freq: f32, t0: f32, dur: f32, amp: f32) {
    let i0 = (t0 * sr) as usize;
    if i0 >= out.len() {
        return;
    }
    let n = (dur * sr) as usize;
    let mut phase = 0.0f32;
    for i in 0..n.min(out.len() - i0) {
        let t = i as f32 / n.max(1) as f32;
        // Fast attack, gentle decay to sustain, short release at the end.
        let env = if t < 0.08 {
            t / 0.08
        } else if t > 0.92 {
            (1.0 - t) / 0.08
        } else {
            1.0 - (t - 0.08) * 0.35
        };
        phase += freq / sr;
        out[i0 + i] += square(phase) * amp * env.clamp(0.0, 1.0);
    }
}

fn add_kick(out: &mut Vec<f32>, sr: f32, t0: f32) {
    let i0 = (t0 * sr) as usize;
    if i0 >= out.len() {
        return;
    }
    let n = (0.12 * sr) as usize;
    let mut phase = 0.0f32;
    for i in 0..n.min(out.len() - i0) {
        let t = i as f32 / n.max(1) as f32;
        let f = 150.0 - 110.0 * t; // pitch drop
        phase += f / sr;
        out[i0 + i] += (phase.sin()) * (1.0 - t) * 0.5;
    }
}

fn add_hat(out: &mut Vec<f32>, sr: f32, t0: f32) {
    let i0 = (t0 * sr) as usize;
    if i0 >= out.len() {
        return;
    }
    let n = (0.045 * sr) as usize;
    for i in 0..n.min(out.len() - i0) {
        let t = i as f32 / n.max(1) as f32;
        // Deterministic pseudo-noise.
        let nz = ((i as f32 * 12.9898 + t0).sin() * 43758.5453).fract() * 2.0 - 1.0;
        out[i0 + i] += nz * (1.0 - t) * 0.09;
    }
}

/// 0.3s engine growl loop: sawtooth + sub sine + noise texture.
pub fn engine_loop(sr: f32) -> Vec<f32> {
    let n = (0.3 * sr) as usize;
    let mut out = vec![0f32; n];
    for i in 0..n {
        let t = i as f32 / sr;
        let saw = 2.0 * (t * 55.0).fract() - 1.0;
        let sub = (std::f32::consts::TAU * t * 110.0).sin();
        let nz = ((i as f32 * 12.9898).sin() * 43758.5453).fract() * 2.0 - 1.0;
        out[i] = saw * 0.45 + sub * 0.3 + nz * 0.12;
    }
    out
}

/// Simple square beep with envelope.
pub fn beep(sr: f32, freq: f32, dur: f32) -> Vec<f32> {
    let n = (dur * sr) as usize;
    let mut out = vec![0f32; n];
    add_note(&mut out, sr, freq, 0.0, dur, 0.4);
    out
}

/// Low thud + noise burst for collisions.
pub fn bump(sr: f32) -> Vec<f32> {
    let n = (0.18 * sr) as usize;
    let mut out = vec![0f32; n];
    add_kick(&mut out, sr, 0.0);
    for i in 0..n {
        let t = i as f32 / n.max(1) as f32;
        let nz = ((i as f32 * 78.233).sin() * 43758.5453).fract() * 2.0 - 1.0;
        out[i] += nz * (1.0 - t) * 0.2;
    }
    out
}

/// Decaying noise burst for skids / off-road rumble.
pub fn skid(sr: f32) -> Vec<f32> {
    let n = (0.45 * sr) as usize;
    let mut out = vec![0f32; n];
    for i in 0..n {
        let t = i as f32 / n.max(1) as f32;
        let nz = ((i as f32 * 12.9898).sin() * 43758.5453).fract() * 2.0 - 1.0;
        out[i] += nz * (1.0 - t) * 0.22;
    }
    out
}

/// 8-bar chiptune loop at 120 BPM: square bass + lead, kick, hats.
pub fn music_buffer(sr: f32, variant: usize) -> Vec<f32> {
    let beat = 60.0 / 120.0; // 0.5s per beat
    let bars = 8usize;
    let total_beats = bars * 4;
    let dur = total_beats as f32 * beat + 0.2; // small tail for releases
    let n = (dur * sr) as usize;
    let mut out = vec![0f32; n];

    // Am - F - C - G, repeated twice.
    let roots: [f32; 8] = [45.0, 41.0, 48.0, 43.0, 45.0, 41.0, 48.0, 43.0];

    // Bass: eighth notes on the root (fifth bounce in the second half of each bar).
    for bar in 0..bars {
        let root = roots[bar];
        for e in 0..8usize {
            let t0 = (bar * 4 + e / 2) as f32 * beat;
            let m = if e >= 4 && e % 2 == 1 { root + 7.0 } else { root };
            add_note(&mut out, sr, midi_freq(m), t0, beat * 0.45, 0.3);
        }
    }

    // Lead melody (two variants).
    let mel_a: [(f32, f32, f32); 28] = [
        (0.0, 69.0, 1.0), (1.0, 72.0, 1.0), (2.0, 76.0, 1.0), (3.0, 74.0, 1.0),
        (4.0, 72.0, 1.0), (5.0, 69.0, 1.0), (6.0, 67.0, 1.0), (7.0, 69.0, 1.0),
        (8.0, 65.0, 1.0), (9.0, 69.0, 1.0), (10.0, 72.0, 1.0), (11.0, 69.0, 1.0),
        (12.0, 67.0, 1.0), (13.0, 72.0, 1.0), (14.0, 76.0, 1.0), (15.0, 74.0, 1.0),
        (16.0, 71.0, 1.0), (17.0, 74.0, 1.0), (18.0, 79.0, 1.0), (19.0, 74.0, 1.0),
        (20.0, 71.0, 1.0), (21.0, 67.0, 1.0), (22.0, 71.0, 1.5), (23.5, 74.0, 0.5),
        (24.0, 71.0, 1.0), (25.0, 74.0, 1.0), (26.0, 79.0, 1.0), (30.0, 74.0, 2.0),
    ];
    let mel_b: [(f32, f32, f32); 28] = [
        (0.0, 65.0, 1.0), (1.0, 69.0, 1.0), (2.0, 72.0, 1.0), (3.0, 69.0, 1.0),
        (4.0, 65.0, 1.0), (5.0, 67.0, 1.0), (6.0, 69.0, 1.0), (7.0, 67.0, 1.0),
        (8.0, 64.0, 1.0), (9.0, 65.0, 1.0), (10.0, 69.0, 1.0), (11.0, 65.0, 1.0),
        (12.0, 67.0, 1.0), (13.0, 69.0, 1.0), (14.0, 72.0, 1.0), (15.0, 69.0, 1.0),
        (16.0, 67.0, 1.0), (17.0, 69.0, 1.0), (18.0, 74.0, 1.0), (19.0, 69.0, 1.0),
        (20.0, 67.0, 1.0), (21.0, 65.0, 1.0), (22.0, 67.0, 1.5), (23.5, 69.0, 0.5),
        (24.0, 67.0, 1.0), (25.0, 69.0, 1.0), (26.0, 74.0, 1.0), (30.0, 69.0, 2.0),
    ];
    let mel = if variant == 0 { &mel_a } else { &mel_b };
    for &(t0b, m, db) in mel.iter() {
        add_note(&mut out, sr, midi_freq(m), t0b * beat, db * beat * 0.92, 0.26);
    }

    // Kick on beats 1 & 3 of each bar; hats on off-beat eighths.
    for bar in 0..bars {
        add_kick(&mut out, sr, (bar * 4) as f32 * beat);
        add_kick(&mut out, sr, (bar * 4 + 2) as f32 * beat);
        for e in 1..8usize {
            if e % 2 == 1 {
                add_hat(&mut out, sr, ((bar * 4) as f32 + e as f32 / 2.0) * beat);
            }
        }
    }

    // Soft-clip to keep peaks in range.
    for s in out.iter_mut() {
        *s = (*s).clamp(-1.0, 1.0) * 0.9;
    }
    out
}
