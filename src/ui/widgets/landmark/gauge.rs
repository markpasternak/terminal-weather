#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use crate::domain::weather::{ForecastBundle, Units, weather_code_to_category};
use crate::ui::animation::UiMotionContext;
use crate::ui::widgets::landmark::shared::{fit_lines, fit_lines_centered};
use crate::ui::widgets::landmark::{LandmarkScene, tint_for_category};

mod columns;
mod context;
mod data;
mod meters;

use columns::{append_wind_direction_block, build_left_lines, build_right_lines, merge_columns};
use context::gauge_context_line;
use data::collect_gauge_data;
#[cfg(test)]
use data::{GaugeData, gauge_sun_times, left_column_width};
#[cfg(test)]
use meters::{
    gust_range_label, meter_with_threshold, precip_range_label, range_label, temp_range_label,
};

#[must_use]
pub fn scene_for_gauge_cluster(
    bundle: &ForecastBundle,
    units: Units,
    width: u16,
    height: u16,
    motion: UiMotionContext,
) -> LandmarkScene {
    let w = width as usize;
    let h = height as usize;
    let category = weather_code_to_category(bundle.current.weather_code);
    let data = collect_gauge_data(bundle, units, w);
    let left_lines = build_left_lines(&data);
    let mut lines = if w >= 74 && h >= 9 {
        let right_lines = build_right_lines(&data, category, bundle.current.is_day);
        merge_columns(&left_lines, &right_lines, data.left_col_width)
    } else {
        left_lines
    };

    if h >= 12 && w < 74 {
        append_wind_direction_block(&mut lines, data.wind_direction_10m);
    }

    let mut fitted = if h >= 12 {
        fit_lines_centered(lines, w, h)
    } else {
        fit_lines(lines, w, h)
    };
    apply_gauge_accent(&mut fitted, motion, w);

    LandmarkScene {
        label: "Gauge Cluster · Live Instruments".to_string(),
        lines: fitted,
        tint: tint_for_category(category),
        context_line: Some(gauge_context_line(&data)),
    }
}

