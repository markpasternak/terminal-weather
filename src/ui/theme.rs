use ratatui::style::Color;

use crate::{
    cli::{ColorArg, ThemeArg},
    domain::weather::WeatherCategory,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    TrueColor,
    Xterm256,
    Basic16,
}

type Rgb = (u8, u8, u8);
type ThemeSeed = (Rgb, Rgb, Rgb);
type Basic16Palette = (Color, Color, Color, Color, Color, Color);

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

pub fn detect_color_capability(mode: ColorArg) -> ColorCapability {
    let term = std::env::var("TERM").ok();
    let colorterm = std::env::var("COLORTERM").ok();
    let no_color = std::env::var("NO_COLOR").ok();
    detect_color_capability_from(
        mode,
        term.as_deref(),
        colorterm.as_deref(),
        no_color.as_deref(),
    )
}

fn detect_color_capability_from(
    mode: ColorArg,
    term: Option<&str>,
    colorterm: Option<&str>,
    no_color: Option<&str>,
) -> ColorCapability {
    if mode == ColorArg::Never {
        return ColorCapability::Basic16;
    }
    if mode == ColorArg::Auto && no_color.is_some_and(|value| !value.is_empty()) {
        return ColorCapability::Basic16;
    }
    if term.is_some_and(|value| value.eq_ignore_ascii_case("dumb")) {
        return ColorCapability::Basic16;
    }

    let colorterm = colorterm.unwrap_or_default().to_lowercase();
    if colorterm.contains("truecolor") || colorterm.contains("24bit") {
        return ColorCapability::TrueColor;
    }

    let term = term.unwrap_or_default().to_lowercase();
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
    let (top, bottom, accent_seed) = match mode {
        ThemeArg::Auto => auto_theme_seed(category, is_day),
        _ => preset_theme_seed(mode),
    };

    if capability == ColorCapability::Basic16 {
        return theme_for_basic16(mode, category, top, bottom, capability);
    }

    theme_for_extended(top, bottom, accent_seed, capability)
}

fn auto_theme_seed(category: WeatherCategory, is_day: bool) -> ThemeSeed {
    match (category, is_day) {
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
    }
}

const PRESET_THEME_SEEDS: &[(ThemeArg, ThemeSeed)] = &[
    (
        ThemeArg::Aurora,
        ((9, 31, 65), (16, 70, 105), (102, 232, 242)),
    ),
    // Dark navy + cyan + amber preset inspired by polished terminal mockups.
    (
        ThemeArg::MidnightCyan,
        ((10, 14, 42), (28, 22, 84), (122, 230, 255)),
    ),
    // Workspace-inspired presets.
    (
        ThemeArg::Aubergine,
        ((46, 24, 73), (82, 36, 114), (106, 212, 243)),
    ),
    (
        ThemeArg::Hoth,
        ((229, 236, 247), (204, 218, 236), (32, 109, 167)),
    ),
    (
        ThemeArg::Monument,
        ((17, 33, 33), (33, 58, 57), (242, 176, 68)),
    ),
    (
        ThemeArg::Nord,
        ((46, 52, 64), (59, 66, 82), (136, 192, 208)),
    ),
    (
        ThemeArg::CatppuccinMocha,
        ((30, 30, 46), (49, 50, 68), (203, 166, 247)),
    ),
    (
        ThemeArg::Mono,
        ((17, 17, 24), (32, 35, 44), (196, 201, 214)),
    ),
    (
        ThemeArg::HighContrast,
        ((0, 0, 0), (10, 10, 16), (255, 210, 0)),
    ),
    (
        ThemeArg::Dracula,
        ((40, 42, 54), (68, 71, 90), (189, 147, 249)),
    ),
    (
        ThemeArg::GruvboxMaterialDark,
        ((40, 40, 40), (60, 56, 54), (216, 166, 87)),
    ),
    (
        ThemeArg::KanagawaWave,
        ((31, 31, 40), (43, 46, 58), (152, 187, 108)),
    ),
    (
        ThemeArg::AyuMirage,
        ((31, 36, 48), (46, 53, 71), (255, 173, 102)),
    ),
    (
        ThemeArg::AyuLight,
        ((248, 249, 250), (232, 236, 242), (255, 148, 64)),
    ),
    (
        ThemeArg::PoimandresStorm,
        ((37, 43, 55), (56, 65, 84), (209, 159, 232)),
    ),
    (
        ThemeArg::SelenizedDark,
        ((16, 60, 72), (24, 73, 86), (90, 170, 255)),
    ),
    (
        ThemeArg::NoClownFiesta,
        ((16, 16, 16), (33, 37, 45), (179, 168, 241)),
    ),
];

