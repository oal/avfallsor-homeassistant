use std::thread::sleep;
use std::time::Duration;
use rumqttc::MqttOptions;
use rumqttc::Event::Incoming;
use rumqttc::PubAck;
use serde::{Deserialize, Serialize};
use crate::web::{get_address_page, get_next_pickups};
use dotenvy::dotenv;
use std::env;
use crate::pickup::Pickup;

mod web;
mod homeassistant;
mod pickup;

const INTEGRATION_NAME: &str = "Avfall Sør";
const INTEGRATION_IDENTIFIER: &str = "avfallsor";

fn get_pickups(address: &str) -> anyhow::Result<Vec<Pickup>> {
    let client = reqwest::blocking::Client::new();
    let address_info = get_address_page(&client, &address);

    let url = address_info?.href;
    get_next_pickups(&client, &url)
}

fn wait_for_acks(connection: &mut rumqttc::Connection, num_packets: usize) {
    let mut packets_received = 0;
    for (_, notification) in connection.iter().enumerate() {
        match notification {
            Ok(Incoming(rumqttc::Packet::PubAck(PubAck { pkid: _pkid }))) => {
                packets_received += 1;
                if packets_received == num_packets {
                    break;
                }
            }
            _ => {}
        }
    }
}

struct AppConfig {
    address: String,
    mqtt_host: String,
    mqtt_port: u16,
}

impl AppConfig {
    fn from_env() -> AppConfig {
        dotenv().expect(".env file not found");

        let address = env::var("ADDRESS").expect("ADDRESS not set");

        let mqtt_host = env::var("MQTT_HOST").expect("MQTT_HOST not set");
        let mqtt_port = env::var("MQTT_PORT").or::<()>(Ok("1883".to_string()));
        let mqtt_port = mqtt_port.unwrap().parse::<u16>().expect("MQTT_PORT is not a valid port");

        AppConfig {
            address,
            mqtt_host,
            mqtt_port,
        }
    }
}

fn main() {
    dotenv().expect(".env file not found");

    let config = AppConfig::from_env();

    let mut mqtt_options = MqttOptions::new(INTEGRATION_IDENTIFIER, config.mqtt_host, config.mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = rumqttc::Client::new(mqtt_options, 10);

    let pickups = get_pickups(&config.address).expect("Failed to get pickups");
    let num_pickups = pickups.len();

    // Register sensors.
    for pickup in &pickups {
        let config_message = pickup.homeassistant_config_message();
        let config_serialized = serde_json::to_string(&config_message).unwrap();
        client.publish(pickup.config_topic(), rumqttc::QoS::AtLeastOnce, false, config_serialized).expect("Failed to publish config message");
    }
    wait_for_acks(&mut connection, num_pickups);
    println!("Registered {} sensors", num_pickups);

    // Give HomeAssistant time to save the provided sensors.
    sleep(Duration::from_secs(2));

    // Publish sensor values.
    for pickup in &pickups {
        let date = pickup.homeassistant_state_message();
        client.publish(pickup.state_topic(), rumqttc::QoS::AtLeastOnce, false, date).expect("Failed to publish state message");
    }
    wait_for_acks(&mut connection, num_pickups);
    println!("Published {} sensor values", num_pickups);

    client.disconnect().unwrap();
}
