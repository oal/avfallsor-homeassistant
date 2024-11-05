use scraper::{Element, Html, Selector};
use std::collections::HashMap;
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone};
use serde::Deserialize;
use crate::pickup::{Pickup, PickupType};

const ADDRESS_ENDPOINT: &str = "https://avfallsor.no/wp-json/addresses/v1/address";

#[derive(Debug, Deserialize, Clone)]
pub struct AddressResponse {
    value: String,
    pub(crate) href: String,
}

pub fn get_address_page(client: &reqwest::blocking::Client, address: &str) -> anyhow::Result<AddressResponse> {
    let response = client.get(ADDRESS_ENDPOINT)
        .query(&[("address", address)])
        .send()?;

    let result = response.json::<HashMap<String, AddressResponse>>()?;
    result.values().next().cloned()
        .ok_or_else(|| anyhow::anyhow!("Address not found"))
}

fn map_month(month: &str) -> i32 {
    match month {
        "januar" => 1,
        "februar" => 2,
        "mars" => 3,
        "april" => 4,
        "mai" => 5,
        "juni" => 6,
        "juli" => 7,
        "august" => 8,
        "september" => 9,
        "oktober" => 10,
        "november" => 11,
        "desember" => 12,
        _ => panic!("Invalid month")
    }
}

pub fn get_next_pickups(client: &reqwest::blocking::Client, url: &str, collection_time: &NaiveTime) -> anyhow::Result<Vec<Pickup>> {
    let response = client.get(url).send()?;
    let body = response.text()?;
    let document = Html::parse_document(&body);

    // Selectors
    let next_pickup_headings_selector = Selector::parse(".pickup-days-small h3").unwrap();
    let waste_icon_selector = Selector::parse(".waste-icon").unwrap();

    let current_year = chrono::Local::now().year();

    // Extract pickup kinds and dates
    Ok(document.select(&next_pickup_headings_selector).map(|el| {
        let date_str = el.text().next()?.split(" ").collect::<Vec<&str>>();
        let day_of_month = &date_str[1][0..2];
        let date_str = format!("{}-{}-{}", day_of_month, map_month(date_str[2]), &current_year);
        let date_time = NaiveDate::parse_from_str(&date_str, "%d-%m-%Y").ok()?.and_time(*collection_time);
        let local_date_time = chrono::Local.from_local_datetime(&date_time).single()?;

        let pickups = if let Some(type_container) = el.next_sibling_element() {
            let pickups = type_container.select(&waste_icon_selector).filter_map(|icon| {
                let icon_class = icon.value().attr("class")?.split("--").last()?;
                if let Some(kind) = PickupType::from_str(icon_class) {
                    Some(Pickup::new(local_date_time, kind.to_string(), kind))
                } else {
                    None
                }
            }).collect::<Vec<Pickup>>();
            Some(pickups)
        } else {
            None
        };
        pickups
    }).filter_map(|x| x).flatten().collect::<Vec<Pickup>>())
}
