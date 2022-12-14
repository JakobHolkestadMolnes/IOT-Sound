use bytes::Bytes;
use iot_sound_backend::loudness_data::LoudnessData;
use iot_sound_database::{self, Pool};
use rumqttc::{AsyncClient, ClientError, MqttOptions, QoS};
use std::env::{self};
use std::error::Error;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;

const MQTT_TOPIC: &str = "ntnu/+/+/+/group06/+";

#[tokio::main]
async fn main() {
    let env_vars = match get_env_variables() {
        Ok(env_vars) => env_vars,
        Err(e) => panic!("Error getting env variables: {}", e),
    };

    let db_pool = match iot_sound_database::Pool::new(
        Some(env_vars.db_host),
        Some(env_vars.db_port),
        Some(env_vars.db_user),
        Some(env_vars.db_password),
        Some(env_vars.db_name),
    )
    .await
    {
        Ok(pool) => pool,
        Err(e) => panic!("Error creating database pool: {}", e),
    };

    if let Err(e) = db_pool.create_sensor_table().await {
        panic!("Error creating sensor table: {}", e);
    }
    if let Err(e) = db_pool.create_loudness_table().await {
        panic!("Error creating loudness table: {}", e);
    }
    if let Err(e) = db_pool.create_log_table().await {
        panic!("Error creating log table: {}", e);
    }

    let (_mqtt_client, eventloop) =
        match setup_mqtt_client(env_vars.mqtt_address, env_vars.mqtt_port).await {
            Ok((client, eventloop)) => (client, eventloop),
            Err(e) => panic!("MQTT setup error: {}", e),
        };

    let (tx, rx) = channel::<(String, Bytes)>(100);

    println!("Backend started...");
    println!("Listening to MQTT topic: {}", MQTT_TOPIC);
    tokio::join!(
        listen_for_messages(eventloop, db_pool.clone(), tx),
        insert_into_database(db_pool.clone(), rx)
    );
}

struct EnvVars {
    mqtt_address: String,
    mqtt_port: u16,
    db_host: String,
    db_port: u16,
    db_user: String,
    db_password: String,
    db_name: String,
}

/// Get the environment variables
/// MQTT_ADDRESS, MQTT_PORT, DB_CONNECTION_STRING
fn get_env_variables() -> Result<EnvVars, Box<dyn Error>> {
    // check if env are set already
    if env::var("MQTT_ADDRESS").is_err()
        || env::var("MQTT_PORT").is_err()
        || env::var("DB_HOST").is_err()
        || env::var("DB_PORT").is_err()
        || env::var("DB_USER").is_err()
        || env::var("DB_PASSWORD").is_err()
        || env::var("DB_NAME").is_err()
    {
        println!("Environment variables not set. Loading .env file");
        dotenv::dotenv().ok();
    }
    // if any of the env are not set, return early with error
    let mqtt_address = env::var("MQTT_ADDRESS")?;
    let mqtt_port = env::var("MQTT_PORT")?;
    let db_host = env::var("DB_HOST")?;
    let db_port = env::var("DB_PORT")?;
    let db_user = env::var("DB_USER")?;
    let db_password = env::var("DB_PASSWORD")?;
    let db_name = env::var("DB_NAME")?;

    let mqtt_port = mqtt_port.parse::<u16>()?;
    let db_port = db_port.parse::<u16>()?;
    Ok(EnvVars {
        mqtt_address,
        mqtt_port,
        db_host,
        db_port,
        db_user,
        db_password,
        db_name,
    })
}

