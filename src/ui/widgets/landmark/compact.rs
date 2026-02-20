use crate::domain::weather::WeatherCategory;
use crate::ui::widgets::landmark::shared::fit_lines;
use crate::ui::widgets::landmark::{LandmarkScene, scene_name, tint_for_category};

const LABELS: &[(WeatherCategory, &str)] = &[
    (WeatherCategory::Cloudy, "CLOUDY"),
    (WeatherCategory::Rain, "RAIN"),
    (WeatherCategory::Snow, "SNOW"),
    (WeatherCategory::Fog, "FOG"),
    (WeatherCategory::Thunder, "THUNDER"),
    (WeatherCategory::Unknown, "WEATHER"),
];

#[must_use]
pub fn compact_condition_scene(
    category: WeatherCategory,
    is_day: bool,
    width: u16,
    height: u16,
) -> LandmarkScene {
    let mut lines = compact_art_lines(category, is_day);
    append_compact_label(&mut lines, category, is_day, width, height);

    LandmarkScene {
        label: format!("Atmos Canvas · {}", scene_name(category, is_day)),
        lines: fit_lines(lines, width as usize, height as usize),
        tint: tint_for_category(category),
        context_line: None,
    }
}

fn compact_scene_label(category: WeatherCategory, is_day: bool) -> &'static str {
    if matches!(category, WeatherCategory::Clear) {
        return if is_day { "CLEAR" } else { "CLEAR NIGHT" };
    }
    for (candidate, label) in LABELS {
        if *candidate == category {
            return label;
        }
    }
    "WEATHER"
}

fn compact_non_clear_lines(category: WeatherCategory) -> Vec<String> {
    match category {
        WeatherCategory::Cloudy => cloud_scene_lines("    (__)     "),
        WeatherCategory::Rain => cloud_scene_lines("   / / / /   "),
        WeatherCategory::Snow => cloud_scene_lines("   *  *  *   "),
        WeatherCategory::Thunder => cloud_scene_lines("    /\\/\\/    "),
        WeatherCategory::Fog => vec![
            "  ~~~~~~~~~~ ".to_string(),
            " ~ ~~~~~~~~ ~".to_string(),
            "  ~~~~~~~~~~ ".to_string(),
        ],
        WeatherCategory::Unknown | WeatherCategory::Clear => vec![
            "    .--.     ".to_string(),
            "   ( ?? )    ".to_string(),
            "    '--'     ".to_string(),
        ],
    }
}

fn compact_art_lines(category: WeatherCategory, is_day: bool) -> Vec<String> {
    if matches!(category, WeatherCategory::Clear) {
        return if is_day {
            vec![
                "   \\  |  /   ".to_string(),
                " --   O   -- ".to_string(),
                "   /  |  \\   ".to_string(),
            ]
        } else {
            vec![
                "    _..._    ".to_string(),
                "  .:::::::.  ".to_string(),
                "   ':::::'   ".to_string(),
            ]
        };
    }
    compact_non_clear_lines(category)
}

fn cloud_scene_lines(bottom_line: &str) -> Vec<String> {
    vec![
        "    .--.     ".to_string(),
        " .-(____)-.  ".to_string(),
        bottom_line.to_string(),
    ]
}

fn append_compact_label(
    lines: &mut Vec<String>,
    category: WeatherCategory,
    is_day: bool,
    width: u16,
    height: u16,
) {
    if usize::from(height) <= lines.len() || width < 10 {
        return;
    }
    lines.push(format!(
        "{:^w$}",
        compact_scene_label(category, is_day),
        w = usize::from(width)
    ));
}
