use ratatui::style::Color;

use crate::domain::weather::WeatherCategory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    TrueColor,
    Xterm256,
    Basic16,
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub top: Color,
    pub bottom: Color,
    pub accent: Color,
}

pub fn detect_color_capability() -> ColorCapability {
    if std::env::var_os("NO_COLOR").is_some() {
        return ColorCapability::Basic16;
    }

    let colorterm = std::env::var("COLORTERM")
        .unwrap_or_default()
        .to_lowercase();
    if colorterm.contains("truecolor") || colorterm.contains("24bit") {
        return ColorCapability::TrueColor;
    }

    let term = std::env::var("TERM").unwrap_or_default().to_lowercase();
    if term.contains("256color") {
        ColorCapability::Xterm256
    } else {
        ColorCapability::Basic16
    }
}

pub fn theme_for(category: WeatherCategory, is_day: bool, capability: ColorCapability) -> Theme {
    let (top, bottom, accent) = match (category, is_day) {
        (WeatherCategory::Clear, true) => ((93, 188, 255), (184, 234, 255), (255, 215, 87)),
        (WeatherCategory::Clear, false) => ((14, 24, 56), (35, 48, 99), (173, 216, 255)),
        (WeatherCategory::Cloudy, true) => ((120, 138, 155), (186, 198, 211), (245, 245, 245)),
        (WeatherCategory::Cloudy, false) => ((36, 40, 56), (70, 74, 97), (194, 207, 224)),
        (WeatherCategory::Rain, true) => ((49, 88, 145), (108, 136, 181), (174, 220, 255)),
        (WeatherCategory::Rain, false) => ((22, 36, 72), (46, 69, 111), (153, 197, 255)),
        (WeatherCategory::Snow, true) => ((141, 181, 227), (224, 239, 255), (255, 255, 255)),
        (WeatherCategory::Snow, false) => ((49, 69, 99), (88, 115, 150), (237, 247, 255)),
        (WeatherCategory::Fog, true) => ((128, 137, 141), (188, 194, 196), (224, 224, 224)),
        (WeatherCategory::Fog, false) => ((46, 52, 57), (87, 95, 101), (201, 207, 211)),
        (WeatherCategory::Thunder, true) => ((61, 58, 115), (95, 94, 164), (255, 223, 112)),
        (WeatherCategory::Thunder, false) => ((25, 22, 58), (48, 45, 89), (255, 208, 75)),
        (WeatherCategory::Unknown, true) => ((90, 110, 125), (160, 178, 190), (224, 224, 224)),
        (WeatherCategory::Unknown, false) => ((40, 46, 58), (75, 86, 99), (195, 205, 215)),
    };

    Theme {
        top: quantize(Color::Rgb(top.0, top.1, top.2), capability),
        bottom: quantize(Color::Rgb(bottom.0, bottom.1, bottom.2), capability),
        accent: quantize(Color::Rgb(accent.0, accent.1, accent.2), capability),
    }
}

pub fn quantize(color: Color, capability: ColorCapability) -> Color {
    match (capability, color) {
        (ColorCapability::TrueColor, c) => c,
        (ColorCapability::Xterm256, Color::Rgb(r, g, b)) => {
            let to_cube = |v: u8| -> u8 { ((f32::from(v) / 255.0) * 5.0).round() as u8 };
            let ri = to_cube(r);
            let gi = to_cube(g);
            let bi = to_cube(b);
            let index = 16 + 36 * ri + 6 * gi + bi;
            Color::Indexed(index)
        }
        (ColorCapability::Basic16, Color::Rgb(r, g, b)) => {
            let luma = (0.2126 * f32::from(r)) + (0.7152 * f32::from(g)) + (0.0722 * f32::from(b));
            if luma > 160.0 {
                Color::White
            } else if luma > 90.0 {
                Color::Gray
            } else {
                Color::Black
            }
        }
        (_, c) => c,
    }
}
