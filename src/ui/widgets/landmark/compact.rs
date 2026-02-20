use crate::domain::weather::WeatherCategory;
use crate::ui::widgets::landmark::shared::fit_lines;
use crate::ui::widgets::landmark::{LandmarkScene, scene_name, tint_for_category};

pub fn compact_condition_scene(
    category: WeatherCategory,
    is_day: bool,
    width: u16,
    height: u16,
) -> LandmarkScene {
    let mut lines = match (category, is_day) {
        (WeatherCategory::Clear, true) => vec![
            "   \\  |  /   ".to_string(),
            " --   O   -- ".to_string(),
            "   /  |  \\   ".to_string(),
        ],
        (WeatherCategory::Clear, false) => vec![
            "    _..._    ".to_string(),
            "  .:::::::.  ".to_string(),
            "   ':::::'   ".to_string(),
        ],
        (WeatherCategory::Cloudy, _) => vec![
            "    .--.     ".to_string(),
            " .-(____)-.  ".to_string(),
            "    (__)     ".to_string(),
        ],
        (WeatherCategory::Rain, _) => vec![
            "    .--.     ".to_string(),
            " .-(____)-.  ".to_string(),
            "   / / / /   ".to_string(),
        ],
        (WeatherCategory::Snow, _) => vec![
            "    .--.     ".to_string(),
            " .-(____)-.  ".to_string(),
            "   *  *  *   ".to_string(),
        ],
        (WeatherCategory::Thunder, _) => vec![
            "    .--.     ".to_string(),
            " .-(____)-.  ".to_string(),
            "    /\\/\\/    ".to_string(),
        ],
        (WeatherCategory::Fog, _) => vec![
            "  ~~~~~~~~~~ ".to_string(),
            " ~ ~~~~~~~~ ~".to_string(),
            "  ~~~~~~~~~~ ".to_string(),
        ],
        _ => vec![
            "    .--.     ".to_string(),
            "   ( ?? )    ".to_string(),
            "    '--'     ".to_string(),
        ],
    };

    if usize::from(height) > lines.len() && width >= 10 {
        lines.push(format!(
            "{:^w$}",
            compact_scene_label(category, is_day),
            w = usize::from(width)
        ));
    }

    LandmarkScene {
        label: format!("Atmos Canvas · {}", scene_name(category, is_day)),
        lines: fit_lines(lines, width as usize, height as usize),
        tint: tint_for_category(category),
    }
}

fn compact_scene_label(category: WeatherCategory, is_day: bool) -> &'static str {
    match (category, is_day) {
        (WeatherCategory::Clear, true) => "CLEAR",
        (WeatherCategory::Clear, false) => "CLEAR NIGHT",
        (WeatherCategory::Cloudy, _) => "CLOUDY",
        (WeatherCategory::Rain, _) => "RAIN",
        (WeatherCategory::Snow, _) => "SNOW",
        (WeatherCategory::Fog, _) => "FOG",
        (WeatherCategory::Thunder, _) => "THUNDER",
        (WeatherCategory::Unknown, _) => "WEATHER",
    }
}
