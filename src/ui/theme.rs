use ratatui::style::Color;

use crate::{cli::ThemeArg, domain::weather::WeatherCategory};

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
    pub border: Color,
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub temp_freezing: Color,
    pub temp_cold: Color,
    pub temp_mild: Color,
    pub temp_warm: Color,
    pub temp_hot: Color,
    pub landmark_warm: Color,
    pub landmark_cool: Color,
    pub landmark_neutral: Color,
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

pub fn theme_for(
    category: WeatherCategory,
    is_day: bool,
    capability: ColorCapability,
    mode: ThemeArg,
) -> Theme {
    let (mut top, mut bottom, accent) = match mode {
        ThemeArg::Auto => match (category, is_day) {
            (WeatherCategory::Clear, true) => ((13, 53, 102), (30, 102, 158), (255, 215, 117)),
            (WeatherCategory::Clear, false) => ((9, 18, 44), (21, 43, 79), (173, 216, 255)),
            (WeatherCategory::Cloudy, true) => ((25, 36, 51), (48, 63, 84), (210, 223, 235)),
            (WeatherCategory::Cloudy, false) => ((20, 26, 40), (34, 42, 62), (194, 207, 224)),
            (WeatherCategory::Rain, true) => ((17, 47, 88), (32, 73, 126), (153, 214, 255)),
            (WeatherCategory::Rain, false) => ((12, 25, 52), (25, 44, 78), (143, 196, 255)),
            (WeatherCategory::Snow, true) => ((27, 51, 77), (43, 74, 106), (237, 247, 255)),
            (WeatherCategory::Snow, false) => ((19, 35, 55), (34, 55, 80), (226, 241, 255)),
            (WeatherCategory::Fog, true) => ((30, 34, 40), (50, 55, 62), (216, 220, 224)),
            (WeatherCategory::Fog, false) => ((21, 24, 30), (33, 37, 43), (201, 207, 211)),
            (WeatherCategory::Thunder, true) => ((28, 25, 66), (42, 40, 97), (255, 223, 112)),
            (WeatherCategory::Thunder, false) => ((18, 15, 44), (28, 24, 63), (255, 208, 95)),
            (WeatherCategory::Unknown, true) => ((28, 36, 51), (42, 53, 73), (205, 219, 234)),
            (WeatherCategory::Unknown, false) => ((19, 24, 35), (31, 39, 53), (195, 205, 215)),
        },
        ThemeArg::Aurora => ((9, 31, 65), (16, 70, 105), (102, 232, 242)),
        ThemeArg::Mono => ((17, 17, 24), (32, 35, 44), (196, 201, 214)),
        ThemeArg::HighContrast => ((0, 0, 0), (10, 10, 16), (255, 210, 0)),
    };

    if capability == ColorCapability::Basic16 {
        top = (0, 0, 0);
        bottom = match category {
            WeatherCategory::Clear => (0, 32, 72),
            WeatherCategory::Cloudy => (25, 30, 35),
            WeatherCategory::Rain => (0, 22, 56),
            WeatherCategory::Snow => (28, 38, 56),
            WeatherCategory::Fog => (30, 30, 30),
            WeatherCategory::Thunder => (24, 0, 44),
            WeatherCategory::Unknown => (20, 24, 32),
        };
    }

    let avg_luma = (luma(top.0, top.1, top.2) + luma(bottom.0, bottom.1, bottom.2)) / 2.0;
    let dark_text = avg_luma >= 170.0;

    let text = if dark_text {
        (12, 16, 24)
    } else {
        (240, 245, 250)
    };
    let muted = if dark_text {
        (51, 65, 85)
    } else {
        (187, 199, 214)
    };
    let particle = if dark_text {
        (70, 85, 100)
    } else {
        (206, 221, 235)
    };
    let border = if dark_text {
        (25, 33, 47)
    } else {
        (214, 225, 239)
    };

    let info = if dark_text {
        (3, 105, 161)
    } else {
        (125, 211, 252)
    };
    let success = if dark_text {
        (21, 128, 61)
    } else {
        (74, 222, 128)
    };
    let warning = if dark_text {
        (161, 98, 7)
    } else {
        (251, 191, 36)
    };
    let danger = if dark_text {
        (185, 28, 28)
    } else {
        (248, 113, 113)
    };

    Theme {
        top: quantize(Color::Rgb(top.0, top.1, top.2), capability),
        bottom: quantize(Color::Rgb(bottom.0, bottom.1, bottom.2), capability),
        accent: quantize(Color::Rgb(accent.0, accent.1, accent.2), capability),
        text: quantize(Color::Rgb(text.0, text.1, text.2), capability),
        muted_text: quantize(Color::Rgb(muted.0, muted.1, muted.2), capability),
        particle: quantize(Color::Rgb(particle.0, particle.1, particle.2), capability),
        border: quantize(Color::Rgb(border.0, border.1, border.2), capability),
        info: quantize(Color::Rgb(info.0, info.1, info.2), capability),
        success: quantize(Color::Rgb(success.0, success.1, success.2), capability),
        warning: quantize(Color::Rgb(warning.0, warning.1, warning.2), capability),
        danger: quantize(Color::Rgb(danger.0, danger.1, danger.2), capability),
        temp_freezing: quantize(Color::Rgb(147, 197, 253), capability),
        temp_cold: quantize(Color::Rgb(56, 189, 248), capability),
        temp_mild: quantize(Color::Rgb(110, 231, 183), capability),
        temp_warm: quantize(Color::Rgb(251, 191, 36), capability),
        temp_hot: quantize(Color::Rgb(248, 113, 113), capability),
        landmark_warm: quantize(Color::Rgb(253, 230, 138), capability),
        landmark_cool: quantize(Color::Rgb(147, 197, 253), capability),
        landmark_neutral: quantize(Color::Rgb(muted.0, muted.1, muted.2), capability),
    }
}

