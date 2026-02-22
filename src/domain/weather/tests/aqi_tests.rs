use super::*;

#[test]
fn us_aqi_categories_follow_epa_thresholds() {
    assert_eq!(categorize_us_aqi(40), AirQualityCategory::Good);
    assert_eq!(categorize_us_aqi(75), AirQualityCategory::Moderate);
    assert_eq!(
        categorize_us_aqi(125),
        AirQualityCategory::UnhealthySensitive
    );
    assert_eq!(categorize_us_aqi(180), AirQualityCategory::Unhealthy);
    assert_eq!(categorize_us_aqi(230), AirQualityCategory::VeryUnhealthy);
    assert_eq!(categorize_us_aqi(350), AirQualityCategory::Hazardous);
}

#[test]
fn air_quality_reading_prefers_us_index_when_available() {
    let reading = AirQualityReading::from_indices(Some(57.0), Some(18.0)).expect("aqi reading");
    assert_eq!(reading.us_aqi, Some(57));
    assert_eq!(reading.european_aqi, Some(18));
    assert_eq!(reading.category, AirQualityCategory::Moderate);
}

#[test]
fn european_aqi_categories_cover_full_range() {
    assert_eq!(categorize_european_aqi(10), AirQualityCategory::Good);
    assert_eq!(categorize_european_aqi(30), AirQualityCategory::Moderate);
    assert_eq!(
        categorize_european_aqi(50),
        AirQualityCategory::UnhealthySensitive
    );
    assert_eq!(categorize_european_aqi(70), AirQualityCategory::Unhealthy);
    assert_eq!(
        categorize_european_aqi(90),
        AirQualityCategory::VeryUnhealthy
    );
    assert_eq!(categorize_european_aqi(110), AirQualityCategory::Hazardous);
}

#[test]
fn air_quality_category_label_covers_all_variants() {
    assert_eq!(AirQualityCategory::Good.label(), "Good");
    assert_eq!(AirQualityCategory::Moderate.label(), "Moderate");
    assert_eq!(AirQualityCategory::UnhealthySensitive.label(), "USG");
    assert_eq!(AirQualityCategory::Unhealthy.label(), "Unhealthy");
    assert_eq!(AirQualityCategory::VeryUnhealthy.label(), "Very Unhealthy");
    assert_eq!(AirQualityCategory::Hazardous.label(), "Hazardous");
    assert_eq!(AirQualityCategory::Unknown.label(), "Unknown");
}

#[test]
fn air_quality_reading_returns_none_when_both_indices_absent() {
    assert!(AirQualityReading::from_indices(None, None).is_none());
}

#[test]
fn air_quality_display_value_uses_us_index_first() {
    let reading = AirQualityReading::from_indices(Some(42.0), Some(25.0)).expect("aqi reading");
    assert_eq!(reading.display_value(), "42");
}

#[test]
fn air_quality_display_value_falls_back_to_european() {
    let reading = AirQualityReading::from_indices(None, Some(30.0)).expect("aqi reading");
    assert_eq!(reading.display_value(), "30");
}

#[test]
fn air_quality_categorizes_via_european_when_us_absent() {
    let reading = AirQualityReading::from_indices(None, Some(30.0)).expect("aqi reading");
    assert_eq!(reading.category, AirQualityCategory::Moderate);
}
