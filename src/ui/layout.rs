#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HourlyDensity {
    Full16,
    Full12,
    Compact8,
    Compact6,
}

#[must_use]
pub fn hourly_density(width: u16) -> HourlyDensity {
    match width {
        130..=u16::MAX => HourlyDensity::Full16,
        80..=129 => HourlyDensity::Full12,
        60..=79 => HourlyDensity::Compact8,
        _ => HourlyDensity::Compact6,
    }
}

#[must_use]
pub fn visible_hour_count(width: u16) -> usize {
    match hourly_density(width) {
        HourlyDensity::Full16 => 16,
        HourlyDensity::Full12 => 12,
        HourlyDensity::Compact8 => 8,
        HourlyDensity::Compact6 => 6,
    }
}
