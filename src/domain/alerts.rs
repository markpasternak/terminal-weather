#![allow(clippy::cast_possible_truncation)]

use crate::domain::weather::{ForecastBundle, HourlyForecast, Units, convert_temp, round_temp};

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

#[must_use]
pub fn scan_alerts(bundle: &ForecastBundle, units: Units) -> Vec<WeatherAlert> {
    let mut alerts = Vec::new();
    let next_24h = next_24h_hours(bundle);

    push_alert(&mut alerts, wind_gust_alert(next_24h));
    push_alert(&mut alerts, uv_alert(bundle));
    push_alert(&mut alerts, freezing_alert(next_24h));
    push_alert(&mut alerts, heavy_precip_alert(next_24h));
    push_alert(&mut alerts, low_visibility_alert(next_24h));
    push_alert(&mut alerts, extreme_heat_alert(next_24h, units));
    push_alert(&mut alerts, extreme_cold_alert(next_24h, units));
    push_alert(&mut alerts, thunder_alert(next_24h));

    alerts.sort_by(|a, b| b.severity.cmp(&a.severity));
    alerts
}

fn push_alert(alerts: &mut Vec<WeatherAlert>, alert: Option<WeatherAlert>) {
    if let Some(alert) = alert {
        alerts.push(alert);
    }
}

fn next_24h_hours(bundle: &ForecastBundle) -> &[HourlyForecast] {
    &bundle.hourly[..bundle.hourly.len().min(24)]
}

fn wind_gust_alert(next_24h: &[HourlyForecast]) -> Option<WeatherAlert> {
    let max_gust = next_24h
        .iter()
        .filter_map(|h| h.wind_gusts_10m)
        .max_by(f32::total_cmp)?;
    if max_gust >= 80.0 {
        return Some(WeatherAlert {
            icon: "⚡",
            message: format!("Wind gusts up to {} km/h", max_gust.round() as i32),
            severity: AlertSeverity::Danger,
        });
    }
    if max_gust >= 50.0 {
        return Some(WeatherAlert {
            icon: "💨",
            message: format!("Wind gusts up to {} km/h", max_gust.round() as i32),
            severity: AlertSeverity::Warning,
        });
    }
    None
}

fn uv_alert(bundle: &ForecastBundle) -> Option<WeatherAlert> {
    let uv = bundle.daily.first().and_then(|d| d.uv_index_max)?;
    if uv >= 8.0 {
        return Some(WeatherAlert {
            icon: "☀",
            message: format!("UV index very high ({uv:.0})"),
            severity: AlertSeverity::Danger,
        });
    }
    if uv >= 6.0 {
        return Some(WeatherAlert {
            icon: "☀",
            message: format!("UV index high ({uv:.0})"),
            severity: AlertSeverity::Warning,
        });
    }
    None
}

fn freezing_alert(next_24h: &[HourlyForecast]) -> Option<WeatherAlert> {
    let has_freezing = next_24h.iter().any(|h| {
        h.weather_code
            .is_some_and(|c| matches!(c, 56 | 57 | 66 | 67))
    });
    if has_freezing {
        Some(WeatherAlert {
            icon: "❄",
            message: "Freezing rain/drizzle expected".to_string(),
            severity: AlertSeverity::Danger,
        })
    } else {
        None
    }
}

fn heavy_precip_alert(next_24h: &[HourlyForecast]) -> Option<WeatherAlert> {
    let total_precip: f32 = next_24h
        .iter()
        .filter_map(|h| h.precipitation_mm)
        .map(|p| p.max(0.0))
        .sum();
    if total_precip >= 25.0 {
        Some(WeatherAlert {
            icon: "🌧",
            message: format!("Heavy precipitation: {total_precip:.1}mm in 24h"),
            severity: AlertSeverity::Warning,
        })
    } else {
        None
    }
}

fn low_visibility_alert(next_24h: &[HourlyForecast]) -> Option<WeatherAlert> {
    let min_vis = next_24h
        .iter()
        .filter_map(|h| h.visibility_m)
        .min_by(f32::total_cmp)?;
    if min_vis < 1000.0 {
        Some(WeatherAlert {
            icon: "░",
            message: format!("Low visibility: {:.1}km", min_vis / 1000.0),
            severity: AlertSeverity::Warning,
        })
    } else {
        None
    }
}

fn extreme_heat_alert(next_24h: &[HourlyForecast], units: Units) -> Option<WeatherAlert> {
    let max_temp = next_24h
        .iter()
        .filter_map(|h| h.temperature_2m_c)
        .max_by(f32::total_cmp)?;
    if max_temp >= 38.0 {
        let display_temp = round_temp(convert_temp(max_temp, units));
        Some(WeatherAlert {
            icon: "🔥",
            message: format!("Extreme heat: up to {display_temp}°{}", unit_label(units)),
            severity: AlertSeverity::Danger,
        })
    } else {
        None
    }
}

