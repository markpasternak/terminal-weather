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
    pub text: Color,
    pub muted_text: Color,
    pub particle: Color,
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
        // Keep cloudy-day palette darker than neutral UI text to avoid low contrast.
        (WeatherCategory::Cloudy, true) => ((74, 88, 104), (112, 126, 140), (230, 238, 245)),
        (WeatherCategory::Cloudy, false) => ((36, 40, 56), (70, 74, 97), (194, 207, 224)),
        (WeatherCategory::Rain, true) => ((49, 88, 145), (108, 136, 181), (174, 220, 255)),
        (WeatherCategory::Rain, false) => ((22, 36, 72), (46, 69, 111), (153, 197, 255)),
        (WeatherCategory::Snow, true) => ((86, 122, 164), (134, 166, 194), (237, 247, 255)),
        (WeatherCategory::Snow, false) => ((49, 69, 99), (88, 115, 150), (237, 247, 255)),
        (WeatherCategory::Fog, true) => ((82, 91, 97), (120, 128, 134), (224, 224, 224)),
        (WeatherCategory::Fog, false) => ((46, 52, 57), (87, 95, 101), (201, 207, 211)),
        (WeatherCategory::Thunder, true) => ((61, 58, 115), (95, 94, 164), (255, 223, 112)),
        (WeatherCategory::Thunder, false) => ((25, 22, 58), (48, 45, 89), (255, 208, 75)),
        (WeatherCategory::Unknown, true) => ((90, 110, 125), (160, 178, 190), (224, 224, 224)),
        (WeatherCategory::Unknown, false) => ((40, 46, 58), (75, 86, 99), (195, 205, 215)),
    };

    let avg_luma = (luma(top.0, top.1, top.2) + luma(bottom.0, bottom.1, bottom.2)) / 2.0;
    let prefers_dark_text = avg_luma >= 140.0;

    let text_rgb = if prefers_dark_text {
        (16, 22, 29)
    } else {
        (241, 245, 249)
    };
    let muted_rgb = if prefers_dark_text {
        (51, 65, 85)
    } else {
        (203, 213, 225)
    };
    let particle_rgb = if prefers_dark_text {
        (75, 85, 99)
    } else {
        (226, 232, 240)
    };

    Theme {
        top: quantize(Color::Rgb(top.0, top.1, top.2), capability),
        bottom: quantize(Color::Rgb(bottom.0, bottom.1, bottom.2), capability),
        accent: quantize(Color::Rgb(accent.0, accent.1, accent.2), capability),
        text: quantize(Color::Rgb(text_rgb.0, text_rgb.1, text_rgb.2), capability),
        muted_text: quantize(
            Color::Rgb(muted_rgb.0, muted_rgb.1, muted_rgb.2),
            capability,
        ),
        particle: quantize(
            Color::Rgb(particle_rgb.0, particle_rgb.1, particle_rgb.2),
            capability,
        ),
    }
}

fn luma(r: u8, g: u8, b: u8) -> f32 {
    (0.2126 * f32::from(r)) + (0.7152 * f32::from(g)) + (0.0722 * f32::from(b))
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
