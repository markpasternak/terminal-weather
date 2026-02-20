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
    if supports_truecolor(colorterm, term) {
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

fn supports_truecolor(colorterm: Option<&str>, term: Option<&str>) -> bool {
    let colorterm = colorterm.unwrap_or_default().to_lowercase();
    let term = term.unwrap_or_default().to_lowercase();
    truecolor_hint(&colorterm) || truecolor_hint(&term)
}

fn truecolor_hint(value: &str) -> bool {
    value.contains("truecolor")
        || value.contains("24bit")
        || value.contains("-direct")
        || value.ends_with("direct")
}

fn supports_256_color(term: Option<&str>) -> bool {
    term.unwrap_or_default().to_lowercase().contains("256color")
}
