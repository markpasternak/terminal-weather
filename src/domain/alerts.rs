use crate::domain::weather::{ForecastBundle, Units, convert_temp, round_temp};

#[derive(Debug, Clone)]
pub struct WeatherAlert {
    pub icon: &'static str,
    pub message: String,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Danger,
}

pub fn scan_alerts(bundle: &ForecastBundle, units: Units) -> Vec<WeatherAlert> {
    let mut alerts = Vec::new();

    // Check next 24h of hourly data
    let next_24h = &bundle.hourly[..bundle.hourly.len().min(24)];

    // Wind gust alert
    if let Some(max_gust) = next_24h
        .iter()
        .filter_map(|h| h.wind_gusts_10m)
        .max_by(f32::total_cmp)
    {
        if max_gust >= 80.0 {
            alerts.push(WeatherAlert {
                icon: "⚡",
                message: format!("Wind gusts up to {} km/h", max_gust.round() as i32),
                severity: AlertSeverity::Danger,
            });
        } else if max_gust >= 50.0 {
            alerts.push(WeatherAlert {
                icon: "💨",
                message: format!("Wind gusts up to {} km/h", max_gust.round() as i32),
                severity: AlertSeverity::Warning,
            });
        }
    }

    // UV alert (from daily)
    if let Some(uv) = bundle.daily.first().and_then(|d| d.uv_index_max) {
        if uv >= 8.0 {
            alerts.push(WeatherAlert {
                icon: "☀",
                message: format!("UV index very high ({uv:.0})"),
                severity: AlertSeverity::Danger,
            });
        } else if uv >= 6.0 {
            alerts.push(WeatherAlert {
                icon: "☀",
                message: format!("UV index high ({uv:.0})"),
                severity: AlertSeverity::Warning,
            });
        }
    }

    // Freezing rain/drizzle alert
    let has_freezing = next_24h.iter().any(|h| {
        h.weather_code
            .is_some_and(|c| matches!(c, 56 | 57 | 66 | 67))
    });
    if has_freezing {
        alerts.push(WeatherAlert {
            icon: "❄",
            message: "Freezing rain/drizzle expected".to_string(),
            severity: AlertSeverity::Danger,
        });
    }

    // Heavy precipitation alert
    let total_precip: f32 = next_24h
        .iter()
        .filter_map(|h| h.precipitation_mm)
        .map(|p| p.max(0.0))
        .sum();
    if total_precip >= 25.0 {
        alerts.push(WeatherAlert {
            icon: "🌧",
            message: format!("Heavy precipitation: {total_precip:.1}mm in 24h"),
            severity: AlertSeverity::Warning,
        });
    }

    // Low visibility alert
    if let Some(min_vis) = next_24h
        .iter()
        .filter_map(|h| h.visibility_m)
        .min_by(f32::total_cmp)
        && min_vis < 1000.0
    {
        alerts.push(WeatherAlert {
            icon: "░",
            message: format!("Low visibility: {:.1}km", min_vis / 1000.0),
            severity: AlertSeverity::Warning,
        });
    }

    // Extreme temperature alert
    if let Some(max_temp) = next_24h
        .iter()
        .filter_map(|h| h.temperature_2m_c)
        .max_by(f32::total_cmp)
    {
        let display_temp = round_temp(convert_temp(max_temp, units));
        let unit_label = match units {
            Units::Celsius => "C",
            Units::Fahrenheit => "F",
        };
        if max_temp >= 38.0 {
            alerts.push(WeatherAlert {
                icon: "🔥",
                message: format!("Extreme heat: up to {display_temp}°{unit_label}"),
                severity: AlertSeverity::Danger,
            });
        }
    }
    if let Some(min_temp) = next_24h
        .iter()
        .filter_map(|h| h.temperature_2m_c)
        .min_by(f32::total_cmp)
    {
        let display_temp = round_temp(convert_temp(min_temp, units));
        let unit_label = match units {
            Units::Celsius => "C",
            Units::Fahrenheit => "F",
        };
        if min_temp <= -15.0 {
            alerts.push(WeatherAlert {
                icon: "❄",
                message: format!("Extreme cold: down to {display_temp}°{unit_label}"),
                severity: AlertSeverity::Danger,
            });
        }
    }

    // Thunderstorm alert
    let has_thunder = next_24h
        .iter()
        .any(|h| h.weather_code.is_some_and(|c| matches!(c, 95 | 96 | 99)));
    if has_thunder {
        alerts.push(WeatherAlert {
            icon: "⚡",
            message: "Thunderstorms expected".to_string(),
            severity: AlertSeverity::Warning,
        });
    }

    // Sort by severity (danger first)
    alerts.sort_by(|a, b| b.severity.cmp(&a.severity));
    alerts
}
