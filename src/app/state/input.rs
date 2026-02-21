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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::settings::{RecentLocation, RuntimeSettings};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn initial_selected_location_returns_none_when_city_provided() {
        let mut cli = crate::test_support::state_test_cli();
        cli.city = Some("Stockholm".to_string());
        let settings = RuntimeSettings::default();
        assert!(initial_selected_location(&cli, &settings).is_none());
    }

    #[test]
    fn initial_selected_location_returns_none_when_demo() {
        let mut cli = crate::test_support::state_test_cli();
        cli.demo = true;
        let settings = RuntimeSettings::default();
        assert!(initial_selected_location(&cli, &settings).is_none());
    }

    #[test]
    fn initial_selected_location_returns_first_recent_when_no_flags() {
        let cli = crate::test_support::state_test_cli();
        let mut settings = RuntimeSettings::default();
        settings.recent_locations.push(RecentLocation {
            name: "Stockholm".to_string(),
            latitude: 59.33,
            longitude: 18.07,
            country: Some("Sweden".to_string()),
            admin1: None,
            timezone: None,
        });
        let loc = initial_selected_location(&cli, &settings);
        assert!(loc.is_some());
        assert_eq!(loc.unwrap().name, "Stockholm");
    }

    #[test]
    fn initial_selected_location_returns_none_when_no_recent() {
        let cli = crate::test_support::state_test_cli();
        let settings = RuntimeSettings::default();
        assert!(initial_selected_location(&cli, &settings).is_none());
    }

    #[test]
    fn is_city_char_accepts_valid_chars() {
        assert!(is_city_char('A'));
        assert!(is_city_char('z'));
        assert!(is_city_char('5'));
        assert!(is_city_char(' '));
        assert!(is_city_char('-'));
        assert!(is_city_char('\''));
        assert!(is_city_char(','));
        assert!(is_city_char('.'));
    }

    #[test]
    fn is_city_char_rejects_special_chars() {
        assert!(!is_city_char('@'));
        assert!(!is_city_char('!'));
        assert!(!is_city_char('$'));
    }

    #[test]
    fn command_char_returns_none_with_modifiers() {
        let k = key(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert!(command_char(k).is_none());

        let k = key(KeyCode::Char('r'), KeyModifiers::ALT);
        assert!(command_char(k).is_none());
    }

    #[test]
    fn command_char_returns_lowercase_for_plain_char() {
        let k = key(KeyCode::Char('R'), KeyModifiers::NONE);
        assert_eq!(command_char(k), Some('r'));
    }

    #[test]
    fn command_char_returns_none_for_non_char_key() {
        let k = key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(command_char(k).is_none());
    }

    #[test]
    fn settings_close_key_matches_esc_s_q() {
        assert!(settings_close_key(KeyCode::Esc));
        assert!(settings_close_key(KeyCode::Char('s')));
        assert!(settings_close_key(KeyCode::Char('Q')));
        assert!(!settings_close_key(KeyCode::Char('x')));
        assert!(!settings_close_key(KeyCode::Enter));
    }
}
