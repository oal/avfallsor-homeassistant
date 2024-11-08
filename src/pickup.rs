use std::fmt;
use chrono::Local;
use crate::homeassistant::{ConfigMessage, Device};
use crate::{INTEGRATION_IDENTIFIER, INTEGRATION_NAME};

#[derive(Debug)]
pub enum PickupType {
    Garbage,
    Paper,
    Plastic,
    GlassMetal,
    FoodWaste,
}

impl PickupType {
    pub(crate) fn from_str(s: &str) -> Option<PickupType> {
        match s {
            "residual" => Some(PickupType::Garbage),
            "cardboard" => Some(PickupType::Paper),
            "plastic" => Some(PickupType::Plastic),
            "glass" => Some(PickupType::GlassMetal),
            "bio" => Some(PickupType::FoodWaste),
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
pub struct Pickup {
    date: chrono::DateTime<Local>,
    label: String,
    kind: PickupType,
}

impl Pickup {
    pub(crate) fn new(date: chrono::DateTime<Local>, label: String, kind: PickupType) -> Pickup {
        Pickup {
            date,
            label,
            kind,
        }
    }
    fn identifier(&self) -> String {
        let kind = self.kind.to_string().to_lowercase();
        format!("{}-{}", INTEGRATION_IDENTIFIER, kind)
    }

    fn topic_prefix(&self) -> String {
        let identifier = self.identifier();
        format!("homeassistant/sensor/{}", identifier).to_string()
    }

    pub(crate) fn config_topic(&self) -> String {
        let topic_prefix = self.topic_prefix();
        format!("{}/config", topic_prefix).to_string()
    }

    pub(crate) fn state_topic(&self) -> String {
        let topic_prefix = self.topic_prefix();
        format!("{}/state", topic_prefix).to_string()
    }

    pub(crate) fn homeassistant_config_message(&self) -> ConfigMessage {
        let identifier = self.identifier();
        ConfigMessage {
            name: self.label.clone(),
            device_class: "timestamp".to_string(),
            // unit_of_measurement: "date".to_string(),
            state_topic: self.state_topic(),
            unique_id: identifier.clone(),
            object_id: identifier,
            device: Device {
                identifiers: vec![INTEGRATION_IDENTIFIER],
                name: INTEGRATION_NAME,
            },
        }
    }

    pub(crate) fn homeassistant_state_message(&self) -> String {
        self.date.format("%+").to_string()
    }
}
