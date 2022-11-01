use std::env;

use paho_mqtt as mqtt;
use dotenv;

fn main() {
    

    println!("Hello, world!");

    // add waitgroup
    let wg = crossbeam::sync::WaitGroup::new();

    // create task

    {
        let wg = wg.clone();

        std::thread::spawn(move || {
            listen_for_mqtt_messages();
            drop(wg);
        });
    }

    wg.wait();

    println!("Goodbye, world!");
}


fn listen_for_mqtt_messages() {

    // load env variables from .env file, place .env file in project root
    dotenv::dotenv().ok();
    let qmtt_adress = env::var("MQTT_ADRESS").expect("MQTT_ADRESS must be set in .env file");
    let mqtt_port = env::var("MQTT_PORT").expect("MQTT_PORT must be set");

    //create a link to the mqtt broker example: "tcp://env_var:1883"
    let mqtt_broker = format!("tcp://{}:{}", qmtt_adress, mqtt_port);
    let cli = mqtt::Client::new(mqtt_broker).unwrap_or_else(|err| {
        panic!("{}", err);
    });
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
         .keep_alive_interval(std::time::Duration::from_secs(20))
         .clean_session(true)
         .finalize();

    println!("Connecting to the MQTT server...");
    if let Err(e) = cli.connect(conn_opts) {
        println!("Error connecting to the MQTT server:\n\t{:?}", e);
        return;
    }

    println!("Subscribing to the topic...");
   let rx = cli.start_consuming();
    let sub = cli.subscribe("test", 1);
    if let Err(e) = sub {
        println!("Error subscribing to the topic:\n\t{:?}", e);
        return;
    }

    println!("Waiting for messages...");
    for m in rx.iter() {
        if let Some(msg) = m {
            //parse message payload
            let payload = String::from_utf8_lossy(msg.payload());
            println!("Received message:\n\t{:?}", payload);
        }
    }
}