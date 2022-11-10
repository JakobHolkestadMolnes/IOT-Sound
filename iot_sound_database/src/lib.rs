use deadpool_postgres;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};


#[derive(Clone)]
pub struct Pool {
    pool: deadpool_postgres::Pool,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    id: i32,
    sound: String,
    time: std::time::SystemTime,
}

// implement a trait for vec of data
impl Data {
    pub fn new(id: i32, sound: String, time: std::time::SystemTime) -> Data {
        Data {
            id,
            sound,
            time,
        }
    }
}

// implement a trait for vec of data
impl Into<serde_json::Value> for Data {
    fn into(self) -> serde_json::Value {
        json!({
            "id": self.id,
            "sound": self.sound,
            "time": self.time,
        })
    }
}



impl Pool {

    pub async fn new(host: Option<String>, port: Option<u16>, user: Option<String>, password: Option<String>, dbname: Option<String>) -> Pool {
        let config = deadpool_postgres::Config {
            user: user,
            password: password,
            host: host,
            port: port,
            dbname: dbname,
            ..Default::default()
        };
        let pool = config.create_pool(None, tokio_postgres::NoTls).unwrap();
        Pool { pool}
    }

  pub  async fn create_sensor_table(&self) -> Result<(), deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "CREATE TABLE IF NOT EXISTS sensor (
                    id text PRIMARY KEY,
                    name text NOT NULL,
                    type text NOT NULL CHECK (type IN ({allowed_sensors})),
                    location text NOT NULL);",
                &[],
            )
            .await?;
        Ok(())
    }

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

    pub async fn get_loudness(&self) -> Result<Vec<Data>, deadpool_postgres::PoolError> {
        let client = self.pool.get().await?;
        let statement = client.prepare("SELECT * FROM loudness").await?;
        let rows = client.query(&statement, &[]).await?;
        let mut data = Vec::new();
        for row in rows {
            data.push(Data {
                id: row.get(0),
                sound: row.get(1),
                time: row.get(2),
            });
        }
        Ok(data)
    }
}

