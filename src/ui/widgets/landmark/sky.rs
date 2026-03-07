#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use crate::domain::weather::{ForecastBundle, weather_code_to_category};
use crate::ui::animation::UiMotionContext;
use crate::ui::widgets::landmark::compact::compact_condition_scene;
use crate::ui::widgets::landmark::shared::canvas_to_lines;
use crate::ui::widgets::landmark::{LandmarkScene, tint_for_category};

mod astronomy;
mod canvas;
mod glyphs;
mod strip;

use astronomy::{
    celestial_progress, current_hour, format_optional_duration_hm, format_time_hm, moon_phase,
    sky_context_line, sun_window,
};
use canvas::{
    arc_bounds, draw_arc, draw_celestial_icon, locate_arc_y, paint_celestial_guide,
    paint_night_stars, paint_solar_noon_marker, paint_sun_event_markers,
};
#[cfg(test)]
use glyphs::{arc_glyph, center_symbol, precip_symbol, symbol_for_code};
use strip::{
    BandDensity, HourSample, build_hour_samples, horizon_y, paint_horizon_strip,
    paint_observatory_band, summary_segments, write_summary_line,
};

#[derive(Debug, Clone)]
struct SkyObservatoryData {
    sunrise_h: f32,
    sunset_h: f32,
    now_h: f32,
    is_day: bool,
    body_x: usize,
    body_y: usize,
    moon_symbol: char,
    sunrise_text: String,
    sunset_text: String,
    daylight_text: String,
    sunshine_text: Option<String>,
    band_density: BandDensity,
    hour_samples: Vec<HourSample>,
}

#[allow(clippy::needless_range_loop)]
#[must_use]
pub fn scene_for_sky_observatory(
    bundle: &ForecastBundle,
    motion: UiMotionContext,
    width: u16,
    height: u16,
) -> LandmarkScene {
    let w = width as usize;
    let h = height as usize;
    let category = weather_code_to_category(bundle.current.weather_code);
    if w < 24 || h < 8 {
        return compact_condition_scene(category, bundle.current.is_day, width, height);
    }

    let data = derive_sky_observatory_data(bundle, w, h);
    let canvas = build_sky_observatory_canvas(bundle, motion, w, h, &data);
    LandmarkScene {
        label: "Sky Observatory · Sun/Moon Arc".to_string(),
        lines: canvas_to_lines(canvas, w),
        tint: tint_for_category(category),
        context_line: Some(sky_context_line(
            data.sunrise_h,
            data.sunset_h,
            data.now_h,
            data.is_day,
        )),
    }
}

#[allow(clippy::needless_range_loop)]
fn build_sky_observatory_canvas(
    _bundle: &ForecastBundle,
    motion: UiMotionContext,
    width: usize,
    height: usize,
    data: &SkyObservatoryData,
) -> Vec<Vec<char>> {
    let mut canvas = vec![vec![' '; width]; height];
    let horizon = horizon_y(height);
    let (arc_top, arc_bottom) = arc_bounds(height, horizon);
    draw_arc(&mut canvas, width, arc_top, arc_bottom);
    paint_solar_noon_marker(&mut canvas, width, arc_top);
    paint_sun_event_markers(&mut canvas, width, arc_top, arc_bottom);
    paint_night_stars(&mut canvas, width, arc_bottom, horizon, data.is_day, motion);
    paint_celestial_guide(&mut canvas, data.body_x, data.body_y, horizon);
    draw_celestial_icon(
        &mut canvas,
        data.body_x,
        data.body_y,
        data.is_day,
        data.moon_symbol,
        width,
        height,
    );

    paint_horizon_strip(&mut canvas, horizon, width);
    let tick_y = horizon.saturating_add(1);
    let weather_y = horizon.saturating_add(2);
    let precip_y = horizon.saturating_add(3);
    let summary_y = horizon.saturating_add(4);
    paint_observatory_band(
        &mut canvas,
        width,
        tick_y,
        weather_y,
        precip_y,
        data.band_density,
        &data.hour_samples,
    );
    let summary = summary_segments(
        width,
        &data.sunrise_text,
        &data.sunset_text,
        &data.daylight_text,
        data.sunshine_text.as_deref(),
        data.moon_symbol,
    );
    write_summary_line(&mut canvas, summary_y, width, &summary);

    canvas
}

