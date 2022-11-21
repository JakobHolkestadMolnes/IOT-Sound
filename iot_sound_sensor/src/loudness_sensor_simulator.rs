use iot_sound_backend::loudness_data::LoudnessData;
use rand::Rng;

const MAX_SENSOR_VALUE: f32 = 100.0;
const MIN_SENSOR_VALUE: f32 = 0.0;

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
        let change = rand::thread_rng().gen_range(-10.0..=10.0);
        let mut loudness = self.latest_loudness + change;
        // if the new loudness is out of bounds, change in the opposite direction
        if loudness > MAX_SENSOR_VALUE || loudness < MIN_SENSOR_VALUE {
            loudness = self.latest_loudness - change;
        }
        return loudness;
    }
}
