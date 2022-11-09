use std::env::{self, VarError};

use dotenv;
use rumqttc::{AsyncClient, MqttOptions, QoS};
//use tokio_postgres;
use tokio_postgres::Client;

#[tokio::main]
async fn main() {
    let (mqtt_address, mqtt_port, db_connection_string) = match get_env_variables() {
        Ok((address, port, db_string)) => (address, port, db_string),
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

    listen_for_message(mqtt_address, mqtt_port, db_client).await;
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
        name text NOT NULL,
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

/// Async function to listen for messages on the MQTT broker
/// * `mqtt_adress`: String
/// * `mqtt_port`: String
async fn listen_for_message(
    mqtt_adress: String,
    mqtt_port: String,
    database_connection: tokio_postgres::Client,
) {
    let mqtt_options = MqttOptions::new("sensor_node", mqtt_adress, mqtt_port.parse().unwrap());

    let mqtt_client_id = env::var("MQTT_CLIENT_ID").expect("MQTT_CLIENT_ID must be set");
    let topic = format!(
        "ntnu/ankeret/biblioteket/loudness/group06/{}",
        mqtt_client_id
    );


    let (mqtt_client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    mqtt_client
        .subscribe("ntnu/+/+/loudness/group06/#", QoS::AtLeastOnce)
        .await
        .expect("Failed to subscribe to topic");

    let mut message_count = 0;
    loop {
        let notification = eventloop.poll().await;
        match notification {
            Ok(rumqttc::Event::Incoming(incoming)) => match incoming {
                rumqttc::Incoming::Publish(publish) => {
                    println!(
                        "Received message: {:?}",
                        std::str::from_utf8(&publish.payload)
                            .unwrap_or("Failed to convert message to string")
                    );
                    println!("Message count: {}", message_count);
                    message_count += 1;

                    // convert payload to string
                    let payload = std::str::from_utf8(&publish.payload).unwrap();

                    let topic_split: Vec<&str> = publish.topic.split('/').collect();
                    let sensor_id = topic_split.last().unwrap();
                    println!("Sender: {}", sensor_id);

                    let insert_loudness = database_connection
                        .prepare(
                            "INSERT INTO loudness (sensor_id, level, time) VALUES ($1, $2, $3)",
                        )
                        .await
                        .expect("Failed to prepare insert statement");

                    // get current time as timestamp
                    let now = std::time::SystemTime::now(); //TODO time should come from sensor
                    database_connection
                        .execute(&insert_loudness, &[&sensor_id, &payload, &now])
                        .await
                        .expect("Failed to insert loudness into database");
                }
                _ => {}
            },
            Ok(_) => {}
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}
