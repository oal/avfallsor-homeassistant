use scraper::{Html, Selector};
use std::collections::HashMap;
use serde::Deserialize;
use crate::{Pickup, PickupType};

const ADDRESS_ENDPOINT: &str = "https://avfallsor.no/wp-json/addresses/v1/address";

#[derive(Debug, Deserialize, Clone)]
pub struct AddressResponse {
    value: String,
    pub(crate) href: String,
}

pub fn get_address_page(client: &reqwest::blocking::Client, address: &str) -> anyhow::Result<AddressResponse> {
    let mut response = client.get(ADDRESS_ENDPOINT)
        .query(&[("address", address)])
        .send()?;

    let result = response.json::<HashMap<String, AddressResponse>>()?;
    result.values().next().cloned()
        .ok_or_else(|| anyhow::anyhow!("Address not found"))
}

pub fn get_next_pickups(client: &reqwest::blocking::Client, url: &str) -> anyhow::Result<Vec<Pickup>> {
    let mut response = client.get(url).send()?;
    let body = response.text()?;
    let document = Html::parse_document(&body);

    // Selectors
    let next_pickups_selector = Selector::parse(".pickup-days-small form").unwrap();
    let description_selector = Selector::parse("input[name=description]").unwrap();
    let date_selector = Selector::parse("input[name=dtstart]").unwrap();

    // Extract pickup kinds and dates
    Ok(document.select(&next_pickups_selector).filter_map(|el| {
        let description = el.select(&description_selector).next()?;
        let date_str = el.select(&date_selector).next()?;
        let date = chrono::NaiveDate::parse_from_str(date_str.value().attr("value")?, "%Y-%m-%d").ok()?;
        let description = description.value().attr("value")?;

        let kind = PickupType::from_str(description)?;

        Some(Pickup {
            date,
            label: description.to_string(),
            kind,
        })
    }).collect())
}
