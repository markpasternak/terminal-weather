use super::*;

fn assert_approx_eq(left: f32, right: f32) {
    assert!((left - right).abs() < f32::EPSILON);
}

#[test]
fn convert_temp_to_celsius_returns_same_value() {
    assert_approx_eq(convert_temp(0.0, Units::Celsius), 0.0);
    assert_approx_eq(convert_temp(100.0, Units::Celsius), 100.0);
    assert_approx_eq(convert_temp(-40.0, Units::Celsius), -40.0);
}

#[test]
fn convert_temp_to_fahrenheit_calculates_correctly() {
    assert_approx_eq(convert_temp(0.0, Units::Fahrenheit), 32.0);
    assert_approx_eq(convert_temp(100.0, Units::Fahrenheit), 212.0);
    assert_approx_eq(convert_temp(-40.0, Units::Fahrenheit), -40.0);
}

#[test]
fn wind_speed_conversion_rounding_covers_negative_and_fractional_values() {
    assert_approx_eq(convert_wind_speed(0.0), 0.0);
    assert_approx_eq(convert_wind_speed(3.6), 1.0);
    assert_approx_eq(convert_wind_speed(36.0), 10.0);
    assert_approx_eq(convert_wind_speed(-18.0), -5.0);

    assert_eq!(round_wind_speed(0.0), 0);
    assert_eq!(round_wind_speed(3.6), 1);
    assert_eq!(round_wind_speed(5.4), 2);
    assert_eq!(round_wind_speed(7.2), 2);
    assert_eq!(round_wind_speed(54.0), 15);
    assert_eq!(round_wind_speed(-3.6), -1);
    assert_eq!(round_wind_speed(-5.4), -2);
}
