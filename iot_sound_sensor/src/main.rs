use dotenv;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use std::{env, time::Duration};
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[tokio::main]
async fn main() {
    println!("Sensor node started");

    if env::var("MQTT_HOST").is_err()
        || env::var("MQTT_PORT").is_err()
        || env::var("MQTT_CLIENT_ID").is_err()
    {
        println!(
            "\x1b[33m{}\x1b[0m",
            "Environment variables not set. Loading .env file"
        );
        dotenv::dotenv().ok();
    }

    let (client, eventloop) = setup_mqtt_client();

    let (tx, rx) = channel::<Message>(100);
    tokio::join!(
        keep_mqtt_client_alive(eventloop),
        send_mqtt_messages(client, rx),
        message_generator(tx)
    );
}

struct Message {
    payload: Vec<u8>,
}
impl Message {
    fn payload_from_string(payload: String) -> Self {
        Message {
            payload: payload.bytes().collect(),
        }
    }
    fn _payload_from_str_slice(payload: &str) -> Self {
        Message {
            payload: payload.bytes().collect(),
        }
    }
}

/// Generates messages and sends them to the mqtt client
///
/// * `channel` - The channel to send the messages to
async fn message_generator(channel: Sender<Message>) {
    let mut i = 0;
    loop {
        match channel
            .send(Message::payload_from_string(format!(
                "This is sensor data {i}"
            )))
            .await
        {
            Ok(_) => {
                println!("message sent to client: {i}");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                println!("Failed to send message: {}", e);
            }
        };
        i += 1;
    }
}

/// Listens for messages on the channel and publishes them to the mqtt client
///
/// * `channel` - The channel to listen for messages on
async fn send_mqtt_messages(client: AsyncClient, mut channel: Receiver<Message>) {
    let mqtt_client_id = env::var("MQTT_CLIENT_ID").expect("MQTT_CLIENT_ID must be set");
    let topic = format!(
        "ntnu/ankeret/biblioteket/loudness/group06/{}",
        mqtt_client_id
    );

    while let Some(message) = channel.recv().await {
        client
            .publish(&topic, QoS::ExactlyOnce, false, message.payload)
            .await
            .unwrap();
    }
}

/// Sets up the mqtt client
///
///
fn setup_mqtt_client() -> (AsyncClient, EventLoop) {
    // Load environment variables
    let qmtt_adress = env::var("MQTT_ADRESS").expect("MQTT_ADRESS must be set in .env file");
    let mqtt_port = env::var("MQTT_PORT").expect("MQTT_PORT must be set");
    let mqtt_port = mqtt_port
        .parse::<u16>()
        .expect("MQTT_PORT must be a number");
    let mqtt_client_id = env::var("MQTT_CLIENT_ID").expect("MQTT_CLIENT_ID must be set");

    // Setup mqtt client
    let mut mqttoptions = MqttOptions::new(mqtt_client_id, qmtt_adress, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
    (client, eventloop)
}

/// Keeps the mqtt client running
///
/// * `eventloop` - The eventloop of the mqtt client
async fn keep_mqtt_client_alive(mut eventloop: EventLoop) {
    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                println!("Notification: = {:?}", notification);
            }
            Err(e) => {
                println!("Failed to poll: {}", e);
            }
        }
    }
}
