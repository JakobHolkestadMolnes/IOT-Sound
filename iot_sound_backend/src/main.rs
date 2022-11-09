use std::env;

use dotenv;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio_postgres;

#[tokio::main]
async fn main() {
    // check if env are set already
    if env::var("MQTT_ADRESS").is_err()
        || env::var("MQTT_PORT").is_err()
        || env::var("DB_CONNECTION_STRING").is_err()
    {
        println!(
            "\x1b[33m{}\x1b[0m",
            "Environment variables not set. Loading .env file"
        );
        dotenv::dotenv().ok();
    }

    let mqtt_adress = env::var("MQTT_ADRESS").expect("MQTT_ADRESS must be set in .env file");
    let mqtt_port = env::var("MQTT_PORT").expect("MQTT_PORT must be set");

    let db_connection_string =
        env::var("DB_CONNECTION_STRING").expect("DB_CONNECTION_STRING must be set");

    let (client, connection) =
        tokio_postgres::connect(&db_connection_string, tokio_postgres::NoTls)
            .await
            .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    //if no tables, create them
    let create_table_sensor = client.prepare(
        "CREATE TABLE IF NOT EXISTS sensor(
            id SERIAL PRIMARY KEY,
            name text NOT NULL,
            type text NOT NULL CHECK (type IN ('loudness', 'temperature', 'humidity', 'light', 'air_quality', 'oxygen', 'co2')),
            location text NOT NULL);"
        ).await.unwrap();

    let create_table_loudness = client
        .prepare(
            "CREATE TABLE IF NOT EXISTS loudness (
            id SERIAL PRIMARY KEY,
            sensor_id int REFERENCES sensor(id),
            level text NOT NULL,
            time timestamp NOT NULL);",
        )
        .await
        .unwrap();

    client.execute(&create_table_sensor, &[]).await.unwrap();
    client.execute(&create_table_loudness, &[]).await.unwrap();

    listen_for_message(mqtt_adress, mqtt_port, client).await;
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
        .subscribe(topic, QoS::AtLeastOnce)
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

                    let sensor_id = String::from("1"); //TODO get sensor id from payload

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
