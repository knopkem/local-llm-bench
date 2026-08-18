pub mod synth;

use std::sync::{Arc, Mutex};

use rodio::buffer::SamplesBuffer;
use rodio::source::Source;
use rodio::{ChannelCount, DeviceSinkBuilder, MixerDeviceSink, Player, SampleRate};

const SR: u32 = 48000; // rodio resamples to the device rate automatically.

fn sr() -> SampleRate {
    SampleRate::new(SR).unwrap()
}

fn ch1() -> ChannelCount {
    ChannelCount::new(1).unwrap()
}

/// Shared mutable engine state (the queued source reads from this each sample).
#[derive(Clone)]
pub struct EngineInner {
    pub buf: Vec<f32>,
    pub pos: f64,
    /// Playback rate 0.5..2.5 (pitch follows speed).
    pub rate: f32,
    pub vol: f32,
}

struct EngineSource {
    inner: Arc<Mutex<EngineInner>>,
}

impl Iterator for EngineSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let mut g = self.inner.lock().unwrap();
        if g.vol <= 0.0 || g.buf.is_empty() {
            return Some(0.0);
        }
        let len = g.buf.len() as f64;
        let p = g.pos.rem_euclid(len);
        let i = p as usize;
        let frac = (p - i as f64) as f32;
        let a = g.buf[i];
        let b = g.buf[(i + 1).rem_euclid(g.buf.len())];
        g.pos += g.rate as f64;
        Some((a + (b - a) * frac) * g.vol)
    }
}

impl Source for EngineSource {
    fn current_span_len(&self) -> Option<usize> {
        None // infinite loop
    }
    fn channels(&self) -> ChannelCount {
        ch1()
    }
    fn sample_rate(&self) -> SampleRate {
        sr()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

/// Infinite looping music source (rodio 0.22 no longer exposes `repeating`).
struct MusicLoop {
    buf: Vec<f32>,
    pos: usize,
}

impl Iterator for MusicLoop {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let s = self.buf[self.pos];
        self.pos = (self.pos + 1).rem_euclid(self.buf.len());
        Some(s)
    }
}

impl Source for MusicLoop {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> ChannelCount {
        ch1()
    }
    fn sample_rate(&self) -> SampleRate {
        sr()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

pub struct AudioSys {
    sink: MixerDeviceSink,
    engine: Arc<Mutex<EngineInner>>,
    pub muted: bool,
    sfx_beep: Vec<f32>,
    sfx_go: Vec<f32>,
    sfx_bump: Vec<f32>,
    sfx_skid: Vec<f32>,
}

impl AudioSys {
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let sink = DeviceSinkBuilder::open_default_sink()?;
        let engine = Arc::new(Mutex::new(EngineInner {
            buf: synth::engine_loop(SR as f32),
            pos: 0.0,
            rate: 1.0,
            vol: 0.0,
        }));
        let p = Player::connect_new(sink.mixer());
        p.append(EngineSource { inner: engine.clone() });
        p.detach();

        Ok(AudioSys {
            sink,
            engine,
            muted: false,
            sfx_beep: synth::beep(SR as f32, 880.0, 0.12),
            sfx_go: synth::beep(SR as f32, 1320.0, 0.4),
            sfx_bump: synth::bump(SR as f32),
            sfx_skid: synth::skid(SR as f32),
        })
    }

    /// Update engine pitch/volume from the player's state.
    pub fn set_engine(&self, rpm_pct: f32, throttle: bool) {
        let mut g = self.engine.lock().unwrap();
        if self.muted {
            g.vol = 0.0;
            return;
        }
        g.rate = 0.55 + 1.9 * rpm_pct.clamp(0.0, 1.0);
        let base = 0.14 + 0.26 * rpm_pct.clamp(0.0, 1.0);
        g.vol = if throttle { base * 1.35 } else { base };
    }

    /// Silence the engine loop (menus / no race).
    pub fn engine_off(&self) {
        let mut g = self.engine.lock().unwrap();
        g.vol = 0.0;
        g.rate = 1.0;
    }

    pub fn start_music(&self, track_idx: usize) {
        let samples = synth::music_buffer(SR as f32, track_idx % 2);
        let p = Player::connect_new(self.sink.mixer());
        p.append(MusicLoop { buf: samples, pos: 0 });
        p.detach();
    }

    fn queue_sfx(&self, samples: &[f32]) {
        if self.muted {
            return;
        }
        let buf = SamplesBuffer::new(ch1(), sr(), samples.to_vec());
        let p = Player::connect_new(self.sink.mixer());
        p.append(buf);
        p.detach();
    }

    pub fn play_beep(&self) {
        self.queue_sfx(&self.sfx_beep);
    }

    pub fn play_go(&self) {
        self.queue_sfx(&self.sfx_go);
    }

    pub fn play_bump(&self) {
        self.queue_sfx(&self.sfx_bump);
    }

    pub fn play_skid(&self) {
        self.queue_sfx(&self.sfx_skid);
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        if self.muted {
            self.engine.lock().unwrap().vol = 0.0;
        }
        self.muted
    }
}
