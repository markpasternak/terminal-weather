use crate::domain::weather::Location;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    country_name: Option<String>,
    region: Option<String>,
    timezone: Option<String>,
}

pub async fn detect_location() -> Option<Location> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let response: IpApiResponse = client
        .get("https://ipapi.co/json/")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let name = response.city.filter(|c| !c.is_empty())?;
    let latitude = response.latitude?;
    let longitude = response.longitude?;
    Some(Location {
        name,
        latitude,
        longitude,
        country: response.country_name,
        admin1: response.region,
        timezone: response.timezone,
        population: None,
    })
}
