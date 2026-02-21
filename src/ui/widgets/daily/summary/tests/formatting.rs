use super::*;

#[test]
fn average_duration_returns_placeholder_for_zero_count() {
    assert_eq!(average_duration(0.0, 0), "--:--");
}

#[test]
fn average_duration_computes_correctly() {
    assert_eq!(average_duration(7200.0, 2), "01:00");
}

#[test]
fn average_precip_hours_returns_placeholder_for_zero_count() {
    assert_eq!(average_precip_hours(0.0, 0), "--");
}

#[test]
fn average_precip_hours_formats_correctly() {
    assert_eq!(average_precip_hours(6.0, 3), "2.0h/day");
}

#[test]
fn format_day_value_mm_none_returns_placeholder() {
    assert_eq!(format_day_value_mm(None), "--");
}

#[test]
fn format_day_value_mm_some_formats_correctly() {
    let result = format_day_value_mm(Some(("Mon".to_string(), 3.2)));
    assert!(result.contains("Mon"), "got: {result}");
    assert!(result.contains("3.2mm"), "got: {result}");
}

#[test]
fn format_day_value_mps_none_returns_placeholder() {
    assert_eq!(format_day_value_mps(None), "--");
}

#[test]
fn format_day_value_mps_some_formats_in_mps() {
    let result = format_day_value_mps(Some(("Tue".to_string(), 36.0)));
    assert!(result.contains("Tue"), "got: {result}");
    assert!(result.contains("m/s"), "got: {result}");
}

#[test]
fn format_uv_peak_none_returns_placeholder() {
    assert_eq!(format_uv_peak(None), "--");
}

#[test]
fn week_thermal_span_missing_data_returns_placeholder() {
    assert_eq!(week_thermal_span(None, None, Units::Celsius), "--");
    assert_eq!(week_thermal_span(Some(1.0), None, Units::Celsius), "--");
}

#[test]
fn week_thermal_span_celsius_range() {
    let result = week_thermal_span(Some(-5.0), Some(15.0), Units::Celsius);
    assert!(result.contains("-5°"), "got: {result}");
    assert!(result.contains("15°"), "got: {result}");
}

#[test]
fn week_thermal_span_fahrenheit() {
    let result = week_thermal_span(Some(-5.0), Some(15.0), Units::Fahrenheit);
    assert!(result.contains('°'));
}