fn extreme_cold_alert(next_24h: &[HourlyForecast], units: Units) -> Option<WeatherAlert> {
    let min_temp = next_24h
        .iter()
        .filter_map(|h| h.temperature_2m_c)
        .min_by(f32::total_cmp)?;
    if min_temp <= -15.0 {
        let display_temp = round_temp(convert_temp(min_temp, units));
        Some(WeatherAlert {
            icon: "❄",
            message: format!("Extreme cold: down to {display_temp}°{}", unit_label(units)),
            severity: AlertSeverity::Danger,
        })
    } else {
        None
    }
}

fn thunder_alert(next_24h: &[HourlyForecast]) -> Option<WeatherAlert> {
    let has_thunder = next_24h
        .iter()
        .any(|h| h.weather_code.is_some_and(|c| matches!(c, 95 | 96 | 99)));
    if has_thunder {
        Some(WeatherAlert {
            icon: "⚡",
            message: "Thunderstorms expected".to_string(),
            severity: AlertSeverity::Warning,
        })
    } else {
        None
    }
}

fn unit_label(units: Units) -> &'static str {
    match units {
        Units::Celsius => "C",
        Units::Fahrenheit => "F",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::weather::{CurrentConditions, DailyForecast, Location};
    use chrono::{NaiveDate, Utc};

    #[test]
    fn scan_alerts_orders_danger_before_warning() {
        let mut bundle = sample_bundle();
        bundle.hourly[0].weather_code = Some(95);
        bundle.hourly[0].wind_gusts_10m = Some(90.0);
        bundle.daily[0].uv_index_max = Some(9.0);

        let alerts = scan_alerts(&bundle, Units::Celsius);
        assert!(!alerts.is_empty());
        assert!(
            alerts
                .first()
                .is_some_and(|a| a.severity == AlertSeverity::Danger)
        );
        assert!(
            alerts
                .windows(2)
                .all(|window| window[0].severity >= window[1].severity)
        );
    }

    #[test]
    fn scan_alerts_handles_temperature_threshold_messages() {
        let mut bundle = sample_bundle();
        bundle.hourly[0].temperature_2m_c = Some(40.0);
        bundle.hourly[1].temperature_2m_c = Some(-20.0);

        let alerts = scan_alerts(&bundle, Units::Celsius);
        assert!(
            alerts
                .iter()
                .any(|a| a.message.contains("Extreme heat: up to 40°C"))
        );
        assert!(
            alerts
                .iter()
                .any(|a| a.message.contains("Extreme cold: down to -20°C"))
        );
    }

    fn sample_bundle() -> ForecastBundle {
        ForecastBundle {
            location: Location::from_coords(59.3293, 18.0686),
            current: CurrentConditions {
                temperature_2m_c: 2.0,
                relative_humidity_2m: 70.0,
                apparent_temperature_c: 1.0,
                dew_point_2m_c: 0.0,
                weather_code: 3,
                precipitation_mm: 0.0,
                cloud_cover: 60.0,
                pressure_msl_hpa: 1012.0,
                visibility_m: 9000.0,
                wind_speed_10m: 12.0,
                wind_gusts_10m: 20.0,
                wind_direction_10m: 180.0,
                is_day: true,
                high_today_c: Some(6.0),
                low_today_c: Some(-2.0),
            },
            hourly: (0..24)
                .map(|_| HourlyForecast {
                    time: NaiveDate::from_ymd_opt(2026, 2, 20)
                        .expect("valid date")
                        .and_hms_opt(0, 0, 0)
                        .expect("valid time"),
                    temperature_2m_c: Some(5.0),
                    weather_code: Some(3),
                    is_day: Some(true),
                    relative_humidity_2m: Some(70.0),
                    precipitation_probability: Some(10.0),
                    precipitation_mm: Some(0.0),
                    rain_mm: Some(0.0),
                    snowfall_cm: Some(0.0),
                    wind_speed_10m: Some(10.0),
                    wind_gusts_10m: Some(20.0),
                    pressure_msl_hpa: Some(1010.0),
                    visibility_m: Some(10000.0),
                    cloud_cover: Some(50.0),
                    cloud_cover_low: Some(20.0),
                    cloud_cover_mid: Some(20.0),
                    cloud_cover_high: Some(10.0),
                })
                .collect(),
            daily: vec![DailyForecast {
                date: NaiveDate::from_ymd_opt(2026, 2, 20).expect("valid date"),
                weather_code: Some(3),
                temperature_max_c: Some(6.0),
                temperature_min_c: Some(-2.0),
                sunrise: None,
                sunset: None,
                uv_index_max: Some(5.0),
                precipitation_probability_max: Some(20.0),
                precipitation_sum_mm: Some(0.0),
                rain_sum_mm: Some(0.0),
                snowfall_sum_cm: Some(0.0),
                precipitation_hours: Some(0.0),
                wind_gusts_10m_max: Some(20.0),
                daylight_duration_s: Some(36000.0),
                sunshine_duration_s: Some(18000.0),
            }],
            fetched_at: Utc::now(),
        }
    }
}
