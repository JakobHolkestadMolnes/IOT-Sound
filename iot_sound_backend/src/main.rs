use bytes::Bytes;
use dotenv;
use rumqttc::{AsyncClient, ClientError, MqttOptions, QoS};
use std::env::{self, VarError};
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_postgres::Client;

const MQTT_ID_BACKEND: &str = "g6backend";
const MQTT_TOPIC: &str = "ntnu/+/+/loudness/group06/+";

#[tokio::main]
async fn main() {
    let (mqtt_address, mqtt_port, db_connection_string) = match get_env_variables() {
        Ok((address, port, db_string)) => (
            address,
            match port.parse() {
                Ok(port) => port,
                Err(e) => panic!("Error parsing port: {}", e),
            },
            db_string,
        ),
        Err(e) => panic!("Env variables error: {}", e),
    };

    let (db_client, connection) =
        match tokio_postgres::connect(&db_connection_string, tokio_postgres::NoTls).await {
            Ok((client, connection)) => (client, connection),
            Err(e) => panic!("Postgres connection error: {}", e),
        };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });

    match setup_database(&db_client).await {
        Ok(_) => println!("Database setup successful"),
        Err(e) => panic!("Database setup error: {}", e),
    }

    let (_mqtt_client, eventloop) = match setup_mqtt_client(mqtt_address, mqtt_port).await {
        Ok((client, eventloop)) => (client, eventloop),
        Err(e) => panic!("MQTT setup error: {}", e),
    };

    let (tx, rx) = channel::<(String, Bytes)>(100);

    tokio::join!(
        listen_for_messages(eventloop, tx),
        insert_into_database(db_client, rx)
    );
}

/// Get the environment variables
/// MQTT_ADDRESS, MQTT_PORT, DB_CONNECTION_STRING
fn get_env_variables() -> Result<(String, String, String), VarError> {
    // check if env are set already
    if env::var("MQTT_ADDRESS").is_err()
        || env::var("MQTT_PORT").is_err()
        || env::var("DB_CONNECTION_STRING").is_err()
    {
        println!(
            "\x1b[33m{}\x1b[0m",
            "Environment variables not set. Loading .env file"
        );
        dotenv::dotenv().ok();
    }
    // if any of the env are not set, return early with error
    let mqtt_adress = env::var("MQTT_ADDRESS")?;
    let mqtt_port = env::var("MQTT_PORT")?;
    let db_connection_string = env::var("DB_CONNECTION_STRING")?;
    Ok((mqtt_adress, mqtt_port, db_connection_string))
}

/// Setup the database
/// Creates tables if they don't exist
async fn setup_database(db_client: &Client) -> Result<(), tokio_postgres::Error> {
    // sensor table
    let allowed_sensors =
        "'loudness', 'temperature', 'humidity', 'light', 'air_quality', 'oxygen', 'co2'";
    let create_sensor_table_sql = format!(
        "CREATE TABLE IF NOT EXISTS sensor (
        id text PRIMARY KEY,
        type text NOT NULL CHECK (type IN ({allowed_sensors})),
        location text NOT NULL);"
    );
    let create_table_sensor = db_client.prepare(&create_sensor_table_sql).await?;
    db_client.execute(&create_table_sensor, &[]).await?;

    // loudness table
    let create_loudness_table_sql = "CREATE TABLE IF NOT EXISTS loudness (
    id SERIAL PRIMARY KEY,
    sensor_id text REFERENCES sensor(id),
    level text NOT NULL,
    time timestamp NOT NULL);";
    let create_table_loudness = db_client.prepare(&create_loudness_table_sql).await?;
    db_client.execute(&create_table_loudness, &[]).await?;

    Ok(())
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

async fn listen_for_messages(
    mut eventloop: rumqttc::EventLoop,
    channel: Sender<(String, Bytes)>,
) {
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

async fn insert_into_database(db_client: Client, mut channel: Receiver<(String, Bytes)>) {
    let mut sensors_cache = get_sensors_from_db(&db_client).await.unwrap();


    while let Some(data) = channel.recv().await {
        let topic_split: Vec<&str> = data.0.split('/').collect();
        let sensor_id = topic_split.last().unwrap();
        let payload = std::str::from_utf8(&data.1).unwrap();

        if !sensors_cache.contains(&sensor_id.to_string()) {
            println!("Sensor {} not found in database", sensor_id);
            add_new_sensor(&db_client, &topic_split).await;
            sensors_cache = get_sensors_from_db(&db_client).await.unwrap();
        }

        println!("Sensorid: {} Message: {}",sensor_id, payload);

        let insert_loudness = db_client
            .prepare("INSERT INTO loudness (sensor_id, level, time) VALUES ($1, $2, $3)")
            .await
            .expect("Failed to prepare insert statement");
        let now = std::time::SystemTime::now(); //TODO time should come from sensor
        db_client
            .execute(&insert_loudness, &[&sensor_id, &payload, &now])
            .await
            .expect("Failed to insert loudness into database");
    }
}

async fn get_sensors_from_db(db_client: &Client) -> Result<Vec<String>, tokio_postgres::Error> {
    let get_sensors = db_client
        .prepare("SELECT id FROM sensor")
        .await
        .expect("Failed to prepare get sensors statement");
    let sensors = db_client
        .query(&get_sensors, &[])
        .await
        .expect("Failed to get sensors from database");

    let mut sensors_cache: Vec<String> = Vec::new();
    
    for sensor in sensors.iter() {
        let sensor_id: String = sensor.get(0);
        sensors_cache.push(sensor_id);
    }

    Ok(sensors_cache)
}

async fn add_new_sensor(db_client: &Client, topic_split: &Vec<&str>) {
    let sensor_id = topic_split.last().unwrap();
    let sensor_type = topic_split[3];
    let sensor_location = format!("{}/{}/{}", topic_split[0], topic_split[1], topic_split[2]);

    let insert_sensor = db_client
        .prepare("INSERT INTO sensor (id, type, location) VALUES ($1, $2, $3)")
        .await
        .expect("Failed to prepare insert statement");
    db_client
        .execute(&insert_sensor, &[&sensor_id, &sensor_type, &sensor_location])
        .await
        .expect("Failed to insert sensor into database");
}
