#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HourlyDensity {
    Full12,
    Compact8,
    Compact6,
}

pub fn hourly_density(width: u16) -> HourlyDensity {
    match width {
        80..=u16::MAX => HourlyDensity::Full12,
        60..=79 => HourlyDensity::Compact8,
        _ => HourlyDensity::Compact6,
    }
}