/// Setup the MQTT client
/// Returns a Result with the client and eventloop
/// # Arguments
/// * `mqtt_address` - The address of the MQTT broker
/// * `mqtt_port` - The port of the MQTT broker
/// # Returns
/// * `Result<(AsyncClient, EventLoop), ClientError>` - The client and eventloop
async fn setup_mqtt_client(
    mqtt_adress: String,
    mqtt_port: u16,
) -> Result<(AsyncClient, rumqttc::EventLoop), ClientError> {
    let mut mqtt_options =
        MqttOptions::new(Uuid::new_v4().to_string().as_str(), mqtt_adress, mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    let (mqtt_client, eventloop) = AsyncClient::new(mqtt_options, 10);

    mqtt_client.subscribe(MQTT_TOPIC, QoS::AtLeastOnce).await?;
    Ok((mqtt_client, eventloop))
}

/// Function that listens for messages from the MQTT broker
/// 
/// # Arguments
/// mut `eventloop` - The eventloop that listens for messages
/// `db_pool` - The database pool for logging purposes
/// `channel` - The channel to send the messages to
async fn listen_for_messages(
    mut eventloop: rumqttc::EventLoop,
    db_pool: Pool,
    channel: Sender<(String, Bytes)>,
) {
    loop {
        match eventloop.poll().await {
            Ok(rumqttc::Event::Incoming(incoming)) => {
                if let rumqttc::Incoming::Publish(publish) = incoming {
                    if let Err(e) = channel.send((publish.topic, publish.payload)).await {
                        println!("Error sending message to channel: {}", e);
                        if let Err(e) = db_pool
                            .insert_log(
                                &format!("Error sending recieved message to db writer: {:?}", e),
                                SystemTime::now(),
                            )
                            .await
                        {
                            eprintln!("Error inserting log into database: {}", e);
                            continue;
                        }
                    };
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("MQTT Connection error: {:?}", &e);
                if let Err(e) = db_pool
                    .insert_log(
                        &format!("Mqtt Connection error: {:?}", e),
                        SystemTime::now(),
                    )
                    .await
                {
                    eprintln!("Error inserting log into database: {}", e);
                    continue;
                }
            }
        }
    }
}

/// Function that inserts the messages into the database
/// 
/// # Arguments
/// * `db_pool` - The database pool
/// * `channel` - The channel to listen for messages on
async fn insert_into_database(db_pool: Pool, mut channel: Receiver<(String, Bytes)>) {
    let mut sensors_cache = match db_pool.get_sensor_ids().await {
        Ok(sensors) => sensors,
        Err(e) => panic!("Error getting sensor ids from db: {}", e),
    };

    while let Some(data) = channel.recv().await {
        let topic_split: Vec<&str> = data.0.split('/').collect();
        let sensor_id = topic_split.last().expect("Subscribed topic is valid");

        let data: &str = match std::str::from_utf8(&data.1) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error converting bytes to string: {}", e);
                if let Err(e) = db_pool
                    .insert_log(
                        &format!("Error converting bytes to string: {:?}", e),
                        SystemTime::now(),
                    )
                    .await
                {
                    eprintln!("Error inserting log into database: {}", e);
                }
                continue;
            }
        };

        let payload = match LoudnessData::parse_csv(data) {
            Ok(payload) => payload,
            Err(e) => {
                eprintln!("Error parsing payload: {}", e);
                if let Err(e) = db_pool
                    .insert_log(
                        &format!("Error parsing payload: {:?}", e),
                        SystemTime::now(),
                    )
                    .await
                {
                    eprintln!("Error inserting log into database: {}", e);
                }
                continue;
            }
        };

        if !sensors_cache.contains(&sensor_id.to_string()) {
            println!("Sensor {} not found in database", sensor_id);
            if let Err(e) = db_pool
                .insert_log(
                    &format!("Sensor {} not found in database, adding...", sensor_id),
                    SystemTime::now(),
                )
                .await
            {
                eprintln!("Error inserting log into database: {}", e);
                continue;
            }
            if let Err(e) = add_new_sensor(&db_pool, &topic_split).await {
                eprintln!("Error adding new sensor: {}", e);
                if let Err(e) = db_pool
                    .insert_log(
                        &format!("Error adding new sensor: {:?}", e),
                        SystemTime::now(),
                    )
                    .await
                {
                    eprintln!("Error inserting log into database: {}", e);
                }
                continue;
            };
            sensors_cache = match db_pool.get_sensor_ids().await {
                Ok(sensors) => sensors,
                Err(e) => {
                    eprintln!("Error getting sensor ids from db: {}", e);
                    continue;
                }
            };
        }

        println!("Sensorid: {} Message: {}", sensor_id, payload.db_level());
        if let Err(e) = db_pool
            .insert_loudness_data(
                sensor_id,
                &format!("{}", payload.db_level()),
                payload.timestamp(),
            )
            .await
        {
            eprintln!("Error inserting loudness data into database: {}", e);
        }
    }
}

async fn add_new_sensor(db_pool: &Pool, topic_split: &[&str]) -> Result<(), Box<dyn Error>> {
    let sensor_id = topic_split.last().unwrap();
    let sensor_type = topic_split[3];
    let sensor_location = format!("{}/{}/{}", topic_split[0], topic_split[1], topic_split[2]);

    if sensor_type == "loudness" {
        if let Err(e) = db_pool
            .insert_new_sensor(sensor_id, sensor_type, &sensor_location)
            .await
        {
            eprintln!("Error inserting new sensor into database: {}", e);
        }
        Ok(())
    } else {
        Err((format!("Sensor type {} not supported", sensor_type)).into())
    }
}
