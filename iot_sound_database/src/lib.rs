use chrono::{DateTime, Local, Utc};
use deadpool_postgres::{self, CreatePoolError};
use serde::{Deserialize, Serialize};
use serde_json::{self, json};

/// Struct that contains a pool of postgres connections
#[derive(Clone)]
pub struct Pool {
    pool: deadpool_postgres::Pool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Sensor {
    id: String,
    type_: String,
    location: String,
}

impl Sensor {
    pub fn new(id: String, type_: String, location: String) -> Sensor {
        Sensor {
            id,
            type_,
            location,
        }
    }
}

/// Struct for data from the database that can be converted to json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    id: i32,
    sensor_name: String,
    sound: String,
    time: std::time::SystemTime,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DataWithDateTimeString {
    id: i32,
    sensor_name: String,
    sound: String,
    time: std::time::SystemTime,
    time_string: String,
}

// implement a trait for vec of data
impl Data {
    /// Create a new Data struct
    /// # Arguments
    /// * `id` - The id of the data
    /// * `sensor_name` - The name of the sensor
    /// * `sound` - The sound level
    /// * `time` - The time the data was created
    /// # Returns
    /// `Data` - The new Data struct
    pub fn new(id: i32, sound: String, sensor_name: String, time: std::time::SystemTime) -> Data {
        Data {
            id,
            sensor_name,
            sound,
            time,
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_sensor_name(&self) -> String {
        self.sensor_name.clone()
    }
    pub fn get_date_time_string(&self) -> DataWithDateTimeString {
        let time_string = self.time;
        let datetime = DateTime::<Utc>::from(time_string);
        let local_time: DateTime<Local> = DateTime::from(datetime);
        let datetime_string = local_time.to_rfc2822();
        DataWithDateTimeString {
            id: self.id,
            sensor_name: self.sensor_name.clone(),
            sound: self.sound.clone(),
            time: self.time,
            time_string: datetime_string,
        }
    }
}

// implement a trait for vec of data
impl From<Data> for serde_json::Value {
    fn from(data: Data) -> Self {
        json!({
            "id": data.id,
            "sensor_name": data.sensor_name,
            "sound": data.sound,
            "time": data.time,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Log {
    id: i64,
    message: String,
    time: std::time::SystemTime,
}

impl Log {
    pub fn new(id: i64, message: String, time: std::time::SystemTime) -> Log {
        Log {
            id,
            message,
            time,
        }
    }
}

impl Pool {
    /// Create a new Pool struct
    /// # Arguments
    /// * `host` - The host of the database
    /// * `port` - The port of the database
    /// * `user` - The user of the database
    /// * `password` - The password of the database
    /// * `database` - The database to connect to
    /// # Returns
    /// `Pool` - The new Pool struct
    pub async fn new(
        host: Option<String>,
        port: Option<u16>,
        user: Option<String>,
        password: Option<String>,
        dbname: Option<String>,
    ) -> Result<Pool, CreatePoolError> {
        let config = deadpool_postgres::Config {
            user,
            password,
            host,
            port,
            dbname,
            ..Default::default()
        };
        let pool = config.create_pool(None, tokio_postgres::NoTls);
        let pool = match pool {
            Ok(pool) => pool,
            Err(e) => {
                println!("Error: {}", e);
                return Err(e);
            }
        };
        Ok(Pool { pool })
    }

    /// Create the sensor table if it does not exist
    /// # Arguments
    /// * `self` - The Pool struct
    ///
    /// # Returns
    /// `Result<(), tokio_postgres::Error>` - The result of the query
    pub async fn create_sensor_table(&self) -> Result<(), deadpool_postgres::PoolError> {
        let allowed_sensors =
            "'loudness', 'temperature', 'humidity', 'light', 'air_quality', 'oxygen', 'co2'";
        let create_sensor_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS sensor (
        id text PRIMARY KEY,
        type text NOT NULL CHECK (type IN ({allowed_sensors})),
        location text NOT NULL);"
        );

        let client = self.pool.get().await?;
        client.execute(&create_sensor_table_sql, &[]).await?;
        Ok(())
    }

    /// Create the table containing the data if it does not exist
    /// # Arguments
    /// * `self` - The Pool struct
    ///
    /// # Returns
    /// `Result<(), tokio_postgres::Error>` - The result of the query
    pub async fn create_loudness_table(&self) -> Result<(), deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "CREATE TABLE IF NOT EXISTS loudness (
                    id SERIAL PRIMARY KEY,
                    sensor_id text REFERENCES sensor(id),
                    level text NOT NULL,
                    time timestamp NOT NULL);",
                &[],
            )
            .await?;
        Ok(())
    }

    /// Return all the data from the database
    /// # Arguments
    /// * `self` - The Pool struct
    ///
    /// # Returns
    /// `Result<Vec<Data>, tokio_postgres::Error>` - The result of the query
    pub async fn get_loudness(&self) -> Result<Vec<Data>, deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        let statement = client.prepare("SELECT * FROM loudness").await?;
        let rows = client.query(&statement, &[]).await?;
        let mut data = Vec::new();

        for row in rows {
            data.push(Data {
                id: row.get(0),
                sensor_name: row.get(1),
                sound: row.get(2),
                time: row.get(3),
            });
        }
        Ok(data)
    }

    /// Return *n* records of given sensor data from the database
    /// # Arguments
    /// * `self` - The Pool struct
    /// * `sensor_name` - The name of the sensor
    /// * `n` - The number of records to return
    ///
    /// # Returns
    /// `Result<Vec<Data>, tokio_postgres::Error>` - The result of the query
    pub async fn get_loudness_limited(
        &self,
        sensor_name: &str,
        n: i64,
    ) -> Result<Vec<Data>, deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        let statement = client
            .prepare(
                "
        WITH latest_n AS
        (SELECT * FROM loudness WHERE sensor_id = $1 ORDER BY time DESC LIMIT $2)
        SELECT * FROM latest_n ORDER BY time ASC
        ",
            )
            .await?;
        let rows = client.query(&statement, &[&sensor_name, &n]).await?;
        let mut data = Vec::new();

        for row in rows {
            data.push(Data {
                id: row.get(0),
                sensor_name: row.get(1),
                sound: row.get(2),
                time: row.get(3),
            });
        }
        Ok(data)
    }

