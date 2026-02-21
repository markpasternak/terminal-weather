use super::*;

pub(crate) fn initial_selected_location(cli: &Cli, settings: &RuntimeSettings) -> Option<Location> {
    if cli.city.is_some() || cli.lat.is_some() || cli.lon.is_some() || cli.demo {
        return None;
    }
    settings
        .recent_locations
        .first()
        .map(RecentLocation::to_location)
}

pub(crate) fn is_city_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, ' ' | '-' | '\'' | '’' | ',' | '.')
}

pub(super) fn command_char(key: KeyEvent) -> Option<char> {
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
    {
        return None;
    }
    if let KeyCode::Char(ch) = key.code {
        Some(ch.to_ascii_lowercase())
    } else {
        None
    }
}

fn command_char_matches_keycode(code: KeyCode, target: char) -> bool {
    matches!(code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&target))
}

pub(super) fn settings_close_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Esc)
        || command_char_matches_keycode(code, 's')
        || command_char_matches_keycode(code, 'q')
}
