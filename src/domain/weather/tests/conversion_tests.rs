use super::*;

#[test]
fn fahrenheit_conversion_rounding() {
    assert_eq!(round_temp(convert_temp(0.0, Units::Fahrenheit)), 32);
    assert_eq!(round_temp(convert_temp(20.0, Units::Fahrenheit)), 68);
}

#[test]
fn wind_speed_conversion_rounding() {
    assert!((convert_wind_speed(36.0) - 10.0).abs() < f32::EPSILON);

    // Test round_wind_speed separately
    assert_eq!(round_wind_speed(0.0), 0);
    assert_eq!(round_wind_speed(3.6), 1); // 1.0 -> 1
    assert_eq!(round_wind_speed(5.4), 2); // 1.5 -> 2
    assert_eq!(round_wind_speed(7.2), 2); // 2.0 -> 2
    assert_eq!(round_wind_speed(54.0), 15); // 15.0 -> 15

    // Negative cases
    assert_eq!(round_wind_speed(-3.6), -1);
    assert_eq!(round_wind_speed(-5.4), -2);
}
