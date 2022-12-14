use iot_sound_backend::loudness_data::LoudnessData;
use rand::Rng;
use std::time::SystemTime;

const DAY_MAX_SENSOR_VALUE: f32 = 100.0;
const DAY_MIN_SENSOR_VALUE: f32 = 40.0;
const NIGHT_MAX_SENSOR_VALUE: f32 = 50.0;
const NIGHT_MIN_SENSOR_VALUE: f32 = 0.0;

#[derive(PartialEq)]
enum DayNight {
    Day,
    Night,
}

/// Represents a loudness sensor simulator
/// Generates random
pub struct LoudnessSensorSimulator {
    latest_loudness: f32,
    state: DayNight,
    last_state_change: SystemTime,
}

impl LoudnessSensorSimulator {
    /// Create a new loudness sensor simulator
    pub fn new() -> Self {
        LoudnessSensorSimulator {
            latest_loudness: 30.0,
            state: DayNight::Night,
            last_state_change: SystemTime::now(),
        }
    }

    /// Generates a random loudness value between 0 and 100
    /// Returns a `LoudnessData` struct with current generated loudness value and timestamp
    pub fn get_loudness_data(&mut self) -> LoudnessData {
        self.latest_loudness = self.next_loudness();

        let timestamp = SystemTime::now();
        LoudnessData::new(self.latest_loudness, timestamp)
    }

    /// Generates next random loudness value
    fn next_loudness(&mut self) -> f32 {
        let time_since_state_change = self.last_state_change.elapsed().unwrap().as_secs();
        if time_since_state_change >= 60 {
            match self.state {
                DayNight::Day => {
                    self.state = DayNight::Night;
                    self.latest_loudness = clampf32(
                        self.latest_loudness,
                        NIGHT_MIN_SENSOR_VALUE,
                        NIGHT_MAX_SENSOR_VALUE,
                    );
                    self.night_loudness()
                }
                DayNight::Night => {
                    self.state = DayNight::Day;
                    self.latest_loudness = clampf32(
                        self.latest_loudness,
                        DAY_MIN_SENSOR_VALUE,
                        DAY_MAX_SENSOR_VALUE,
                    );
                    self.day_loudness()
                }
            }
        } else {
            match self.state {
                DayNight::Day => self.day_loudness(),
                DayNight::Night => self.night_loudness(),
            }
        }
    }

    fn night_loudness(&mut self) -> f32 {
        let change = rand::thread_rng().gen_range(-5.0..=5.0);
        let mut loudness = self.latest_loudness + change;
        // if the new loudness is out of bounds, change in the opposite direction
        if !(NIGHT_MIN_SENSOR_VALUE..=NIGHT_MAX_SENSOR_VALUE).contains(&loudness) {
            loudness = self.latest_loudness - change;
        }
        loudness
    }

    fn day_loudness(&mut self) -> f32 {
        let change = rand::thread_rng().gen_range(-10.0..=10.0);
        let mut loudness = self.latest_loudness + change;
        // if the new loudness is out of bounds, change in the opposite direction
        if !(DAY_MIN_SENSOR_VALUE..=DAY_MAX_SENSOR_VALUE).contains(&loudness) {
            loudness = self.latest_loudness - change;
        }
        loudness
    }
}

fn clampf32(variable: f32, lower: f32, upper: f32) -> f32 {
    if variable < lower {
        return lower;
    } else if variable > upper {
        return upper;
    }
    variable
}
