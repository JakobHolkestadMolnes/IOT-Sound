use std::env;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use dotenv;
use tokio_postgres;

#[tokio::main]
async fn main () {

    // check if env are set already
    if env :: var ( "MQTT_ADRESS" ). is_err ()  || env :: var ( "MQTT_PORT" ). is_err () || env :: var ( "DB_CONNECTION_STRING" ). is_err () {
        println! ( "\x1b[33m{}" , "Environment variables not set. Loading .env file" );
        dotenv::dotenv().ok();
    }


    let mqtt_adress = env::var("MQTT_ADRESS").expect("MQTT_ADRESS must be set in .env file");
    let mqtt_port = env::var("MQTT_PORT").expect("MQTT_PORT must be set");

    let db_connection_string = env::var("DB_CONNECTION_STRING").expect("DB_CONNECTION_STRING must be set");

    let (client, connection) = tokio_postgres::connect(&db_connection_string, tokio_postgres::NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

   //if not tables create one
   let create_table = client.prepare("CREATE TABLE IF NOT EXISTS sound (id SERIAL PRIMARY KEY, sound VARCHAR(255), time TIMESTAMP)").await.unwrap();
    client.execute(&create_table, &[]).await.unwrap();

   listen_for_message(mqtt_adress, mqtt_port, client).await;

}



/// Async function to listen for messages on the MQTT broker
/// * `mqtt_adress`: String
/// * `mqtt_port`: String
async fn listen_for_message(mqtt_adress: String, mqtt_port: String, database_connection: tokio_postgres::Client) {
    let mqtt_options = MqttOptions::new("sensor_node", mqtt_adress, mqtt_port.parse().unwrap());
    let (mqtt_client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    mqtt_client.subscribe("g6/sensor", QoS::AtLeastOnce).await
    .expect("Failed to subscribe to topic");

 

    let mut message_count = 0;
    loop {
        let notification = eventloop.poll().await;
        match notification {
            Ok(rumqttc::Event::Incoming(incoming)) => match incoming {
                rumqttc::Incoming::Publish(publish) => {
                    println!(
                        "Received message: {:?}",
                        std::str::from_utf8(&publish.payload).unwrap()
                    );
                    println!("Message count: {}", message_count);
                    message_count += 1;

                    // convert payload to string
                    let payload = std::str::from_utf8(&publish.payload).unwrap();

                    let insert_sound = database_connection.prepare("INSERT INTO sound (sound, time) VALUES ($1, $2)").await
                    .expect("Failed to prepare insert statement");

                    // get current time as timestamp
                    let now = std::time::SystemTime::now();
                    database_connection.execute(&insert_sound, &[&payload, &now]).await
                    .expect("Failed to insert sound into database");
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