fn apply_gauge_accent(lines: &mut [String], motion: UiMotionContext, width: usize) {
    if !matches!(
        motion.motion_mode,
        crate::ui::animation::MotionMode::Cinematic | crate::ui::animation::MotionMode::Standard
    ) {
        return;
    }
    let Some(first_line) = lines.first_mut() else {
        return;
    };
    if width < 20 || first_line.len() + 2 >= width {
        return;
    }
    let glyph = if motion
        .lane("gauge-accent")
        .pulse(motion.elapsed_seconds, 1.1, 0)
        > 0.6
    {
        '•'
    } else {
        '·'
    };
    first_line.push(' ');
    first_line.push(glyph);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};

    fn sample_data() -> GaugeData {
        GaugeData {
            temp_c: 20,
            temp_display: 20,
            temp_unit: "C",
            humidity: 50.0,
            pressure: 1012.0,
            wind: 5.0,
            gust: 10.0,
            wind_direction_10m: 180.0,
            uv: 2.0,
            vis_km: 10.0,
            precip_now: 0.0,
            cloud: 30.0,
            meter_w: 12,
            left_col_width: 40,
            right_trend_width: 12,
            sunrise: "06:10".to_string(),
            sunset: "17:40".to_string(),
            temp_track_display: vec![10.0, 12.0, 14.0],
            precip_track: vec![0.0, 1.0, 2.0],
            gust_track: vec![8.0, 10.0, 12.0],
        }
    }

    fn bundle_with_hourly(count: usize) -> ForecastBundle {
        let mut bundle = crate::test_support::sample_bundle();
        let base = bundle.hourly[0].clone();
        bundle.hourly = (0..count)
            .map(|idx| {
                let mut hour = base.clone();
                hour.time = base.time + chrono::Duration::hours(i64::try_from(idx).unwrap_or(0));
                hour.temperature_2m_c = Some(5.0 + idx as f32);
                hour.precipitation_mm = Some((idx % 4) as f32);
                hour.wind_gusts_10m = Some(12.0 + idx as f32);
                hour
            })
            .collect();
        bundle
    }

    #[test]
    fn gauge_context_line_prioritizes_rules_in_order() {
        let mut data = sample_data();
        data.uv = 8.0;
        data.gust = 70.0;
        let line = gauge_context_line(&data);
        assert!(line.contains("very high"));

        data.uv = 4.0;
        let line = gauge_context_line(&data);
        assert!(line.contains("secure loose objects"));
    }

    #[test]
    fn gauge_context_line_uses_secondary_conditions() {
        let mut data = sample_data();
        data.vis_km = 0.6;
        assert!(gauge_context_line(&data).contains("reduced visibility"));

        data.vis_km = 10.0;
        data.precip_now = 1.4;
        assert!(gauge_context_line(&data).contains("Active precipitation"));

        data.precip_now = 0.0;
        data.humidity = 90.0;
        data.temp_c = 20;
        assert!(gauge_context_line(&data).contains("muggy"));

        data.temp_c = 10;
        assert!(gauge_context_line(&data).contains("damp"));
    }

    #[test]
    fn meter_and_range_helpers_cover_edge_cases() {
        let bar = meter_with_threshold(0.5, 6, Some(0.5));
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
        assert!(bar.contains('|'));

        assert_eq!(range_label(&[], "°"), "");
        assert_eq!(temp_range_label(&[1.0, 3.0, 2.0]), "1°–3°");
        assert_eq!(precip_range_label(&[0.0, 0.0]), "");
        assert_eq!(precip_range_label(&[0.2, 1.8]), "2mm");
        assert_eq!(gust_range_label(&[0.0, 0.0]), "");
        assert!(gust_range_label(&[10.0, 12.9]).contains("m/s"));
    }

    #[test]
    fn left_column_width_changes_at_breakpoints() {
        // Below 74: returns width as-is.
        assert_eq!(left_column_width(60), 60);
        // 74-85: scaled to ~62% — narrower than the raw width.
        assert!(left_column_width(80) < 80);
        // >=86: scaled to ~58%; wider terminals still give more absolute columns.
        assert!(left_column_width(100) > left_column_width(80));
    }

    #[test]
    fn collect_gauge_data_and_tracks_use_bundle_content() {
        let bundle = bundle_with_hourly(24);
        let data = collect_gauge_data(&bundle, Units::Celsius, 100);
        assert_eq!(data.temp_unit, "C");
        assert_eq!(data.temp_track_display.len(), 24);
        assert_eq!(data.precip_track.len(), 24);
        assert_eq!(data.gust_track.len(), 24);
        assert!(data.left_col_width > 0);
        assert!(data.meter_w >= 10);
    }

    #[test]
    fn gauge_sun_times_default_and_explicit() {
        let mut bundle = crate::test_support::sample_bundle();
        let (sunrise, sunset) = gauge_sun_times(&bundle);
        assert_eq!(sunrise, "--:--");
        assert_eq!(sunset, "--:--");

        let date = NaiveDate::from_ymd_opt(2026, 2, 12).expect("date");
        let sunrise_dt =
            NaiveDateTime::parse_from_str("2026-02-12T06:30", "%Y-%m-%dT%H:%M").expect("sunrise");
        let sunset_dt =
            NaiveDateTime::parse_from_str("2026-02-12T17:50", "%Y-%m-%dT%H:%M").expect("sunset");
        bundle.daily[0] = crate::domain::weather::DailyForecast {
            date,
            sunrise: Some(sunrise_dt),
            sunset: Some(sunset_dt),
            ..bundle.daily[0].clone()
        };
        let (sunrise, sunset) = gauge_sun_times(&bundle);
        assert_eq!(sunrise, "06:30");
        assert_eq!(sunset, "17:50");
    }

    #[test]
    fn scene_for_gauge_cluster_renders_context_and_lines() {
        let bundle = bundle_with_hourly(24);
        let scene = scene_for_gauge_cluster(
            &bundle,
            Units::Celsius,
            90,
            12,
            crate::test_support::test_motion_context(),
        );
        assert!(!scene.lines.is_empty());
        assert!(scene.context_line.is_some());
    }
}
