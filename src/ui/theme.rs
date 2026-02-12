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
    pub surface: Color,
    pub surface_alt: Color,
    pub popup_surface: Color,
    pub accent: Color,
    pub text: Color,
    pub muted_text: Color,
    pub popup_text: Color,
    pub popup_muted_text: Color,
    pub particle: Color,
    pub border: Color,
    pub popup_border: Color,
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub temp_freezing: Color,
    pub temp_cold: Color,
    pub temp_mild: Color,
    pub temp_warm: Color,
    pub temp_hot: Color,
    pub range_track: Color,
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
    let (mut top, mut bottom, accent_seed) = match mode {
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
        ThemeArg::Dracula => ((40, 42, 54), (68, 71, 90), (189, 147, 249)),
        ThemeArg::GruvboxMaterialDark => ((40, 40, 40), (60, 56, 54), (216, 166, 87)),
        ThemeArg::KanagawaWave => ((31, 31, 40), (43, 46, 58), (126, 156, 216)),
        ThemeArg::AyuMirage => ((31, 36, 48), (46, 53, 71), (109, 203, 250)),
        ThemeArg::AyuLight => ((248, 249, 250), (232, 236, 242), (49, 153, 225)),
        ThemeArg::PoimandresStorm => ((37, 43, 55), (56, 65, 84), (137, 221, 255)),
        ThemeArg::SelenizedDark => ((16, 60, 72), (24, 73, 86), (70, 149, 247)),
        ThemeArg::NoClownFiesta => ((16, 16, 16), (33, 37, 45), (186, 215, 255)),
    };

    if capability == ColorCapability::Basic16 {
        if mode == ThemeArg::Auto {
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

            return Theme {
                top: quantize(Color::Rgb(top.0, top.1, top.2), capability),
                bottom: quantize(Color::Rgb(bottom.0, bottom.1, bottom.2), capability),
                surface: Color::Black,
                surface_alt: Color::DarkGray,
                popup_surface: Color::Blue,
                accent: Color::Cyan,
                text: Color::White,
                muted_text: Color::Gray,
                popup_text: Color::White,
                popup_muted_text: Color::Gray,
                particle: Color::Gray,
                border: Color::LightCyan,
                popup_border: Color::Yellow,
                info: Color::LightCyan,
                success: Color::LightGreen,
                warning: Color::Yellow,
                danger: Color::LightRed,
                temp_freezing: Color::LightBlue,
                temp_cold: Color::Cyan,
                temp_mild: Color::Green,
                temp_warm: Color::Yellow,
                temp_hot: Color::LightRed,
                range_track: Color::Gray,
                landmark_warm: Color::Yellow,
                landmark_cool: Color::LightBlue,
                landmark_neutral: Color::Gray,
            };
        }

        let (surface, surface_alt, popup_surface, accent, border, popup_border) = match mode {
            ThemeArg::Aurora => (
                Color::Blue,
                Color::Cyan,
                Color::DarkGray,
                Color::LightCyan,
                Color::LightCyan,
                Color::White,
            ),
            ThemeArg::Mono => (
                Color::Black,
                Color::DarkGray,
                Color::DarkGray,
                Color::White,
                Color::Gray,
                Color::White,
            ),
            ThemeArg::HighContrast => (
                Color::Black,
                Color::Black,
                Color::Black,
                Color::Yellow,
                Color::White,
                Color::Yellow,
            ),
            ThemeArg::Dracula => (
                Color::Magenta,
                Color::Blue,
                Color::DarkGray,
                Color::LightMagenta,
                Color::LightMagenta,
                Color::White,
            ),
            ThemeArg::GruvboxMaterialDark => (
                Color::Black,
                Color::DarkGray,
                Color::DarkGray,
                Color::Yellow,
                Color::Yellow,
                Color::White,
            ),
            ThemeArg::KanagawaWave => (
                Color::Blue,
                Color::DarkGray,
                Color::DarkGray,
                Color::LightBlue,
                Color::LightBlue,
                Color::White,
            ),
            ThemeArg::AyuMirage => (
                Color::Blue,
                Color::DarkGray,
                Color::DarkGray,
                Color::Cyan,
                Color::LightCyan,
                Color::White,
            ),
            ThemeArg::AyuLight => (
                Color::Gray,
                Color::White,
                Color::DarkGray,
                Color::Blue,
                Color::Blue,
                Color::Black,
            ),
            ThemeArg::PoimandresStorm => (
                Color::Blue,
                Color::DarkGray,
                Color::DarkGray,
                Color::LightCyan,
                Color::Cyan,
                Color::White,
            ),
            ThemeArg::SelenizedDark => (
                Color::Cyan,
                Color::Blue,
                Color::DarkGray,
                Color::LightBlue,
                Color::LightBlue,
                Color::White,
            ),
            ThemeArg::NoClownFiesta => (
                Color::Black,
                Color::DarkGray,
                Color::DarkGray,
                Color::LightBlue,
                Color::Gray,
                Color::White,
            ),
            ThemeArg::Auto => unreachable!("handled above"),
        };

        let text = if mode == ThemeArg::AyuLight {
            Color::Black
        } else {
            Color::White
        };
        let muted = if mode == ThemeArg::AyuLight {
            Color::DarkGray
        } else {
            Color::Gray
        };
        let particle = if mode == ThemeArg::AyuLight {
            Color::Gray
        } else {
            Color::DarkGray
        };

        return Theme {
            top: quantize(Color::Rgb(top.0, top.1, top.2), capability),
            bottom: quantize(Color::Rgb(bottom.0, bottom.1, bottom.2), capability),
            surface,
            surface_alt,
            popup_surface,
            accent,
            text,
            muted_text: muted,
            popup_text: text,
            popup_muted_text: muted,
            particle,
            border,
            popup_border,
            info: Color::LightCyan,
            success: Color::LightGreen,
            warning: Color::Yellow,
            danger: Color::LightRed,
            temp_freezing: Color::LightBlue,
            temp_cold: Color::Cyan,
            temp_mild: Color::Green,
            temp_warm: Color::Yellow,
            temp_hot: Color::LightRed,
            range_track: muted,
            landmark_warm: Color::Yellow,
            landmark_cool: Color::LightBlue,
            landmark_neutral: muted,
        };
    }

    let avg_luma = (luma(top.0, top.1, top.2) + luma(bottom.0, bottom.1, bottom.2)) / 2.0;
    let dark_text = avg_luma >= 170.0;

    if dark_text {
        // Keep light themes readable in terminals by avoiding near-white panel stacks.
        top = mix_rgb(top, (220, 228, 239), 0.24);
        bottom = mix_rgb(bottom, (204, 216, 232), 0.20);
    }

    let base_surface = mix_rgb(top, bottom, 0.80);
    let base_surface_alt = mix_rgb(top, bottom, 0.60);
    let surface = mix_rgb(
        base_surface,
        accent_seed,
        if dark_text { 0.08 } else { 0.16 },
    );
    let surface_alt = mix_rgb(
        base_surface_alt,
        accent_seed,
        if dark_text { 0.12 } else { 0.24 },
    );
    let popup_surface = if dark_text {
        mix_rgb(surface_alt, (20, 27, 38), 0.24)
    } else {
        mix_rgb(surface_alt, (236, 243, 251), 0.18)
    };

    let text_seed = if dark_text {
        (12, 16, 24)
    } else {
        (240, 245, 250)
    };
    let muted_seed = if dark_text {
        (55, 68, 85)
    } else {
        (183, 198, 214)
    };
    let text = ensure_contrast_multi(text_seed, &[surface, surface_alt, popup_surface], 4.7);
    let muted = ensure_contrast_multi(muted_seed, &[surface, surface_alt, popup_surface], 3.2);
    let popup_text = ensure_contrast(text_seed, popup_surface, 4.7);
    let popup_muted_text = ensure_contrast(muted_seed, popup_surface, 3.2);
    let accent = ensure_contrast_multi(accent_seed, &[surface, surface_alt, popup_surface], 3.0);

    let particle = if dark_text {
        (92, 108, 124)
    } else {
        (202, 218, 235)
    };
    let border_seed = if dark_text {
        mix_rgb(surface, (18, 26, 38), 0.74)
    } else {
        mix_rgb(surface, accent, 0.54)
    };
    let border = ensure_contrast_multi(border_seed, &[surface, surface_alt, top, bottom], 2.25);
    let popup_border_seed = if dark_text {
        mix_rgb(popup_surface, (5, 11, 18), 0.82)
    } else {
        mix_rgb(popup_surface, accent, 0.70)
    };
    let popup_border = ensure_contrast(popup_border_seed, popup_surface, 2.6);

    let info = ensure_contrast_multi(
        if dark_text {
            (3, 105, 161)
        } else {
            (125, 211, 252)
        },
        &[surface, surface_alt, popup_surface],
        3.0,
    );
    let success = ensure_contrast_multi(
        if dark_text {
            (21, 128, 61)
        } else {
            (74, 222, 128)
        },
        &[surface, surface_alt, popup_surface],
        3.0,
    );
    let warning = ensure_contrast_multi(
        if dark_text {
            (161, 98, 7)
        } else {
            (251, 191, 36)
        },
        &[surface, surface_alt, popup_surface],
        3.0,
    );
    let danger = ensure_contrast_multi(
        if dark_text {
            (185, 28, 28)
        } else {
            (248, 113, 113)
        },
        &[surface, surface_alt, popup_surface],
        3.0,
    );
    let range_track = ensure_contrast(muted, surface_alt, 2.6);
    let landmark_warm = ensure_contrast_multi((253, 230, 138), &[top, bottom], 2.1);
    let landmark_cool = ensure_contrast_multi((147, 197, 253), &[top, bottom], 2.1);
    let landmark_neutral = ensure_contrast_multi(muted, &[top, bottom], 2.0);
    let temp_freezing = ensure_contrast((147, 197, 253), surface_alt, 2.3);
    let temp_cold = ensure_contrast((56, 189, 248), surface_alt, 2.3);
    let temp_mild = ensure_contrast((110, 231, 183), surface_alt, 2.3);
    let temp_warm = ensure_contrast((251, 191, 36), surface_alt, 2.3);
    let temp_hot = ensure_contrast((248, 113, 113), surface_alt, 2.3);

    Theme {
        top: quantize(Color::Rgb(top.0, top.1, top.2), capability),
        bottom: quantize(Color::Rgb(bottom.0, bottom.1, bottom.2), capability),
        surface: quantize(Color::Rgb(surface.0, surface.1, surface.2), capability),
        surface_alt: quantize(
            Color::Rgb(surface_alt.0, surface_alt.1, surface_alt.2),
            capability,
        ),
        popup_surface: quantize(
            Color::Rgb(popup_surface.0, popup_surface.1, popup_surface.2),
            capability,
        ),
        accent: quantize(Color::Rgb(accent.0, accent.1, accent.2), capability),
        text: quantize(Color::Rgb(text.0, text.1, text.2), capability),
        muted_text: quantize(Color::Rgb(muted.0, muted.1, muted.2), capability),
        popup_text: quantize(
            Color::Rgb(popup_text.0, popup_text.1, popup_text.2),
            capability,
        ),
        popup_muted_text: quantize(
            Color::Rgb(popup_muted_text.0, popup_muted_text.1, popup_muted_text.2),
            capability,
        ),
        particle: quantize(Color::Rgb(particle.0, particle.1, particle.2), capability),
        border: quantize(Color::Rgb(border.0, border.1, border.2), capability),
        popup_border: quantize(
            Color::Rgb(popup_border.0, popup_border.1, popup_border.2),
            capability,
        ),
        info: quantize(Color::Rgb(info.0, info.1, info.2), capability),
        success: quantize(Color::Rgb(success.0, success.1, success.2), capability),
        warning: quantize(Color::Rgb(warning.0, warning.1, warning.2), capability),
        danger: quantize(Color::Rgb(danger.0, danger.1, danger.2), capability),
        temp_freezing: quantize(
            Color::Rgb(temp_freezing.0, temp_freezing.1, temp_freezing.2),
            capability,
        ),
        temp_cold: quantize(
            Color::Rgb(temp_cold.0, temp_cold.1, temp_cold.2),
            capability,
        ),
        temp_mild: quantize(
            Color::Rgb(temp_mild.0, temp_mild.1, temp_mild.2),
            capability,
        ),
        temp_warm: quantize(
            Color::Rgb(temp_warm.0, temp_warm.1, temp_warm.2),
            capability,
        ),
        temp_hot: quantize(Color::Rgb(temp_hot.0, temp_hot.1, temp_hot.2), capability),
        range_track: quantize(
            Color::Rgb(range_track.0, range_track.1, range_track.2),
            capability,
        ),
        landmark_warm: quantize(
            Color::Rgb(landmark_warm.0, landmark_warm.1, landmark_warm.2),
            capability,
        ),
        landmark_cool: quantize(
            Color::Rgb(landmark_cool.0, landmark_cool.1, landmark_cool.2),
            capability,
        ),
        landmark_neutral: quantize(
            Color::Rgb(landmark_neutral.0, landmark_neutral.1, landmark_neutral.2),
            capability,
        ),
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

fn mix_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| -> u8 {
        (f32::from(x) + (f32::from(y) - f32::from(x)) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

fn ensure_contrast(fg: (u8, u8, u8), bg: (u8, u8, u8), min_ratio: f32) -> (u8, u8, u8) {
    ensure_contrast_multi(fg, &[bg], min_ratio)
}

fn ensure_contrast_multi(
    fg: (u8, u8, u8),
    backgrounds: &[(u8, u8, u8)],
    min_ratio: f32,
) -> (u8, u8, u8) {
    if backgrounds.is_empty() {
        return fg;
    }
    if min_contrast_ratio(fg, backgrounds) >= min_ratio {
        return fg;
    }

    let black = (0, 0, 0);
    let white = (255, 255, 255);
    let black_score = min_contrast_ratio(black, backgrounds);
    let white_score = min_contrast_ratio(white, backgrounds);
    let target = if white_score >= black_score {
        white
    } else {
        black
    };

    let mut best = fg;
    let mut best_ratio = min_contrast_ratio(fg, backgrounds);
    for step in 1..=24 {
        let t = step as f32 / 24.0;
        let candidate = mix_rgb(fg, target, t);
        let ratio = min_contrast_ratio(candidate, backgrounds);
        if ratio > best_ratio {
            best = candidate;
            best_ratio = ratio;
        }
        if ratio >= min_ratio {
            return candidate;
        }
    }
    best
}

fn min_contrast_ratio(color: (u8, u8, u8), backgrounds: &[(u8, u8, u8)]) -> f32 {
    backgrounds
        .iter()
        .map(|bg| contrast_ratio(color, *bg))
        .fold(f32::INFINITY, f32::min)
}

fn contrast_ratio(a: (u8, u8, u8), b: (u8, u8, u8)) -> f32 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

fn relative_luminance(rgb: (u8, u8, u8)) -> f32 {
    let r = srgb_to_linear(rgb.0);
    let g = srgb_to_linear(rgb.1);
    let b = srgb_to_linear(rgb.2);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn srgb_to_linear(v: u8) -> f32 {
    let s = f32::from(v) / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic16_explicit_themes_are_distinct() {
        let aurora = theme_for(
            WeatherCategory::Clear,
            true,
            ColorCapability::Basic16,
            ThemeArg::Aurora,
        );
        let mono = theme_for(
            WeatherCategory::Clear,
            true,
            ColorCapability::Basic16,
            ThemeArg::Mono,
        );

        assert!(
            aurora.surface != mono.surface
                || aurora.accent != mono.accent
                || aurora.border != mono.border
        );
    }
}
