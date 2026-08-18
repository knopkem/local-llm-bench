use std::f32::consts::PI;

pub struct SoundEngine {
    enabled: bool,
    engine_phase: f32,
    music_phase: f32,
}

impl SoundEngine {
    pub fn new() -> Self {
        // Audio initialization is optional - we'll try but not fail if it doesn't work
        let enabled = true; // Assume audio is available
        
        SoundEngine {
            enabled,
            engine_phase: 0.0,
            music_phase: 0.0,
        }
    }

    pub fn update(&mut self, speed: f32, _race_time: f32) {
        if !self.enabled {
            return;
        }

        // Update engine sound phase based on speed
        let engine_freq = 40.0 + speed * 0.3;
        self.engine_phase += (engine_freq / 22050.0) * 2.0 * PI;
        if self.engine_phase > 2.0 * PI {
            self.engine_phase -= 2.0 * PI;
        }

        // Update music phase
        self.music_phase += 0.01;
        if self.music_phase > 2.0 * PI {
            self.music_phase -= 2.0 * PI;
        }
    }

    pub fn generate_audio_frame(&self, buffer: &mut [i16]) {
        if !self.enabled {
            return;
        }

        for sample in buffer.iter_mut() {
            // Engine sound (sawtooth wave)
            let engine_sample = ((self.engine_phase % (2.0 * PI)) / (2.0 * PI)).fract();
            let engine = (engine_sample * 2.0 - 1.0) as f32 * 8000.0;

            // Background music (simple arpeggio)
            let note_freqs: [f32; 4] = [261.63, 329.63, 392.0, 523.25];
            let beat = (self.music_phase / PI).floor() as usize % 4;
            let music_freq = note_freqs[beat];
            let music_sample = ((self.music_phase * music_freq) % (2.0 * PI)).sin() as f32 * 5000.0;

            // Mix and clamp
            let mixed = (engine + music_sample) / 2.0;
            *sample = mixed.max(-32768.0).min(32767.0) as i16;
        }
    }
}
