use super::data::ALL_NON_AUTO_THEMES;
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
