use paho_mqtt as mqtt;

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
   let cli = mqtt::Client::new("tcp://\"ip adress here\"").unwrap();
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