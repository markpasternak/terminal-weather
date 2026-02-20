pub mod atmos;
pub mod compact;
pub mod gauge;
pub mod shared;
pub mod sky;

pub use atmos::scene_for_weather;
pub use gauge::scene_for_gauge_cluster;
pub use sky::scene_for_sky_observatory;

use crate::domain::weather::WeatherCategory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandmarkTint {
    Warm,
    Cool,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct LandmarkScene {
    pub label: String,
    pub lines: Vec<String>,
    pub tint: LandmarkTint,
}

#[must_use]
pub fn tint_for_category(category: WeatherCategory) -> LandmarkTint {
    match category {
        WeatherCategory::Clear => LandmarkTint::Warm,
        WeatherCategory::Cloudy | WeatherCategory::Fog => LandmarkTint::Neutral,
        WeatherCategory::Rain | WeatherCategory::Snow | WeatherCategory::Thunder => {
            LandmarkTint::Cool
        }
        WeatherCategory::Unknown => LandmarkTint::Neutral,
    }
}

#[must_use]
pub fn scene_name(category: WeatherCategory, is_day: bool) -> &'static str {
    match (category, is_day) {
        (WeatherCategory::Clear, true) => "Clear sky",
        (WeatherCategory::Clear, false) => "Clear night",
        (WeatherCategory::Cloudy, _) => "Cloudy",
        (WeatherCategory::Rain, _) => "Rain",
        (WeatherCategory::Snow, _) => "Snow",
        (WeatherCategory::Fog, _) => "Fog",
        (WeatherCategory::Thunder, _) => "Thunderstorm",
        (WeatherCategory::Unknown, _) => "Unknown",
    }
}