pub fn condition_color(theme: &Theme, category: WeatherCategory) -> Color {
    match category {
        WeatherCategory::Clear => theme.warning,
        WeatherCategory::Cloudy => theme.muted_text,
        WeatherCategory::Rain => theme.info,
        WeatherCategory::Snow => theme.text,
        WeatherCategory::Fog => theme.landmark_neutral,
        WeatherCategory::Thunder => theme.danger,
        WeatherCategory::Unknown => theme.accent,
    }
}

pub fn icon_color(theme: &Theme, category: WeatherCategory) -> Color {
    match category {
        WeatherCategory::Clear => theme.warning,
        WeatherCategory::Cloudy => theme.muted_text,
        WeatherCategory::Rain => theme.info,
        WeatherCategory::Snow => theme.text,
        WeatherCategory::Fog => theme.landmark_neutral,
        WeatherCategory::Thunder => theme.danger,
        WeatherCategory::Unknown => theme.accent,
    }
}

pub fn temp_color(theme: &Theme, temp: f32) -> Color {
    if temp <= -8.0 {
        theme.temp_freezing
    } else if temp <= 2.0 {
        theme.temp_cold
    } else if temp <= 16.0 {
        theme.temp_mild
    } else if temp <= 28.0 {
        theme.temp_warm
    } else {
        theme.temp_hot
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
        (ColorCapability::Basic16, Color::Rgb(r, g, b)) => basic16_from_rgb(r, g, b),
        (_, c) => c,
    }
}

fn basic16_from_rgb(r: u8, g: u8, b: u8) -> Color {
    let rf = f32::from(r) / 255.0;
    let gf = f32::from(g) / 255.0;
    let bf = f32::from(b) / 255.0;

    let max = rf.max(gf.max(bf));
    let min = rf.min(gf.min(bf));
    let delta = max - min;
    let light = (max + min) / 2.0;

    if delta < 0.08 {
        if light < 0.20 {
            return Color::Black;
        }
        if light < 0.40 {
            return Color::DarkGray;
        }
        if light < 0.72 {
            return Color::Gray;
        }
        return Color::White;
    }

    let hue = if (max - rf).abs() < f32::EPSILON {
        60.0 * ((gf - bf) / delta).rem_euclid(6.0)
    } else if (max - gf).abs() < f32::EPSILON {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };

    let bright = light >= 0.55;
    match hue {
        h if !(30.0..330.0).contains(&h) => {
            if bright {
                Color::LightRed
            } else {
                Color::Red
            }
        }
        h if h < 90.0 => {
            if bright {
                Color::LightYellow
            } else {
                Color::Yellow
            }
        }
        h if h < 150.0 => {
            if bright {
                Color::LightGreen
            } else {
                Color::Green
            }
        }
        h if h < 210.0 => {
            if bright {
                Color::LightCyan
            } else {
                Color::Cyan
            }
        }
        h if h < 270.0 => {
            if bright {
                Color::LightBlue
            } else {
                Color::Blue
            }
        }
        _ => {
            if bright {
                Color::LightMagenta
            } else {
                Color::Magenta
            }
        }
    }
}
