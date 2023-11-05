use std::fmt;
use std::thread::sleep;
use std::time::Duration;
use rumqttc::MqttOptions;
use rumqttc::Event::Incoming;
use rumqttc::PubAck;
use serde::{Deserialize, Serialize};
use crate::homeassistant::{ConfigMessage, Device};
use crate::web::{get_address_page, get_next_pickups};
use dotenvy::dotenv;
use std::env;

mod web;
mod homeassistant;

const INTEGRATION_NAME: &str = "Avfall Sør";
const INTEGRATION_IDENTIFIER: &str = "avfallsor";


#[derive(Debug)]
enum PickupType {
    Garbage,
    PaperPlastic,
    GlassMetal,
    FoodWaste,
}

impl PickupType {
    fn from_str(s: &str) -> Option<PickupType> {
        match s {
            "Restavfall" => Some(PickupType::Garbage),
            "Papp, papir og plastemballasje" => Some(PickupType::PaperPlastic),
            "Glass- og metallemballasje" => Some(PickupType::GlassMetal),
            "Bioavfall" => Some(PickupType::FoodWaste),
            _ => None
        }
    }
}

impl fmt::Display for PickupType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct Pickup {
    date: chrono::NaiveDate,
    label: String,
    kind: PickupType,
}

impl Pickup {
    fn identifier(&self) -> String {
        let kind = self.kind.to_string().to_lowercase();
        format!("{}-{}", INTEGRATION_IDENTIFIER, kind)
    }

    fn topic_prefix(&self) -> String {
        let identifier = self.identifier();
        format!("homeassistant/sensor/{}", identifier).to_string()
    }

    fn config_topic(&self) -> String {
        let topic_prefix = self.topic_prefix();
        format!("{}/config", topic_prefix).to_string()
    }

    fn state_topic(&self) -> String {
        let topic_prefix = self.topic_prefix();
        format!("{}/state", topic_prefix).to_string()
    }

    fn homeassistant_config_message(&self) -> ConfigMessage {
        let identifier = self.identifier();
        ConfigMessage {
            name: self.label.clone(),
            device_class: "date".to_string(),
            state_topic: self.state_topic(),
            unique_id: identifier.clone(),
            object_id: identifier,
            device: Device {
                identifiers: vec![INTEGRATION_IDENTIFIER],
                name: INTEGRATION_NAME,
            },
        }
    }

    fn homeassistant_state_message(&self) -> String {
        self.date.to_string()
    }
}


fn main() {
    dotenv().expect(".env file not found");
    let client = reqwest::blocking::Client::new();

    let address = env::var("ADDRESS").expect("ADDRESS not set");

    let mqtt_host = env::var("MQTT_HOST").expect("MQTT_HOST not set");
    let mqtt_port = env::var("MQTT_PORT").or::<()>(Ok("1883".to_string()));
    let mqtt_port = mqtt_port.unwrap().parse::<u16>().expect("MQTT_PORT is not a valid port");

    let address_info = get_address_page(&client, &address);

    let next_pickups_url = address_info.unwrap().href;
    let pickups = get_next_pickups(&client, &next_pickups_url);

    let mut mqtt_options = MqttOptions::new(INTEGRATION_IDENTIFIER, mqtt_host, mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = rumqttc::Client::new(mqtt_options, 10);

    let mut ps = pickups.unwrap();
    let num_pickups = ps.len() as u16;

    for pickup in &ps {
        let config_message = pickup.homeassistant_config_message();
        let config_serialized = serde_json::to_string(&config_message).unwrap();
        client.publish(pickup.config_topic(), rumqttc::QoS::AtLeastOnce, false, config_serialized).unwrap();
    }

    for (_, notification) in connection.iter().enumerate() {
        match notification {
            Ok(Incoming(rumqttc::Packet::PubAck(PubAck { pkid }))) => {
                if pkid == num_pickups {
                    break;
                }
            }
            _ => {}
        }
    }

    sleep(Duration::from_secs(2));

    for pickup in &ps {
        let date = pickup.homeassistant_state_message();
        client.publish(pickup.state_topic(), rumqttc::QoS::AtLeastOnce, false, date).unwrap();
    }

    for (_, notification) in connection.iter().enumerate() {
        match notification {
            Ok(Incoming(rumqttc::Packet::PubAck(PubAck { pkid }))) => {
                if pkid == num_pickups * 2 {
                    break;
                }
            }
            _ => {}
        }
    }
    client.disconnect().unwrap();
}
