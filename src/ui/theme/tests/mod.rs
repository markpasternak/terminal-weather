use ratatui::style::Color;

fn as_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        other => panic!("expected Color::Rgb, got {other:?}"),
    }
}

fn warning_accent_distance(theme: super::Theme) -> Option<f32> {
    let warning = as_rgb(theme.warning);
    let accent = as_rgb(theme.accent);
    let both_washed =
        super::relative_luminance(warning) > 0.75 && super::relative_luminance(accent) > 0.75;
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

mod auto;
mod explicit;
