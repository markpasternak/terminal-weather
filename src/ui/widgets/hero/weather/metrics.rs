use crate::domain::weather::HourlyForecast;

pub fn compass(deg: f32) -> &'static str {
    const DIRS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    let mut idx = ((deg.rem_euclid(360.0) / 45.0).round() as usize) % 8;
    if idx >= DIRS.len() {
        idx = 0;
    }
    DIRS[idx]
}

pub fn format_visibility(meters: f32) -> String {
    if !meters.is_finite() || meters <= 0.0 {
        return "--".to_string();
    }
    let km = meters / 1000.0;
    if km >= 20.0 {
        format!("{km:.0}km")
    } else {
        format!("{km:.1}km")
    }
}

pub fn pressure_trend_marker(hourly: &[HourlyForecast]) -> &'static str {
    let mut values = hourly.iter().take(6).filter_map(|h| h.pressure_msl_hpa);
    let Some(start) = values.next() else {
        return "";
    };
    let end = values.next_back().unwrap_or(start);
    let delta = end - start;
    if delta >= 1.2 {
        "↗"
    } else if delta <= -1.2 {
        "↘"
    } else {
        "→"
    }
}

pub fn cloud_layers_from_hourly(
    hourly: &[HourlyForecast],
) -> Option<(Option<f32>, Option<f32>, Option<f32>)> {
    let mut low = Vec::new();
    let mut mid = Vec::new();
    let mut high = Vec::new();
    for hour in hourly.iter().take(8) {
        if let Some(v) = hour.cloud_cover_low {
            low.push(v);
        }
        if let Some(v) = hour.cloud_cover_mid {
            mid.push(v);
        }
        if let Some(v) = hour.cloud_cover_high {
            high.push(v);
        }
    }

    let low_avg = average(&low);
    let mid_avg = average(&mid);
    let high_avg = average(&high);
    if low_avg.is_none() && mid_avg.is_none() && high_avg.is_none() {
        None
    } else {
        Some((low_avg, mid_avg, high_avg))
    }
}

pub fn format_cloud_layers(low: Option<f32>, mid: Option<f32>, high: Option<f32>) -> String {
    format!(
        "{}/{}/{}%",
        format_pct(low),
        format_pct(mid),
        format_pct(high)
    )
}

fn format_pct(value: Option<f32>) -> String {
    value
        .map(|v| format!("{:>2}", v.round() as i32))
        .unwrap_or_else(|| "--".to_string())
}

fn average(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f32>() / values.len() as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::weather::HourlyForecast;
    use chrono::NaiveDate;

    fn hour_with_pressure(pressure: Option<f32>) -> HourlyForecast {
        let time = NaiveDate::from_ymd_opt(2026, 2, 12)
            .unwrap()
            .and_hms_opt(8, 0, 0)
            .unwrap();
        HourlyForecast {
            time,
            temperature_2m_c: None,
            weather_code: None,
            is_day: None,
            relative_humidity_2m: None,
            precipitation_probability: None,
            precipitation_mm: None,
            rain_mm: None,
            snowfall_cm: None,
            wind_speed_10m: None,
            wind_gusts_10m: None,
            pressure_msl_hpa: pressure,
            visibility_m: None,
            cloud_cover: None,
            cloud_cover_low: None,
            cloud_cover_mid: None,
            cloud_cover_high: None,
        }
    }

    fn hour_with_clouds(low: Option<f32>, mid: Option<f32>, high: Option<f32>) -> HourlyForecast {
        let mut h = hour_with_pressure(None);
        h.cloud_cover_low = low;
        h.cloud_cover_mid = mid;
        h.cloud_cover_high = high;
        h
    }

    #[test]
    fn compass_maps_all_eight_directions() {
        assert_eq!(compass(0.0), "N");
        assert_eq!(compass(45.0), "NE");
        assert_eq!(compass(90.0), "E");
        assert_eq!(compass(135.0), "SE");
        assert_eq!(compass(180.0), "S");
        assert_eq!(compass(225.0), "SW");
        assert_eq!(compass(270.0), "W");
        assert_eq!(compass(315.0), "NW");
        assert_eq!(compass(360.0), "N");
    }

    #[test]
    fn format_visibility_handles_edge_cases() {
        assert_eq!(format_visibility(0.0), "--");
        assert_eq!(format_visibility(-10.0), "--");
        assert_eq!(format_visibility(f32::INFINITY), "--");
        assert_eq!(format_visibility(f32::NAN), "--");
        assert_eq!(format_visibility(500.0), "0.5km");
        assert_eq!(format_visibility(5000.0), "5.0km");
        assert_eq!(format_visibility(25_000.0), "25km");
    }

    #[test]
    fn pressure_trend_marker_returns_correct_arrows() {
        // No data → empty string
        assert_eq!(pressure_trend_marker(&[]), "");

        // Rising: +1.2 or more
        let rising: Vec<HourlyForecast> = vec![
            hour_with_pressure(Some(1000.0)),
            hour_with_pressure(Some(1001.5)),
        ];
        assert_eq!(pressure_trend_marker(&rising), "↗");

        // Falling: -1.2 or less
        let falling: Vec<HourlyForecast> = vec![
            hour_with_pressure(Some(1005.0)),
            hour_with_pressure(Some(1003.5)),
        ];
        assert_eq!(pressure_trend_marker(&falling), "↘");

        // Stable: within ±1.2
        let stable: Vec<HourlyForecast> = vec![
            hour_with_pressure(Some(1010.0)),
            hour_with_pressure(Some(1010.5)),
        ];
        assert_eq!(pressure_trend_marker(&stable), "→");
    }

    #[test]
    fn cloud_layers_all_none_returns_none() {
        let hourly = vec![hour_with_clouds(None, None, None)];
        assert!(cloud_layers_from_hourly(&hourly).is_none());
    }

    #[test]
    fn cloud_layers_with_data_returns_some() {
        let hourly = vec![hour_with_clouds(Some(20.0), Some(40.0), Some(60.0))];
        let result = cloud_layers_from_hourly(&hourly);
        assert!(result.is_some());
        let (low, mid, high) = result.unwrap();
        assert!(low.is_some());
        assert!(mid.is_some());
        assert!(high.is_some());
    }

    #[test]
    fn format_cloud_layers_formats_correctly() {
        let out = format_cloud_layers(Some(20.0), None, Some(60.0));
        assert!(out.contains("--"));
        assert!(out.contains('%'));
    }
}
