use super::*;

#[test]
fn same_place_handles_unicode_case() {
    let a = RecentLocation {
        name: "Åre".to_string(),
        latitude: 63.4,
        longitude: 13.1,
        country: Some("Sweden".to_string()),
        admin1: Some("Jämtland".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
    };
    let b = RecentLocation {
        name: "åre".to_string(),
        latitude: 63.41,
        longitude: 13.11,
        country: Some("sweden".to_string()),
        admin1: Some("Jämtland".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
    };
    assert!(a.same_place(&b));
}

#[test]
fn recent_location_display_name_variants() {
    let full = RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.3,
        longitude: 18.0,
        country: Some("Sweden".to_string()),
        admin1: Some("Stockholm".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
    };
    assert_eq!(full.display_name(), "Stockholm, Stockholm, Sweden");

    let country_only = RecentLocation {
        admin1: None,
        ..full.clone()
    };
    assert_eq!(country_only.display_name(), "Stockholm, Sweden");

    let name_only = RecentLocation {
        country: None,
        ..country_only
    };
    assert_eq!(name_only.display_name(), "Stockholm");
}

#[test]
fn display_name_both_none_returns_name_only() {
    let loc = RecentLocation {
        name: "Tokyo".to_string(),
        latitude: 35.68,
        longitude: 139.69,
        country: None,
        admin1: None,
        timezone: None,
    };
    assert_eq!(loc.display_name(), "Tokyo");
}

#[test]
fn from_location_and_to_location_roundtrip_core_fields() {
    let location = crate::test_support::stockholm_location();
    let recent = RecentLocation::from_location(&location);
    let restored = recent.to_location();
    assert_eq!(restored.name, location.name);
    assert!((restored.latitude - location.latitude).abs() < f64::EPSILON);
    assert!((restored.longitude - location.longitude).abs() < f64::EPSILON);
    assert_eq!(restored.timezone, location.timezone);
}

#[test]
fn same_place_lat_not_close_returns_false() {
    let a = stockholm_recent_location();
    let b = RecentLocation {
        latitude: 59.50,
        ..stockholm_recent_location()
    };
    assert!(!a.same_place(&b));
}

#[test]
fn same_place_lon_not_close_returns_false() {
    let a = stockholm_recent_location();
    let b = RecentLocation {
        longitude: 18.20,
        ..stockholm_recent_location()
    };
    assert!(!a.same_place(&b));
}

#[test]
fn same_place_name_mismatch_returns_false() {
    let a = stockholm_recent_location();
    let b = named_recent_location("Gothenburg");
    assert!(!a.same_place(&b));
}

#[test]
fn same_place_country_mismatch_returns_false() {
    let a = stockholm_recent_location();
    let b = RecentLocation {
        country: Some("Norway".to_string()),
        ..stockholm_recent_location()
    };
    assert!(!a.same_place(&b));
}
