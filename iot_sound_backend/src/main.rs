use std::env;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use dotenv;

#[tokio::main]
async fn main () {
    dotenv::dotenv().ok();
    let mqtt_adress = env::var("MQTT_ADRESS").expect("MQTT_ADRESS must be set in .env file");
    let mqtt_port = env::var("MQTT_PORT").expect("MQTT_PORT must be set");

   listen_for_message(mqtt_adress, mqtt_port).await;

}

/// Async function to listen for messages on the MQTT broker
/// * `mqtt_adress`: String
/// * `mqtt_port`: String
async fn listen_for_message(mqtt_adress: String, mqtt_port: String) {
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