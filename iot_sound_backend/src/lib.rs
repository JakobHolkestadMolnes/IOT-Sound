pub mod loudness_data {

    use serde::{Deserialize, Serialize};

    /// Struct for loudness data
    /// Represents a single measurement of loudness in decibel
    /// with a timestamp of when the measurement was taken.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct LoudnessData {
        db_level: f32,
        timestamp: std::time::SystemTime,
    }
    /// Create a new LoudnessData
    ///
    /// # Arguments
    ///
    /// * `db_level` - The loudness in decibel
    /// * `timestamp` - The time the loudness was measured
    impl LoudnessData {
        pub fn new(db_level: f32, timestamp: std::time::SystemTime) -> Self {
            LoudnessData {
                db_level,
                timestamp,
            }
        }
        /// Returns db_level of the LoudnessData
        pub fn db_level(&self) -> f32 {
            self.db_level
        }
        /// Returns timestamp of the LoudnessData
        pub fn timestamp(&self) -> std::time::SystemTime {
            self.timestamp
        }
        /// Parses a csv string into a LoudnessData.
        /// Returns a LoudnessData with the values from the csv string.
        ///
        /// # Arguments
        ///
        /// * `csv` - The csv string to parse
        pub fn parse_csv(csv: &str) -> Self {
            let mut iter = csv.split(',');
            let db_level = iter.next().unwrap().parse::<f32>().unwrap();
            let timestamp = iter.next().unwrap().parse::<u64>().unwrap();
            LoudnessData::new(
                db_level,
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp),
            )
        }

        /// Returns a csv string representation of the LoudnessData.
        /// db_level,timestamp
        pub fn to_csv(&self) -> String {
            format!(
                "{},{}",
                self.db_level,
                self.timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            )
        }
    }
}
