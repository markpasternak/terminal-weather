#![allow(clippy::match_same_arms)]

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
    pub context_line: Option<String>,
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
    if category == WeatherCategory::Clear {
        if is_day { "Clear sky" } else { "Clear night" }
    } else {
        non_clear_scene_name(category)
    }
}

fn non_clear_scene_name(category: WeatherCategory) -> &'static str {
    match category {
        WeatherCategory::Cloudy => "Cloudy",
        WeatherCategory::Rain => "Rain",
        WeatherCategory::Snow => "Snow",
        WeatherCategory::Fog => "Fog",
        WeatherCategory::Thunder => "Thunderstorm",
        WeatherCategory::Unknown | WeatherCategory::Clear => "Unknown",
    }
}
