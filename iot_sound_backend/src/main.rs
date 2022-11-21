use bytes::Bytes;
use iot_sound_backend::loudness_data::LoudnessData;
use iot_sound_database::{self, Pool};
use rumqttc::{AsyncClient, ClientError, MqttOptions, QoS};
use std::env::{self, VarError};
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};

const MQTT_ID_BACKEND: &str = "g6backend";
const MQTT_TOPIC: &str = "ntnu/+/+/loudness/group06/+";

#[tokio::main]
async fn main() {
    let (mqtt_address, mqtt_port, db_host, db_port, db_user, db_password, db_name) =
        match get_env_variables() {
            Ok((address, port, db_host, db_port, db_user, db_password, db_name)) => (
                address,
                match port.parse() {
                    Ok(port) => port,
                    Err(e) => panic!("Error parsing mqtt_port: {}", e),
                },
                db_host,
                match db_port.parse() {
                    Ok(port) => port,
                    Err(e) => panic!("Error parsing db_port: {}", e),
                },
                db_user,
                db_password,
                db_name,
            ),
            Err(e) => panic!("Env variables error: {}", e),
        };

    let db_pool = iot_sound_database::Pool::new(
        Some(db_host),
        Some(db_port),
        Some(db_user),
        Some(db_password),
        Some(db_name),
    )
    .await;

    let db_pool = match db_pool {
        Ok(pool) => pool,
        Err(e) => panic!("Error creating database pool: {}", e),
    };

    if let Err(e) = db_pool.create_sensor_table().await {
        panic!("Error creating sensor table: {}", e);
    }
    if let Err(e) = db_pool.create_loudness_table().await {
        panic!("Error creating loudness table: {}", e);
    }

    let (_mqtt_client, eventloop) = match setup_mqtt_client(mqtt_address, mqtt_port).await {
        Ok((client, eventloop)) => (client, eventloop),
        Err(e) => panic!("MQTT setup error: {}", e),
    };

    let (tx, rx) = channel::<(String, Bytes)>(100);

    tokio::join!(
        listen_for_messages(eventloop, tx),
        insert_into_database(db_pool, rx)
    );
}

/// Get the environment variables
/// MQTT_ADDRESS, MQTT_PORT, DB_CONNECTION_STRING
fn get_env_variables() -> Result<(String, String, String, String, String, String, String), VarError>
{
    // check if env are set already
    if env::var("MQTT_ADDRESS").is_err()
        || env::var("MQTT_PORT").is_err()
        || env::var("DB_HOST").is_err()
        || env::var("DB_PORT").is_err()
        || env::var("DB_USER").is_err()
        || env::var("DB_PASSWORD").is_err()
        || env::var("DB_NAME").is_err()
    {
        println!("\x1b[33mEnvironment variables not set. Loading .env file\x1b[0m");
        dotenv::dotenv().ok();
    }
    // if any of the env are not set, return early with error
    let mqtt_adress = env::var("MQTT_ADDRESS")?;
    let mqtt_port = env::var("MQTT_PORT")?;
    let db_host = env::var("DB_HOST")?;
    let db_port = env::var("DB_PORT")?;
    let db_user = env::var("DB_USER")?;
    let db_password = env::var("DB_PASSWORD")?;
    let db_name = env::var("DB_NAME")?;
    Ok((
        mqtt_adress,
        mqtt_port,
        db_host,
        db_port,
        db_user,
        db_password,
        db_name,
    ))
}

async fn setup_mqtt_client(
    mqtt_adress: String,
    mqtt_port: u16,
) -> Result<(AsyncClient, rumqttc::EventLoop), ClientError> {
    let mut mqtt_options = MqttOptions::new(MQTT_ID_BACKEND, mqtt_adress, mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    let (mqtt_client, eventloop) = AsyncClient::new(mqtt_options, 10);

    mqtt_client.subscribe(MQTT_TOPIC, QoS::AtLeastOnce).await?;
    Ok((mqtt_client, eventloop))
}

async fn listen_for_messages(mut eventloop: rumqttc::EventLoop, channel: Sender<(String, Bytes)>) {
    loop {
        match eventloop.poll().await {
            Ok(rumqttc::Event::Incoming(incoming)) => {
                if let rumqttc::Incoming::Publish(publish) = incoming {
                    channel
                        .send((publish.topic, publish.payload))
                        .await
                        .unwrap();
                }
            }
            Ok(_) => {}
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}

async fn insert_into_database(db_pool: Pool, mut channel: Receiver<(String, Bytes)>) {
    let mut sensors_cache = match db_pool.get_sensor_ids().await {
        Ok(sensors) => sensors,
        Err(e) => panic!("Error getting sensor ids from db: {}", e),
    };

    while let Some(data) = channel.recv().await {
        let topic_split: Vec<&str> = data.0.split('/').collect();
        let sensor_id = topic_split.last().unwrap();

        let data: &str = match std::str::from_utf8(&data.1) {
            Ok(data) => data,
            Err(e) => {
                println!("Error converting bytes to string: {}", e);
                continue;
            }
        };

        let payload = match LoudnessData::parse_csv(data) {
            Ok(payload) => payload,
            Err(e) => {
                println!("Error parsing payload: {}", e);
                continue;
            }
        };

        if !sensors_cache.contains(&sensor_id.to_string()) {
            println!("Sensor {} not found in database", sensor_id);
            add_new_sensor(&db_pool, &topic_split).await;
            sensors_cache = db_pool
                .get_sensor_ids()
                .await
                .expect("Sensor ids should be in db");
        }

        println!("Sensorid: {} Message: {}", sensor_id, payload.db_level());
        db_pool
            .insert_loudness_data(
                sensor_id,
                &format!("{}", payload.db_level()),
                payload.timestamp(),
            )
            .await
            .expect("Inserting loudness into db should work");
    }
}

async fn add_new_sensor(db_pool: &Pool, topic_split: &[&str]) {
    let sensor_id = topic_split.last().unwrap();
    let sensor_type = topic_split[3];
    let sensor_location = format!("{}/{}/{}", topic_split[0], topic_split[1], topic_split[2]);

    db_pool
        .insert_new_sensor(sensor_id, sensor_type, &sensor_location)
        .await
        .expect("Inserting new sensor into db should work");
}
