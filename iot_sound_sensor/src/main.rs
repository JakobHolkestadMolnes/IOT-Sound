mod loudness_sensor_simulator;

use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use std::{env, error::Error, time::Duration};
use tokio::sync::mpsc::{channel, Receiver, Sender};

// mqtt topic for this sensor to publish to
const MQTT_TOPIC: &str = "ntnu/ankeret/biblioteket/loudness/group06/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
        message_generator(tx),
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
#[derive(Debug)]
struct Message {
    payload: Vec<u8>,
}
impl Message {
    fn _payload_from_str_slice(payload: &str) -> Self {
        Message {
            payload: payload.bytes().collect(),
        }
    }
}

/// Generates messages and sends them to the mqtt client
///
/// * `channel` - The channel to send the messages to
async fn message_generator(channel: Sender<Message>) -> Result<(), Box<dyn Error>> {
    let mut loudness_sensor_simulator = loudness_sensor_simulator::LoudnessSensorSimulator::new();
    loop {
        let loudness = loudness_sensor_simulator.get_loudness_data();
        let message = loudness.to_csv();

        match channel
            .send(Message::_payload_from_str_slice(&message))
            .await
        {
            Ok(_) => {
                println!("Message sent to mqtt publisher: {}", &message);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) => {
                println!("Failed to send message to publisher: {}", e);
                return Err(Box::new(e));
            }
        };
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
) -> Result<(), Box<dyn Error>> {
    let topic = format!("{MQTT_TOPIC}{client_id}");
    while let Some(message) = channel.recv().await {
        if let Err(e) = client
            .publish(&topic, QoS::ExactlyOnce, false, message.payload)
            .await
        {
            println!("Failed to publish: {}", e);
            return Err(Box::new(e));
        }
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
async fn keep_mqtt_client_alive(mut eventloop: EventLoop) -> Result<(), Box<dyn Error>> {
    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                println!("Notification: = {:?}", notification);
            }
            Err(e) => {
                println!("Failed to poll: {}", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}
