use dotenv;
use json;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use std::{env, time::Duration};
use tokio::sync::mpsc::{channel, Receiver, Sender};

// mqtt topic for this sensor to publish to
const MQTT_TOPIC: &str = "ntnu/ankeret/biblioteket/loudness/group06/";

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    println!("Sensor node started");

    // get env variables
    let (mqtt_address, mqtt_port, mqtt_client_id) = match get_env_variables() {
        Ok((address, port, client_id)) => (
            address,
            match port.parse() {
                Ok(port) => port,
                Err(e) => panic!("Error parsing port: {}", e),
            },
            client_id,
        ),
        Err(e) => panic!("Error getting env variables: {}", e),
    };

    let (client, eventloop) = setup_mqtt_client(&mqtt_address, mqtt_port, &mqtt_client_id);

    let (tx, rx) = channel::<Message>(100);
    let err = tokio::try_join!(
        keep_mqtt_client_alive(eventloop),
        send_mqtt_messages(client, &mqtt_client_id, rx),
        message_generator(tx)
    );

    if let Err(e) = err {
        println!("Error: {}", e);
        return Err(e);
    }

    Ok(())
}

/// Get env variables
///
/// `MQTT_ADDRESS`, `MQTT_PORT`, `MQTT_CLIENT_ID`
fn get_env_variables() -> Result<(String, String, String), env::VarError> {
    if env::var("MQTT_ADDRESS").is_err()
        || env::var("MQTT_PORT").is_err()
        || env::var("MQTT_CLIENT_ID").is_err()
    {
        dotenv::dotenv().ok();
    }
    let mqtt_adress = env::var("MQTT_ADDRESS")?;
    let mqtt_port = env::var("MQTT_PORT")?;
    let mqtt_client_id = env::var("MQTT_CLIENT_ID")?;
    Ok((mqtt_adress, mqtt_port, mqtt_client_id))
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

struct JsonDblevel {
    db_level: f64,
}

impl JsonDblevel {
    fn new(db_level: f64) -> Self {
        JsonDblevel { db_level }
    }

    fn to_string(&self) -> String {
        json::from(self.db_level).to_string()
    }
}

/// Generates messages and sends them to the mqtt client
///
/// * `channel` - The channel to send the messages to
async fn message_generator(channel: Sender<Message>) -> Result<(), std::io::Error> {
    let mut i = 0;
    loop {
        let message = JsonDblevel::new(i as f64).to_string();
        match channel.send(Message::payload_from_string(message)).await {
            Ok(_) => {
                println!("message sent to client: {i}");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                println!("Failed to send message: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to send message",
                ));
            }
        };
        i += 1;
    }
}

/// Listens for messages on the channel and publishes them to the mqtt client
///
/// * `client` - The mqtt client
/// * `client_id` - Mqtt client id for this device
/// * `channel` - The channel to listen for messages on
async fn send_mqtt_messages(
    client: AsyncClient,
    client_id: &str,
    mut channel: Receiver<Message>,
) -> Result<(), std::io::Error> {
    let topic = format!("{MQTT_TOPIC}{client_id}");

    while let Some(message) = channel.recv().await {
        client
            .publish(&topic, QoS::ExactlyOnce, false, message.payload)
            .await
            .expect("Failed to publish message");
    }
    Ok(())
}

/// Sets up the mqtt client
///
///
fn setup_mqtt_client(
    mqtt_address: &str,
    mqtt_port: u16,
    mqtt_client_id: &str,
) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new(mqtt_client_id, mqtt_address, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
    (client, eventloop)
}

/// Keeps the mqtt client running
///
/// * `eventloop` - The eventloop of the mqtt client
async fn keep_mqtt_client_alive(mut eventloop: EventLoop) -> Result<(), std::io::Error> {
    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                println!("Notification: = {:?}", notification);
            }
            Err(e) => {
                println!("Failed to poll: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to poll",
                ));
            }
        }
    }
}