    /// Return all sensors from the database
    /// # Arguments
    /// * `self` - The Pool struct
    /// # Returns
    /// `Result<Vec<Sensor>, tokio_postgres::Error>` - The result of the query
    pub async fn get_sensors(&self) -> Result<Vec<Sensor>, deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        let statement = client.prepare("SELECT * FROM sensor").await?;
        let rows = client.query(&statement, &[]).await?;
        let mut data = Vec::new();

        for row in rows {
            data.push(Sensor {
                id: row.get(0),
                type_: row.get(1),
                location: row.get(2),
            });
        }
        Ok(data)
    }

    /// Return all ids of sensors from the database
    /// # Arguments
    /// * `self` - The Pool struct
    ///
    /// # Returns
    /// `Result<Vec<String>, tokio_postgres::Error>` - The result of the query
    pub async fn get_sensor_ids(&self) -> Result<Vec<String>, deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        let statement = client.prepare("SELECT id FROM sensor").await?;
        let rows = client.query(&statement, &[]).await?;

        let mut data = Vec::new();

        for row in rows {
            data.push(row.get(0));
        }
        Ok(data)
    }

    /// Insert loudness data into the database
    /// # Arguments
    /// * `self` - The Pool struct
    /// * `sensor_id` - The id of the sensor
    /// * `level` - The sound level
    /// * `time` - The time the data was created
    ///
    /// # Returns
    /// `Result<(), tokio_postgres::Error>` - The result of the query
    pub async fn insert_loudness_data(
        &self,
        sensor_id: &str,
        level: &str,
        time: std::time::SystemTime,
    ) -> Result<(), deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        let statement = client
            .prepare("INSERT INTO loudness (sensor_id, level, time) VALUES ($1, $2, $3)")
            .await?;
        client
            .execute(&statement, &[&sensor_id, &level, &time])
            .await?;
        Ok(())
    }

    /// Insert sensor data into the database
    /// # Arguments
    /// * `self` - The Pool struct
    /// * `sensor_id` - The id of the sensor
    /// * `sensor_type` - The type of the sensor
    /// * `location` - The location of the sensor
    ///
    /// # Returns
    /// `Result<(), tokio_postgres::Error>` - The result of the query
    pub async fn insert_new_sensor(
        &self,
        sensor_id: &str,
        sensor_type: &str,
        sensor_location: &str,
    ) -> Result<(), deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        let statement = client
            .prepare("INSERT INTO sensor (id, type, location) VALUES ($1, $2, $3)")
            .await?;
        client
            .execute(&statement, &[&sensor_id, &sensor_type, &sensor_location])
            .await?;
        Ok(())
    }

    pub async fn create_log_table(&self) -> Result<(), deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "CREATE TABLE IF NOT EXISTS log (
                    id SERIAL PRIMARY KEY,
                    message TEXT NOT NULL,
                    time timestamp NOT NULL);",
                &[],
            )
            .await?;
        Ok(())
    }


    pub async fn get_logs(&self) -> Result<Vec<Log>, deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        let statement = client.prepare("SELECT * FROM log").await?;
        let rows = client.query(&statement, &[]).await?;
        let mut data = Vec::new();

        for row in rows {
            data.push(Log {
                id: row.get(0),
                time: row.get(1),
                message: row.get(2),
            });
        }
    
        Ok(data)
    }

    pub async fn insert_log(
        &self,
        message: &str,
        time: std::time::SystemTime,
    ) -> Result<(), deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        let statement = client
            .prepare("INSERT INTO log (message, time) VALUES ($1, $2)")
            .await?;
        client
            .execute(&statement, &[&message, &time])
            .await?;
        Ok(())
    }
}
