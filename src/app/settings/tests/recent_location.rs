use super::*;

#[test]
fn same_place_identical_returns_true() {
    let a = stockholm_recent_location();
    let b = stockholm_recent_location();
    assert!(a.same_place(&b));
}

#[test]
fn same_place_coordinates_within_tolerance_returns_true() {
    let a = stockholm_recent_location();
    let b = RecentLocation {
        latitude: a.latitude + 0.04,
        longitude: a.longitude - 0.04,
        ..stockholm_recent_location()
    };
    assert!(a.same_place(&b));
}

#[test]
fn same_place_both_missing_country_returns_true() {
    let a = RecentLocation {
        country: None,
        ..stockholm_recent_location()
    };
    let b = RecentLocation {
        country: None,
        ..stockholm_recent_location()
    };
    assert!(a.same_place(&b));
}

#[test]
fn same_place_missing_country_handled_as_empty_string() {
    let a = RecentLocation {
        country: None,
        ..stockholm_recent_location()
    };
    let b = RecentLocation {
        country: Some(String::new()),
        ..stockholm_recent_location()
    };
    assert!(a.same_place(&b));
}

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
fn display_name_admin_only_returns_name_only() {
    let loc = RecentLocation {
        name: "London".to_string(),
        latitude: 51.5,
        longitude: -0.1,
        country: None,
        admin1: Some("Greater London".to_string()),
        timezone: None,
    };
    assert_eq!(loc.display_name(), "London");
}

#[test]
fn from_location_copies_all_fields() {
    let location = crate::test_support::stockholm_location();

    let recent = RecentLocation::from_location(&location);

    assert_eq!(recent.name, location.name);
    assert!((recent.latitude - location.latitude).abs() < f64::EPSILON);
    assert!((recent.longitude - location.longitude).abs() < f64::EPSILON);
    assert_eq!(recent.country, location.country);
    assert_eq!(recent.admin1, location.admin1);
    assert_eq!(recent.timezone, location.timezone);
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
fn recent_location_to_location_preserves_fields() {
    let recent = RecentLocation {
        name: "Test City".to_string(),
        latitude: 12.34,
        longitude: 56.78,
        country: Some("Test Country".to_string()),
        admin1: Some("Test Admin".to_string()),
        timezone: Some("Test/Zone".to_string()),
    };

    let location = recent.to_location();

    assert_eq!(location.name, recent.name);
    assert!((location.latitude - recent.latitude).abs() < f64::EPSILON);
    assert!((location.longitude - recent.longitude).abs() < f64::EPSILON);
    assert_eq!(location.country, recent.country);
    assert_eq!(location.admin1, recent.admin1);
    assert_eq!(location.timezone, recent.timezone);
    assert_eq!(location.population, None);
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
