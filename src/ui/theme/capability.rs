use super::*;

pub(super) fn detect_color_capability_from(
    mode: ColorArg,
    term: Option<&str>,
    colorterm: Option<&str>,
    no_color: Option<&str>,
) -> ColorCapability {
    if should_force_basic16(mode, term, no_color) {
        return ColorCapability::Basic16;
    }
    if supports_truecolor(colorterm) {
        return ColorCapability::TrueColor;
    }
    if supports_256_color(term) {
        ColorCapability::Xterm256
    } else {
        ColorCapability::Basic16
    }
}

fn should_force_basic16(mode: ColorArg, term: Option<&str>, no_color: Option<&str>) -> bool {
    mode == ColorArg::Never
        || (mode == ColorArg::Auto && no_color.is_some_and(|value| !value.is_empty()))
        || term.is_some_and(|value| value.eq_ignore_ascii_case("dumb"))
}

fn supports_truecolor(colorterm: Option<&str>) -> bool {
    let colorterm = colorterm.unwrap_or_default().to_lowercase();
    colorterm.contains("truecolor") || colorterm.contains("24bit")
}

fn supports_256_color(term: Option<&str>) -> bool {
    term.unwrap_or_default().to_lowercase().contains("256color")
}
