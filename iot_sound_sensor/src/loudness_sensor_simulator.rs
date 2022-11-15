use iot_sound_backend::loudness_data::LoudnessData;
use rand::Rng;

/// Represents a loudness sensor simulator
/// Generates random
pub struct LoudnessSensorSimulator {
    latest_loudness: f32,
}

impl LoudnessSensorSimulator {
    /// Create a new loudness sensor simulator
    pub fn new() -> Self {
        LoudnessSensorSimulator {
            latest_loudness: 30.0,
        }
    }

    /// Generates a random loudness value between 0 and 100
    /// Returns a `LoudnessData` struct with current generated loudness value and timestamp
    pub fn get_loudness_data(&mut self) -> LoudnessData {
        self.latest_loudness = self.next_loudness();

        let timestamp = std::time::SystemTime::now();
        LoudnessData::new(self.latest_loudness, timestamp)
    }

    /// Generates next random loudness value
    fn next_loudness(&mut self) -> f32 {
        rand::thread_rng().gen_range(0.0..100.0)
    }
}
