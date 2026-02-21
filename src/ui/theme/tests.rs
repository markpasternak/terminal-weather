use super::data::{ALL_NON_AUTO_THEMES, AUTO_THEME_SEEDS};
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
        auto_basic16_gradient(WeatherCategory::Clear, true),
        ((13, 53, 102), (30, 102, 158))
    );
    assert_eq!(
        auto_basic16_gradient(WeatherCategory::Rain, true),
        ((17, 47, 88), (32, 73, 126))
    );
    assert_eq!(
        auto_basic16_gradient(WeatherCategory::Unknown, false),
        ((19, 24, 35), (31, 39, 53))
    );
}

#[test]
fn auto_basic16_clear_day_and_night_are_distinct() {
    let day = theme_for(
        WeatherCategory::Clear,
        true,
        ColorCapability::Basic16,
        ThemeArg::Auto,
    );
    let night = theme_for(
        WeatherCategory::Clear,
        false,
        ColorCapability::Basic16,
        ThemeArg::Auto,
    );
    assert_ne!(day.accent, night.accent);
}

#[test]
fn auto_theme_seed_matrix_is_complete_and_unique() {
    let categories = [
        WeatherCategory::Clear,
        WeatherCategory::Cloudy,
        WeatherCategory::Rain,
        WeatherCategory::Snow,
        WeatherCategory::Fog,
        WeatherCategory::Thunder,
        WeatherCategory::Unknown,
    ];

    for category in categories {
        for is_day in [true, false] {
            let count = AUTO_THEME_SEEDS
                .iter()
                .filter(|((candidate_category, candidate_is_day), _)| {
                    *candidate_category == category && *candidate_is_day == is_day
                })
                .count();
            assert_eq!(
                count, 1,
                "expected one AUTO_THEME_SEEDS entry for {category:?}, is_day={is_day}"
            );
        }
    }
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
    for mode in ALL_NON_AUTO_THEMES {
        assert_mode_readable_contrast(*mode);
    }
}

fn assert_mode_readable_contrast(mode: ThemeArg) {
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

    let text_ratio = min_contrast_ratio(as_rgb(theme.text), &backgrounds);
    assert!(
        text_ratio >= 4.5,
        "mode={mode:?} text ratio={text_ratio:.2} < 4.5"
    );

    let accent_ratio = min_contrast_ratio(as_rgb(theme.accent), &backgrounds);
    assert!(
        accent_ratio >= 3.0,
        "mode={mode:?} accent ratio={accent_ratio:.2} < 3.0"
    );

    let popup_ratio = contrast_ratio(as_rgb(theme.popup_text), as_rgb(theme.popup_surface));
    assert!(
        popup_ratio >= 4.5,
        "mode={mode:?} popup ratio={popup_ratio:.2} < 4.5"
    );

    if let Some(distance) = warning_accent_distance(theme) {
        assert!(
            distance >= 50.0,
            "mode={mode:?} warning≈accent dist={distance:.1}"
        );
    }
}

fn warning_accent_distance(theme: Theme) -> Option<f32> {
    let warning = as_rgb(theme.warning);
    let accent = as_rgb(theme.accent);
    let both_washed = relative_luminance(warning) > 0.75 && relative_luminance(accent) > 0.75;
    if both_washed {
        return None;
    }
    Some(
        ((warning.0 as f32 - accent.0 as f32).powi(2)
            + (warning.1 as f32 - accent.1 as f32).powi(2)
            + (warning.2 as f32 - accent.2 as f32).powi(2))
        .sqrt(),
    )
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
fn detect_color_capability_respects_mode_and_env_overrides() {
    let cases = [
        (
            ColorArg::Auto,
            Some("xterm-256color"),
            Some("truecolor"),
            Some(""),
            ColorCapability::TrueColor,
        ),
        (
            ColorArg::Auto,
            Some("xterm-256color"),
            Some("truecolor"),
            Some("1"),
            ColorCapability::Basic16,
        ),
        (
            ColorArg::Always,
            Some("xterm-256color"),
            Some("24bit"),
            Some("1"),
            ColorCapability::TrueColor,
        ),
        (
            ColorArg::Auto,
            Some("xterm-direct"),
            None,
            None,
            ColorCapability::TrueColor,
        ),
        (
            ColorArg::Never,
            Some("xterm-256color"),
            Some("truecolor"),
            None,
            ColorCapability::Basic16,
        ),
    ];

    for (mode, term, colorterm, no_color, expected) in cases {
        let capability = detect_color_capability_from(mode, term, colorterm, no_color);
        assert_eq!(capability, expected);
    }
}

#[test]
fn temp_color_covers_all_five_bands() {
    let theme = theme_for(
        WeatherCategory::Clear,
        true,
        ColorCapability::TrueColor,
        ThemeArg::Nord,
    );
    // Each branch: <= -8, <= 2, <= 16, <= 28, > 28
    let freezing = temp_color(&theme, -10.0);
    let cold = temp_color(&theme, 0.0);
    let mild = temp_color(&theme, 10.0);
    let warm = temp_color(&theme, 22.0);
    let hot = temp_color(&theme, 35.0);

    // All should return a Color (just verify they differ)
    assert_ne!(freezing, hot);
    assert_ne!(cold, warm);
    assert_ne!(mild, hot);
    // Boundary checks — result should equal the expected field
    assert_eq!(freezing, theme.temp_freezing);
    assert_eq!(cold, theme.temp_cold);
    assert_eq!(mild, theme.temp_mild);
    assert_eq!(warm, theme.temp_warm);
    assert_eq!(hot, theme.temp_hot);
}

#[test]
fn quantize_basic16_covers_achromatic_and_hue_paths() {
    // Very dark color → Black (achromatic, light < 0.20)
    assert_eq!(
        quantize(Color::Rgb(10, 10, 10), ColorCapability::Basic16),
        Color::Black
    );
    // Dark-gray range (light 0.20..0.40)
    assert_eq!(
        quantize(Color::Rgb(70, 70, 70), ColorCapability::Basic16),
        Color::DarkGray
    );
    // Gray range (light 0.40..0.72)
    assert_eq!(
        quantize(Color::Rgb(140, 140, 140), ColorCapability::Basic16),
        Color::Gray
    );
    // White (light >= 0.72)
    assert_eq!(
        quantize(Color::Rgb(220, 220, 220), ColorCapability::Basic16),
        Color::White
    );
    // Red hue (max=R, hue near 0)
    let red = quantize(Color::Rgb(200, 50, 50), ColorCapability::Basic16);
    assert!(matches!(red, Color::Red | Color::LightRed));
    // Green hue (max=G)
    let green = quantize(Color::Rgb(50, 200, 50), ColorCapability::Basic16);
    assert!(matches!(green, Color::Green | Color::LightGreen));
    // Blue hue (max=B)
    let blue = quantize(Color::Rgb(50, 50, 200), ColorCapability::Basic16);
    assert!(matches!(blue, Color::Blue | Color::LightBlue));
}

#[test]
fn quantize_truecolor_passes_through() {
    let color = Color::Rgb(123, 45, 67);
    assert_eq!(quantize(color, ColorCapability::TrueColor), color);
}
