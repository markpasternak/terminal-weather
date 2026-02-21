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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hourly_density_ranges() {
        assert_eq!(hourly_density(130), HourlyDensity::Full16);
        assert_eq!(hourly_density(129), HourlyDensity::Full12);
        assert_eq!(hourly_density(80), HourlyDensity::Full12);
        assert_eq!(hourly_density(79), HourlyDensity::Compact8);
        assert_eq!(hourly_density(60), HourlyDensity::Compact8);
        assert_eq!(hourly_density(59), HourlyDensity::Compact6);
        assert_eq!(hourly_density(0), HourlyDensity::Compact6);
    }

    #[test]
    fn test_visible_hour_count() {
        assert_eq!(visible_hour_count(130), 16);
        assert_eq!(visible_hour_count(129), 12);
        assert_eq!(visible_hour_count(80), 12);
        assert_eq!(visible_hour_count(79), 8);
        assert_eq!(visible_hour_count(60), 8);
        assert_eq!(visible_hour_count(59), 6);
        assert_eq!(visible_hour_count(0), 6);
    }
}
