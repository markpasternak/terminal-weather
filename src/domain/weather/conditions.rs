use crate::cli::IconMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherCategory {
    Clear,
    Cloudy,
    Rain,
    Snow,
    Fog,
    Thunder,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleKind {
    None,
    Rain,
    Snow,
    Fog,
    Thunder,
}

#[must_use]
pub fn weather_code_to_category(code: u8) -> WeatherCategory {
    match code {
        0 | 1 => WeatherCategory::Clear,
        2 | 3 => WeatherCategory::Cloudy,
        45 | 48 => WeatherCategory::Fog,
        51..=57 | 61..=67 | 80..=82 => WeatherCategory::Rain,
        71..=77 | 85..=86 => WeatherCategory::Snow,
        95 | 96 | 99 => WeatherCategory::Thunder,
        _ => WeatherCategory::Unknown,
    }
}

#[must_use]
pub fn weather_code_to_particle(code: u8) -> ParticleKind {
    match weather_code_to_category(code) {
        WeatherCategory::Rain => ParticleKind::Rain,
        WeatherCategory::Snow => ParticleKind::Snow,
        WeatherCategory::Fog => ParticleKind::Fog,
        WeatherCategory::Thunder => ParticleKind::Thunder,
        WeatherCategory::Cloudy | WeatherCategory::Clear | WeatherCategory::Unknown => {
            ParticleKind::None
        }
    }
}

#[must_use]
pub fn weather_label(code: u8) -> &'static str {
    weather_label_for_time(code, true)
}

#[must_use]
pub fn weather_label_for_time(code: u8, is_day: bool) -> &'static str {
    match code {
        0 => {
            if is_day {
                "Clear sky"
            } else {
                "Clear night"
            }
        }
        1 => {
            if is_day {
                "Mainly clear"
            } else {
                "Mainly clear night"
            }
        }
        _ => weather_label_lookup(code).unwrap_or("Unknown"),
    }
}

#[must_use]
pub fn weather_icon(code: u8, mode: IconMode, is_day: bool) -> &'static str {
    let (ascii, emoji, unicode, nerd) = icon_tokens(weather_code_to_category(code), is_day);
    match mode {
        IconMode::Ascii => ascii,
        IconMode::Emoji => emoji,
        IconMode::Unicode => unicode,
        IconMode::NerdFont => nerd,
    }
}

const WEATHER_LABELS: &[(u8, &str)] = &[
    (2, "Partly cloudy"),
    (3, "Overcast"),
    (45, "Fog"),
    (48, "Depositing rime fog"),
    (51, "Light drizzle"),
    (53, "Moderate drizzle"),
    (55, "Dense drizzle"),
    (56, "Light freezing drizzle"),
    (57, "Dense freezing drizzle"),
    (61, "Slight rain"),
    (63, "Moderate rain"),
    (65, "Heavy rain"),
    (66, "Light freezing rain"),
    (67, "Heavy freezing rain"),
    (71, "Slight snowfall"),
    (73, "Moderate snowfall"),
    (75, "Heavy snowfall"),
    (77, "Snow grains"),
    (80, "Slight rain showers"),
    (81, "Moderate rain showers"),
    (82, "Violent rain showers"),
    (85, "Slight snow showers"),
    (86, "Heavy snow showers"),
    (95, "Thunderstorm"),
    (96, "Thunderstorm + light hail"),
    (99, "Thunderstorm + heavy hail"),
];

fn weather_label_lookup(code: u8) -> Option<&'static str> {
    WEATHER_LABELS
        .iter()
        .find_map(|(candidate, label)| (*candidate == code).then_some(*label))
}

fn icon_tokens(
    category: WeatherCategory,
    is_day: bool,
) -> (&'static str, &'static str, &'static str, &'static str) {
    if matches!(category, WeatherCategory::Clear) {
        return clear_icon_tokens(is_day);
    }
    non_clear_icon_tokens(category)
}

fn clear_icon_tokens(is_day: bool) -> (&'static str, &'static str, &'static str, &'static str) {
    use nerd_font_symbols::weather::{WEATHER_DAY_SUNNY, WEATHER_NIGHT_CLEAR};
    if is_day {
        ("SUN", "☀️", "☀", WEATHER_DAY_SUNNY)
    } else {
        ("MON", "🌙", "☾", WEATHER_NIGHT_CLEAR)
    }
}

fn non_clear_icon_tokens(
    category: WeatherCategory,
) -> (&'static str, &'static str, &'static str, &'static str) {
    use nerd_font_symbols::weather::{
        WEATHER_CLOUDY, WEATHER_FOG, WEATHER_RAIN, WEATHER_SNOW, WEATHER_THUNDERSTORM,
    };
    match category {
        WeatherCategory::Cloudy => ("CLD", "☁️", "☁", WEATHER_CLOUDY),
        WeatherCategory::Rain => ("RAN", "🌧️", "🌧", WEATHER_RAIN),
        WeatherCategory::Snow => ("SNW", "🌨️", "❄", WEATHER_SNOW),
        WeatherCategory::Fog => ("FOG", "🌫️", "≡", WEATHER_FOG),
        WeatherCategory::Thunder => ("THN", "⛈️", "⚡", WEATHER_THUNDERSTORM),
        WeatherCategory::Unknown | WeatherCategory::Clear => ("---", "☁️", "☁", WEATHER_CLOUDY),
    }
}