const BASIC16_MODE_PALETTES: &[(ThemeArg, Basic16Palette)] = &[
    (
        ThemeArg::Aurora,
        (
            Color::Blue,
            Color::Cyan,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightCyan,
            Color::White,
        ),
    ),
    (
        ThemeArg::MidnightCyan,
        (
            Color::Blue,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightCyan,
            Color::Yellow,
        ),
    ),
    (
        ThemeArg::Aubergine,
        (
            Color::Magenta,
            Color::Blue,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightMagenta,
            Color::Yellow,
        ),
    ),
    (
        ThemeArg::Hoth,
        (
            Color::Gray,
            Color::White,
            Color::DarkGray,
            Color::Blue,
            Color::Blue,
            Color::Black,
        ),
    ),
    (
        ThemeArg::Monument,
        (
            Color::Black,
            Color::DarkGray,
            Color::DarkGray,
            Color::Yellow,
            Color::LightGreen,
            Color::White,
        ),
    ),
    (
        ThemeArg::Nord,
        (
            Color::Blue,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightBlue,
            Color::White,
        ),
    ),
    (
        ThemeArg::CatppuccinMocha,
        (
            Color::Magenta,
            Color::Blue,
            Color::DarkGray,
            Color::LightMagenta,
            Color::LightCyan,
            Color::White,
        ),
    ),
    (
        ThemeArg::Mono,
        (
            Color::Black,
            Color::DarkGray,
            Color::DarkGray,
            Color::White,
            Color::Gray,
            Color::White,
        ),
    ),
    (
        ThemeArg::HighContrast,
        (
            Color::Black,
            Color::Black,
            Color::Black,
            Color::Yellow,
            Color::White,
            Color::Yellow,
        ),
    ),
    (
        ThemeArg::Dracula,
        (
            Color::Magenta,
            Color::Blue,
            Color::DarkGray,
            Color::LightMagenta,
            Color::LightMagenta,
            Color::White,
        ),
    ),
    (
        ThemeArg::GruvboxMaterialDark,
        (
            Color::Black,
            Color::DarkGray,
            Color::DarkGray,
            Color::Yellow,
            Color::Yellow,
            Color::White,
        ),
    ),
    (
        ThemeArg::KanagawaWave,
        (
            Color::Green,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightGreen,
            Color::LightGreen,
            Color::White,
        ),
    ),
    (
        ThemeArg::AyuMirage,
        (
            Color::Yellow,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightYellow,
            Color::Yellow,
            Color::White,
        ),
    ),
    (
        ThemeArg::AyuLight,
        (
            Color::Gray,
            Color::White,
            Color::DarkGray,
            Color::Red,
            Color::Yellow,
            Color::Black,
        ),
    ),
    (
        ThemeArg::PoimandresStorm,
        (
            Color::Magenta,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightMagenta,
            Color::LightMagenta,
            Color::White,
        ),
    ),
    (
        ThemeArg::SelenizedDark,
        (
            Color::Cyan,
            Color::Blue,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightBlue,
            Color::White,
        ),
    ),
    (
        ThemeArg::NoClownFiesta,
        (
            Color::Black,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightMagenta,
            Color::Magenta,
            Color::White,
        ),
    ),
];

fn lookup_theme_entry<T: Copy>(entries: &[(ThemeArg, T)], mode: ThemeArg) -> T {
    debug_assert!(mode != ThemeArg::Auto, "auto mode handled separately");
    for (candidate, value) in entries {
        if *candidate == mode {
            return *value;
        }
    }
    unreachable!("missing theme mapping for {:?}", mode)
}