fn derive_sky_observatory_data(
    bundle: &ForecastBundle,
    width: usize,
    height: usize,
) -> SkyObservatoryData {
    let (sunrise_h, sunset_h) = sun_window(bundle);
    let now_h = current_hour(bundle);
    let is_day = bundle.current.is_day;
    let moon_symbol = moon_phase(bundle);
    let horizon = horizon_y(height);
    let (arc_top, arc_bottom) = arc_bounds(height, horizon);
    let progress = celestial_progress(sunrise_h, sunset_h, now_h, is_day);
    let body_x = (progress * (width.saturating_sub(1)) as f32).round() as usize;
    let body_y = locate_arc_y(body_x, width, arc_top, arc_bottom);
    let day = bundle.daily.first();
    let (band_density, hour_samples) = build_hour_samples(bundle, width);

    SkyObservatoryData {
        sunrise_h,
        sunset_h,
        now_h,
        is_day,
        body_x,
        body_y,
        moon_symbol,
        sunrise_text: format_time_hm(sunrise_h),
        sunset_text: format_time_hm(sunset_h),
        daylight_text: format_optional_duration_hm(day.and_then(|entry| entry.daylight_duration_s))
            .unwrap_or_else(|| "--:--".to_string()),
        sunshine_text: format_optional_duration_hm(day.and_then(|entry| entry.sunshine_duration_s)),
        band_density,
        hour_samples,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDateTime};

    fn fixture_dt(value: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").expect("valid fixture datetime")
    }

    fn sample_bundle_with_times(
        now: &str,
        is_day: bool,
        sunrise: &str,
        sunset: &str,
    ) -> ForecastBundle {
        let mut bundle = crate::test_support::sample_bundle();
        bundle.current.is_day = is_day;
        bundle.current.weather_code = 0;
        bundle.daily[0].sunrise = Some(fixture_dt(sunrise));
        bundle.daily[0].sunset = Some(fixture_dt(sunset));
        bundle.daily[0].daylight_duration_s = Some(40_260.0);
        bundle.daily[0].sunshine_duration_s = Some(15_300.0);
        bundle.hourly = (0..24)
            .map(|offset| {
                let mut hour = crate::test_support::sample_hourly();
                hour.time = fixture_dt(now) + Duration::hours(i64::from(offset));
                hour.weather_code = Some(if offset % 5 == 0 { 61 } else { 3 });
                hour.precipitation_mm = Some(if offset % 7 == 0 { 2.8 } else { 0.1 });
                hour
            })
            .collect();
        bundle
    }

    // ── sky_context_line ─────────────────────────────────────────────────────

    #[test]
    fn sky_context_line_day_with_remaining_daylight() {
        let line = sky_context_line(6.0, 18.0, 16.0, true);
        assert!(line.contains("daylight remaining"), "got: {line}");
        assert!(line.contains("2h"), "got: {line}");
    }

    #[test]
    fn sky_context_line_day_sunset_passing() {
        let line = sky_context_line(6.0, 18.0, 18.0, true);
        assert!(line.contains("twilight"), "got: {line}");
    }

    #[test]
    fn sky_context_line_night_before_sunrise() {
        let line = sky_context_line(6.0, 18.0, 4.0, false);
        assert!(line.contains("until sunrise"), "got: {line}");
    }

    #[test]
    fn sky_context_line_night_after_sunset() {
        let line = sky_context_line(6.0, 18.0, 22.0, false);
        assert!(line.contains("until sunrise"), "got: {line}");
    }

    // ── celestial_progress ───────────────────────────────────────────────────

    #[test]
    fn celestial_progress_maps_day_endpoints() {
        assert!((celestial_progress(6.0, 18.0, 6.0, true) - 0.0).abs() < f32::EPSILON);
        assert!((celestial_progress(6.0, 18.0, 18.0, true) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn celestial_progress_wraps_at_night() {
        assert!((celestial_progress(6.0, 18.0, 18.0, false) - 1.0).abs() < f32::EPSILON);
        assert!((celestial_progress(6.0, 18.0, 6.0, false) - 0.0).abs() < f32::EPSILON);
        let late_night = celestial_progress(6.0, 18.0, 23.0, false);
        assert!(
            late_night < 1.0 && late_night > 0.0,
            "progress={late_night}"
        );
    }

    // ── precip_symbol ────────────────────────────────────────────────────────

    #[test]
    fn precip_symbol_covers_all_thresholds() {
        assert_eq!(precip_symbol(None), '·');
        assert_eq!(precip_symbol(Some(0.0)), '·');
        assert_eq!(precip_symbol(Some(0.1)), '░');
        assert_eq!(precip_symbol(Some(0.5)), '▒');
        assert_eq!(precip_symbol(Some(1.5)), '▓');
        assert_eq!(precip_symbol(Some(3.0)), '█');
    }

    // ── symbol_for_code ──────────────────────────────────────────────────────

    #[test]
    fn symbol_for_code_maps_all_categories() {
        let cases = [
            (0u8, 'o'),
            (3, '~'),
            (61, '/'),
            (71, '*'),
            (45, '='),
            (95, '!'),
        ];
        for (code, expected) in cases {
            assert_eq!(symbol_for_code(code), expected, "code={code}");
        }
    }

    // ── moon_phase ───────────────────────────────────────────────────────────

    #[test]
    fn moon_phase_returns_a_valid_moon_symbol() {
        let bundle = crate::test_support::sample_bundle();
        let symbol = moon_phase(&bundle);
        let valid = ['●', '◔', '◑', '◕', '○', '◖', '◐', '◗'];
        assert!(valid.contains(&symbol), "unexpected symbol: {symbol}");
    }

    // ── format_time_hm ───────────────────────────────────────────────────────

    #[test]
    fn format_time_hm_rounds_minutes_correctly() {
        assert_eq!(format_time_hm(0.0), "00:00");
        assert_eq!(format_time_hm(6.5), "06:30");
        assert_eq!(format_time_hm(23.75), "23:45");
        assert_eq!(format_time_hm(24.0), "00:00");
    }

    // ── arc_glyph ────────────────────────────────────────────────────────────

    #[test]
    fn arc_glyph_returns_dash_at_top_and_corners() {
        let g = arc_glyph(40, 1, 80, 1, 40);
        assert_eq!(g, '─');
        let g2 = arc_glyph(0, 5, 80, 1, 40);
        assert_eq!(g2, '·');
        let g3 = arc_glyph(20, 5, 80, 1, 40);
        assert_eq!(g3, '╭');
        let g4 = arc_glyph(60, 5, 80, 1, 40);
        assert_eq!(g4, '╮');
    }

    // ── center_symbol ────────────────────────────────────────────────────────

    #[test]
    fn center_symbol_returns_correct_glyph() {
        assert_eq!(center_symbol(true, true, '●'), '☀');
        assert_eq!(center_symbol(true, false, '●'), '◉');
        assert_eq!(center_symbol(false, true, '◑'), '◑');
        assert_eq!(center_symbol(false, false, '●'), '●');
    }

    // ── derive_sky_observatory_data ─────────────────────────────────────────

    #[test]
    fn derive_sky_observatory_data_preserves_night_positioning() {
        let bundle = sample_bundle_with_times(
            "2026-02-12T23:00",
            false,
            "2026-02-12T06:55",
            "2026-02-12T18:06",
        );
        let data = derive_sky_observatory_data(&bundle, 80, 16);

        assert!(data.body_x < 79, "body_x={}", data.body_x);
        assert!(data.body_x > 0, "body_x={}", data.body_x);
        assert_eq!(data.sunrise_text, "06:55");
        assert_eq!(data.sunset_text, "18:06");
        assert_eq!(data.daylight_text, "11:11");
        assert_eq!(data.sunshine_text.as_deref(), Some("04:15"));
    }

    // ── scene_for_sky_observatory ───────────────────────────────────────────

    #[test]
    fn scene_for_sky_observatory_compact_for_tiny_area() {
        let bundle = crate::test_support::sample_bundle();
        let scene =
            scene_for_sky_observatory(&bundle, crate::test_support::test_motion_context(), 10, 4);
        assert!(!scene.lines.is_empty());
    }

    #[test]
    fn scene_for_sky_observatory_day_scene_includes_noon_marker_and_daylight_summary() {
        let bundle = sample_bundle_with_times(
            "2026-02-12T10:00",
            true,
            "2026-02-12T06:55",
            "2026-02-12T18:06",
        );
        let scene =
            scene_for_sky_observatory(&bundle, crate::test_support::test_motion_context(), 110, 18);
        let output = scene.lines.join("\n");

        assert!(!scene.lines.is_empty());
        assert!(scene.context_line.is_some());
        assert!(output.contains('┬'));
        assert!(output.contains("Daylight 11:11"));
        assert!(output.contains("Sunshine 04:15"));
        assert!(output.contains("Rise 06:55"));
    }

    #[test]
    fn scene_for_sky_observatory_night_scene_includes_guide_and_stars() {
        let bundle = sample_bundle_with_times(
            "2026-02-12T23:00",
            false,
            "2026-02-12T06:55",
            "2026-02-12T18:06",
        );
        let scene =
            scene_for_sky_observatory(&bundle, crate::test_support::test_motion_context(), 96, 18);
        let output = scene.lines.join("\n");

        assert!(output.contains('│'));
        assert!(output.contains('✦') || output.contains('*'));
        assert!(output.contains("Moon "));
    }

    #[test]
    fn scene_for_sky_observatory_narrow_scene_drops_extra_summary_segments() {
        let bundle = sample_bundle_with_times(
            "2026-02-12T10:00",
            true,
            "2026-02-12T06:55",
            "2026-02-12T18:06",
        );
        let scene =
            scene_for_sky_observatory(&bundle, crate::test_support::test_motion_context(), 60, 14);
        let summary = scene.lines.last().expect("summary line");

        assert!(summary.contains("Rise 06:55"));
        assert!(summary.contains("Moon "));
        assert!(!summary.contains("Daylight"));
        assert!(!summary.contains("Sunshine"));
    }

    #[test]
    fn scene_for_sky_observatory_wet_scene_shows_precip_lane_blocks() {
        let mut bundle = sample_bundle_with_times(
            "2026-02-12T10:00",
            true,
            "2026-02-12T06:55",
            "2026-02-12T18:06",
        );
        for hour in &mut bundle.hourly {
            hour.weather_code = Some(61);
            hour.precipitation_mm = Some(3.4);
        }
        let scene =
            scene_for_sky_observatory(&bundle, crate::test_support::test_motion_context(), 96, 18);
        let output = scene.lines.join("\n");

        assert!(output.contains('█'));
        assert!(output.contains('/'));
    }

    #[test]
    fn symbol_for_unknown_code_returns_question_mark() {
        let sym = symbol_for_code(255);
        assert_eq!(sym, '?');
    }
}