fn preset_theme_seed(mode: ThemeArg) -> ThemeSeed {
    lookup_theme_entry(PRESET_THEME_SEEDS, mode)
}

fn theme_for_basic16(
    mode: ThemeArg,
    category: WeatherCategory,
    top: (u8, u8, u8),
    bottom: (u8, u8, u8),
    capability: ColorCapability,
) -> Theme {
    if mode == ThemeArg::Auto {
        let (top, bottom) = auto_basic16_gradient(category);

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

    let (surface, surface_alt, popup_surface, accent, border, popup_border) =
        basic16_mode_palette(mode);
    let semantics = basic16_semantics(matches!(mode, ThemeArg::AyuLight | ThemeArg::Hoth));

    Theme {
        top: quantize(Color::Rgb(top.0, top.1, top.2), capability),
        bottom: quantize(Color::Rgb(bottom.0, bottom.1, bottom.2), capability),
        surface,
        surface_alt,
        popup_surface,
        accent,
        text: semantics.text,
        muted_text: semantics.muted,
        popup_text: semantics.text,
        popup_muted_text: semantics.muted,
        particle: semantics.particle,
        border,
        popup_border,
        info: semantics.info,
        success: semantics.success,
        warning: semantics.warning,
        danger: semantics.danger,
        temp_freezing: semantics.temp_freezing,
        temp_cold: semantics.temp_cold,
        temp_mild: semantics.temp_mild,
        temp_warm: semantics.temp_warm,
        temp_hot: semantics.temp_hot,
        range_track: semantics.muted,
        landmark_warm: semantics.landmark_warm,
        landmark_cool: semantics.landmark_cool,
        landmark_neutral: semantics.muted,
    }
}

#[derive(Debug, Clone, Copy)]
struct Basic16Semantics {
    text: Color,
    muted: Color,
    particle: Color,
    info: Color,
    success: Color,
    warning: Color,
    danger: Color,
    temp_freezing: Color,
    temp_cold: Color,
    temp_mild: Color,
    temp_warm: Color,
    temp_hot: Color,
    landmark_warm: Color,
    landmark_cool: Color,
}

fn auto_basic16_gradient(category: WeatherCategory) -> ((u8, u8, u8), (u8, u8, u8)) {
    let top = (0, 0, 0);
    let bottom = match category {
        WeatherCategory::Clear => (0, 32, 72),
        WeatherCategory::Cloudy => (25, 30, 35),
        WeatherCategory::Rain => (0, 22, 56),
        WeatherCategory::Snow => (28, 38, 56),
        WeatherCategory::Fog => (30, 30, 30),
        WeatherCategory::Thunder => (24, 0, 44),
        WeatherCategory::Unknown => (20, 24, 32),
    };
    (top, bottom)
}

fn basic16_mode_palette(mode: ThemeArg) -> (Color, Color, Color, Color, Color, Color) {
    lookup_theme_entry(BASIC16_MODE_PALETTES, mode)
}

fn basic16_semantics(is_light_theme: bool) -> Basic16Semantics {
    let text = if is_light_theme {
        Color::Black
    } else {
        Color::White
    };
    let muted = if is_light_theme {
        Color::DarkGray
    } else {
        Color::Gray
    };
    let particle = if is_light_theme {
        Color::Gray
    } else {
        Color::DarkGray
    };
    let info = if is_light_theme {
        Color::Blue
    } else {
        Color::LightCyan
    };
    let success = if is_light_theme {
        Color::Green
    } else {
        Color::LightGreen
    };
    let warning = if is_light_theme {
        Color::Magenta
    } else {
        Color::Yellow
    };
    let danger = if is_light_theme {
        Color::Red
    } else {
        Color::LightRed
    };
    let temp_freezing = if is_light_theme {
        Color::Blue
    } else {
        Color::LightBlue
    };
    let temp_warm = if is_light_theme {
        Color::Magenta
    } else {
        Color::Yellow
    };
    let temp_hot = if is_light_theme {
        Color::Red
    } else {
        Color::LightRed
    };
    let landmark_warm = if is_light_theme {
        Color::Magenta
    } else {
        Color::Yellow
    };
    let landmark_cool = if is_light_theme {
        Color::Blue
    } else {
        Color::LightBlue
    };

    Basic16Semantics {
        text,
        muted,
        particle,
        info,
        success,
        warning,
        danger,
        temp_freezing,
        temp_cold: Color::Cyan,
        temp_mild: Color::Green,
        temp_warm,
        temp_hot,
        landmark_warm,
        landmark_cool,
    }
}

fn theme_for_extended(
    mut top: (u8, u8, u8),
    mut bottom: (u8, u8, u8),
    accent_seed: (u8, u8, u8),
    capability: ColorCapability,
) -> Theme {
    let avg_luma = (luma(top.0, top.1, top.2) + luma(bottom.0, bottom.1, bottom.2)) / 2.0;
    let dark_text = avg_luma >= 170.0;

    (top, bottom) = adjust_light_gradients(top, bottom, dark_text);

    let base_surface = mix_rgb(top, bottom, 0.80);
    let base_surface_alt = mix_rgb(top, bottom, 0.60);
    let (tint_factor, tint_factor_alt) = surface_tint_factors(dark_text, avg_luma);
    let surface = mix_rgb(base_surface, accent_seed, tint_factor);
    let surface_alt = mix_rgb(base_surface_alt, accent_seed, tint_factor_alt);
    let popup_surface = choose_rgb(
        dark_text,
        mix_rgb(surface_alt, accent_seed, 0.20),
        mix_rgb(surface_alt, (236, 243, 251), 0.18),
    );

    let all_backgrounds = [surface, surface_alt, popup_surface, top, bottom];
    let hero_backgrounds = [top, bottom, surface];

    let text_seed = choose_rgb(dark_text, (12, 16, 24), (240, 245, 250));
    let muted_seed = choose_rgb(dark_text, (55, 68, 85), (183, 198, 214));
    let text = ensure_contrast_multi(
        text_seed,
        &all_backgrounds,
        if dark_text { 4.9 } else { 4.7 },
    );
    let muted = ensure_contrast_multi(
        muted_seed,
        &all_backgrounds,
        if dark_text { 4.5 } else { 4.2 },
    );
    let popup_text = ensure_contrast(text_seed, popup_surface, 4.7);
    let popup_muted_text = ensure_contrast(muted_seed, popup_surface, 4.5);
    let accent = ensure_contrast_multi(
        accent_seed,
        &all_backgrounds,
        if dark_text { 4.5 } else { 4.0 },
    );

    let particle = choose_rgb(dark_text, (92, 108, 124), (202, 218, 235));
    let border_seed = choose_rgb(
        dark_text,
        mix_rgb(surface, (18, 26, 38), 0.74),
        mix_rgb(surface, accent, 0.54),
    );
    let border = ensure_contrast_multi(border_seed, &[surface, surface_alt, top, bottom], 3.0);
    let popup_border_seed = choose_rgb(
        dark_text,
        mix_rgb(popup_surface, (5, 11, 18), 0.82),
        mix_rgb(popup_surface, accent, 0.70),
    );
    let popup_border = ensure_contrast(popup_border_seed, popup_surface, 3.2);

    let info = ensure_contrast_multi(
        choose_rgb(dark_text, (3, 105, 161), (125, 211, 252)),
        &all_backgrounds,
        4.5,
    );
    let success = ensure_contrast_multi(
        choose_rgb(dark_text, (21, 128, 61), (74, 222, 128)),
        &all_backgrounds,
        4.5,
    );
    let warning_seed = warning_seed_for(accent_seed, dark_text);
    let warning = ensure_contrast_multi(warning_seed, &all_backgrounds, 4.5);
    let danger = ensure_contrast_multi(
        choose_rgb(dark_text, (185, 28, 28), (248, 113, 113)),
        &all_backgrounds,
        4.5,
    );
    let range_track = ensure_contrast(muted, surface_alt, if dark_text { 4.0 } else { 3.2 });
    let landmark_warm = ensure_contrast_multi(
        (253, 230, 138),
        &hero_backgrounds,
        if dark_text { 4.5 } else { 3.5 },
    );
    let landmark_cool = ensure_contrast_multi(
        (147, 197, 253),
        &hero_backgrounds,
        if dark_text { 4.5 } else { 3.5 },
    );
    let landmark_neutral =
        ensure_contrast_multi(muted, &hero_backgrounds, if dark_text { 4.2 } else { 3.2 });
    let temp_freezing = ensure_contrast(
        (147, 197, 253),
        surface_alt,
        if dark_text { 4.5 } else { 3.8 },
    );
    let temp_cold = ensure_contrast(
        (56, 189, 248),
        surface_alt,
        if dark_text { 4.5 } else { 3.8 },
    );
    let temp_mild = ensure_contrast(
        (110, 231, 183),
        surface_alt,
        if dark_text { 4.5 } else { 3.8 },
    );
    let temp_warm = ensure_contrast(
        (251, 191, 36),
        surface_alt,
        if dark_text { 4.5 } else { 3.8 },
    );
    let temp_hot = ensure_contrast(
        (248, 113, 113),
        surface_alt,
        if dark_text { 4.5 } else { 3.8 },
    );

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

fn adjust_light_gradients(
    top: (u8, u8, u8),
    bottom: (u8, u8, u8),
    dark_text: bool,
) -> ((u8, u8, u8), (u8, u8, u8)) {
    if dark_text {
        // Keep light themes readable by pulling gradients away from near-white.
        (
            mix_rgb(top, (198, 210, 226), 0.42),
            mix_rgb(bottom, (176, 193, 214), 0.40),
        )
    } else {
        (top, bottom)
    }
}

fn surface_tint_factors(dark_text: bool, avg_luma: f32) -> (f32, f32) {
    // Reduce accent tint on very dark backgrounds to avoid hue-on-hue illegibility.
    if dark_text || avg_luma < 40.0 {
        (0.08, 0.12)
    } else {
        (0.16, 0.24)
    }
}

fn choose_rgb(condition: bool, when_true: (u8, u8, u8), when_false: (u8, u8, u8)) -> (u8, u8, u8) {
    if condition { when_true } else { when_false }
}

fn warning_seed_for(accent_seed: (u8, u8, u8), dark_text: bool) -> (u8, u8, u8) {
    // Shift warning toward orange-red when accent is already warm/amber to avoid collision.
    let warm_accent = accent_seed.0 > 180 && accent_seed.1 > 140 && accent_seed.2 < 140;
    if warm_accent {
        // Use pink-red so it stays distinct from the amber/gold accent after contrast push.
        choose_rgb(dark_text, (180, 40, 60), (255, 110, 130))
    } else {
        choose_rgb(dark_text, (161, 98, 7), (251, 191, 36))
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
        return achromatic_basic16(light);
    }

    let hue = hue_from_rgb_components(rf, gf, bf, max, delta);
    hue_to_basic16(hue, light >= 0.55)
}

fn achromatic_basic16(light: f32) -> Color {
    if light < 0.20 {
        return Color::Black;
    }
    if light < 0.40 {
        return Color::DarkGray;
    }
    if light < 0.72 {
        return Color::Gray;
    }
    Color::White
}

fn hue_from_rgb_components(rf: f32, gf: f32, bf: f32, max: f32, delta: f32) -> f32 {
    if (max - rf).abs() < f32::EPSILON {
        return 60.0 * ((gf - bf) / delta).rem_euclid(6.0);
    }
    if (max - gf).abs() < f32::EPSILON {
        return 60.0 * (((bf - rf) / delta) + 2.0);
    }
    60.0 * (((rf - gf) / delta) + 4.0)
}

fn hue_to_basic16(hue: f32, bright: bool) -> Color {
    let band = if !(30.0..330.0).contains(&hue) {
        0
    } else if hue < 90.0 {
        1
    } else if hue < 150.0 {
        2
    } else if hue < 210.0 {
        3
    } else if hue < 270.0 {
        4
    } else {
        5
    };
    hue_band_color(band, bright)
}

fn hue_band_color(band: usize, bright: bool) -> Color {
    const DIM: [Color; 6] = [
        Color::Red,
        Color::Yellow,
        Color::Green,
        Color::Cyan,
        Color::Blue,
        Color::Magenta,
    ];
    const BRIGHT: [Color; 6] = [
        Color::LightRed,
        Color::LightYellow,
        Color::LightGreen,
        Color::LightCyan,
        Color::LightBlue,
        Color::LightMagenta,
    ];
    if bright { BRIGHT[band] } else { DIM[band] }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn as_rgb(color: Color) -> (u8, u8, u8) {
        match color {
            Color::Rgb(r, g, b) => (r, g, b),
            other => panic!("expected Color::Rgb, got {other:?}"),
        }
    }

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

    #[test]
    fn auto_basic16_gradient_matches_category_map() {
        assert_eq!(
            auto_basic16_gradient(WeatherCategory::Clear),
            ((0, 0, 0), (0, 32, 72))
        );
        assert_eq!(
            auto_basic16_gradient(WeatherCategory::Rain),
            ((0, 0, 0), (0, 22, 56))
        );
        assert_eq!(
            auto_basic16_gradient(WeatherCategory::Unknown),
            ((0, 0, 0), (20, 24, 32))
        );
    }

    #[test]
    fn basic16_light_themes_keep_expected_semantic_polarity() {
        for mode in [ThemeArg::AyuLight, ThemeArg::Hoth] {
            let theme = theme_for(
                WeatherCategory::Cloudy,
                true,
                ColorCapability::Basic16,
                mode,
            );
            assert_eq!(theme.text, Color::Black);
            assert_eq!(theme.muted_text, Color::DarkGray);
            assert_eq!(theme.popup_text, Color::Black);
            assert_eq!(theme.popup_muted_text, Color::DarkGray);
            assert_eq!(theme.info, Color::Blue);
            assert_eq!(theme.warning, Color::Magenta);
            assert_eq!(theme.danger, Color::Red);
            assert_eq!(theme.temp_hot, Color::Red);
            assert_eq!(theme.landmark_cool, Color::Blue);
            assert_eq!(theme.range_track, Color::DarkGray);
        }
    }

    /// Every TrueColor theme must have readable text, accent, and semantic colors
    /// against all background surfaces.
    #[test]
    fn all_themes_have_readable_contrast() {
        let all_modes = [
            ThemeArg::Aurora,
            ThemeArg::MidnightCyan,
            ThemeArg::Aubergine,
            ThemeArg::Hoth,
            ThemeArg::Monument,
            ThemeArg::Nord,
            ThemeArg::CatppuccinMocha,
            ThemeArg::Mono,
            ThemeArg::HighContrast,
            ThemeArg::Dracula,
            ThemeArg::GruvboxMaterialDark,
            ThemeArg::KanagawaWave,
            ThemeArg::AyuMirage,
            ThemeArg::AyuLight,
            ThemeArg::PoimandresStorm,
            ThemeArg::SelenizedDark,
            ThemeArg::NoClownFiesta,
        ];
        for mode in all_modes {
            let theme = theme_for(
                WeatherCategory::Cloudy,
                true,
                ColorCapability::TrueColor,
                mode,
            );
            let backgrounds = [
                as_rgb(theme.top),
                as_rgb(theme.bottom),
                as_rgb(theme.surface),
                as_rgb(theme.surface_alt),
            ];

            // Primary text must be clearly readable (WCAG AA = 4.5)
            let text_ratio = min_contrast_ratio(as_rgb(theme.text), &backgrounds);
            assert!(
                text_ratio >= 4.5,
                "mode={mode:?} text ratio={text_ratio:.2} < 4.5"
            );

            // Accent must be distinguishable (relaxed for decorative use)
            let accent_ratio = min_contrast_ratio(as_rgb(theme.accent), &backgrounds);
            assert!(
                accent_ratio >= 3.0,
                "mode={mode:?} accent ratio={accent_ratio:.2} < 3.0"
            );

            // Popup text on popup surface
            let popup_ratio = contrast_ratio(as_rgb(theme.popup_text), as_rgb(theme.popup_surface));
            assert!(
                popup_ratio >= 4.5,
                "mode={mode:?} popup ratio={popup_ratio:.2} < 4.5"
            );

            // Warning must be visually distinguishable from accent (by color distance).
            // Skip when both are pushed to near-white by the contrast engine (high luminance);
            // in that case the bg contrast carries the distinction.
            let w = as_rgb(theme.warning);
            let a = as_rgb(theme.accent);
            let w_luma = relative_luminance(w);
            let a_luma = relative_luminance(a);
            let both_washed = w_luma > 0.75 && a_luma > 0.75;
            if !both_washed {
                let dist = ((w.0 as f32 - a.0 as f32).powi(2)
                    + (w.1 as f32 - a.1 as f32).powi(2)
                    + (w.2 as f32 - a.2 as f32).powi(2))
                .sqrt();
                assert!(
                    dist >= 50.0,
                    "mode={mode:?} warning≈accent dist={dist:.1} (warning={w:?}, accent={a:?})"
                );
            }
        }
    }

    #[test]
    fn light_themes_keep_semantic_tokens_legible() {
        for mode in [ThemeArg::AyuLight, ThemeArg::Hoth] {
            let theme = theme_for(
                WeatherCategory::Cloudy,
                true,
                ColorCapability::TrueColor,
                mode,
            );
            let backgrounds = [
                as_rgb(theme.top),
                as_rgb(theme.bottom),
                as_rgb(theme.surface),
                as_rgb(theme.surface_alt),
                as_rgb(theme.popup_surface),
            ];

            let checks = [
                (as_rgb(theme.text), 4.8),
                (as_rgb(theme.muted_text), 4.1),
                (as_rgb(theme.accent), 3.9),
                (as_rgb(theme.info), 4.4),
                (as_rgb(theme.success), 4.4),
                (as_rgb(theme.warning), 4.4),
                (as_rgb(theme.danger), 4.4),
                (as_rgb(theme.temp_cold), 3.5),
                (as_rgb(theme.temp_warm), 3.5),
                (as_rgb(theme.temp_hot), 3.5),
                (as_rgb(theme.landmark_cool), 3.4),
                (as_rgb(theme.landmark_warm), 3.4),
            ];

            for (color, minimum) in checks {
                let ratio = min_contrast_ratio(color, &backgrounds);
                assert!(
                    ratio >= minimum,
                    "mode={mode:?} color={color:?} ratio={ratio:.2} minimum={minimum}"
                );
            }

            let popup_ratio = contrast_ratio(as_rgb(theme.popup_text), as_rgb(theme.popup_surface));
            assert!(
                popup_ratio >= 4.7,
                "mode={mode:?} popup_ratio={popup_ratio:.2}"
            );
        }
    }

    #[test]
    fn auto_mode_ignores_empty_no_color() {
        let capability = detect_color_capability_from(
            ColorArg::Auto,
            Some("xterm-256color"),
            Some("truecolor"),
            Some(""),
        );
        assert_eq!(capability, ColorCapability::TrueColor);
    }

    #[test]
    fn auto_mode_honors_non_empty_no_color() {
        let capability = detect_color_capability_from(
            ColorArg::Auto,
            Some("xterm-256color"),
            Some("truecolor"),
            Some("1"),
        );
        assert_eq!(capability, ColorCapability::Basic16);
    }

    #[test]
    fn always_mode_bypasses_no_color() {
        let capability = detect_color_capability_from(
            ColorArg::Always,
            Some("xterm-256color"),
            Some("24bit"),
            Some("1"),
        );
        assert_eq!(capability, ColorCapability::TrueColor);
    }

    #[test]
    fn never_mode_forces_basic16() {
        let capability = detect_color_capability_from(
            ColorArg::Never,
            Some("xterm-256color"),
            Some("truecolor"),
            None,
        );
        assert_eq!(capability, ColorCapability::Basic16);
    }
}
