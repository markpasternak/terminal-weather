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

#[cfg(test)]
use astronomy::format_time_hm;
use astronomy::{current_hour, moon_phase, sky_context_line, sun_window};
use canvas::{
    arc_bounds, draw_arc, draw_celestial_icon, locate_arc_y, paint_cardinal_markers,
    paint_night_stars, paint_sun_event_markers,
};
#[cfg(test)]
use glyphs::{arc_glyph, center_symbol, precip_symbol, symbol_for_code};
use strip::{paint_horizon_strip, plot_hourly_strip, write_summary_line};

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

    let canvas = build_sky_observatory_canvas(bundle, motion, w, h);
    let (sunrise_h, sunset_h) = sun_window(bundle);
    let now_h = current_hour(bundle);
    LandmarkScene {
        label: "Sky Observatory · Sun/Moon Arc".to_string(),
        lines: canvas_to_lines(canvas, w),
        tint: tint_for_category(category),
        context_line: Some(sky_context_line(
            sunrise_h,
            sunset_h,
            now_h,
            bundle.current.is_day,
        )),
    }
}

#[allow(clippy::needless_range_loop)]
fn build_sky_observatory_canvas(
    bundle: &ForecastBundle,
    motion: UiMotionContext,
    width: usize,
    height: usize,
) -> Vec<Vec<char>> {
    let mut canvas = vec![vec![' '; width]; height];
    let (arc_top, arc_bottom) = arc_bounds(height);
    draw_arc(&mut canvas, width, arc_top, arc_bottom);

    let (sunrise_h, sunset_h) = sun_window(bundle);
    let now_h = current_hour(bundle);
    let day_span = (sunset_h - sunrise_h).max(0.1);
    let progress = ((now_h - sunrise_h) / day_span).clamp(0.0, 1.0);
    let marker_x = (progress * (width.saturating_sub(1)) as f32).round() as usize;
    let marker_y = locate_arc_y(marker_x, width, arc_top, arc_bottom);
    draw_celestial_icon(
        &mut canvas,
        marker_x,
        marker_y,
        bundle.current.is_day,
        moon_phase(bundle),
        width,
        height,
    );
    paint_cardinal_markers(&mut canvas, width, arc_top, arc_bottom, marker_x);
    paint_sun_event_markers(&mut canvas, width, arc_top, arc_bottom, sunrise_h, sunset_h);
    paint_night_stars(
        &mut canvas,
        width,
        arc_bottom,
        bundle.current.is_day,
        motion,
    );

    let strip_y = height.saturating_sub(3);
    let precip_y = height.saturating_sub(2);
    let summary_y = height.saturating_sub(1);
    paint_horizon_strip(&mut canvas, strip_y, width);
    plot_hourly_strip(bundle, &mut canvas, strip_y, precip_y, width);
    write_summary_line(&mut canvas, summary_y, width, sunrise_h, sunset_h, now_h);

    canvas
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── sky_context_line ─────────────────────────────────────────────────────

    #[test]
    fn sky_context_line_day_with_remaining_daylight() {
        // is_day=true, 2h of daylight left
        let line = sky_context_line(6.0, 18.0, 16.0, true);
        assert!(line.contains("daylight remaining"), "got: {line}");
        assert!(line.contains("2h"), "got: {line}");
    }

    #[test]
    fn sky_context_line_day_sunset_passing() {
        // is_day=true but now_h == sunset_h
        let line = sky_context_line(6.0, 18.0, 18.0, true);
        assert!(line.contains("twilight"), "got: {line}");
    }

    #[test]
    fn sky_context_line_night_before_sunrise() {
        // is_day=false, now_h < sunrise_h
        let line = sky_context_line(6.0, 18.0, 4.0, false);
        assert!(line.contains("until sunrise"), "got: {line}");
    }

    #[test]
    fn sky_context_line_night_after_sunset() {
        // is_day=false, now_h > sunset_h
        let line = sky_context_line(6.0, 18.0, 22.0, false);
        assert!(line.contains("until sunrise"), "got: {line}");
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
            (0u8, 'o'), // Clear
            (3, '~'),   // Cloudy
            (61, '/'),  // Rain
            (71, '*'),  // Snow
            (45, '='),  // Fog
            (95, '!'),  // Thunder
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
        // Wraps via rem_euclid
        assert_eq!(format_time_hm(24.0), "00:00");
    }

    // ── arc_glyph ────────────────────────────────────────────────────────────

    #[test]
    fn arc_glyph_returns_dash_at_top_and_corners() {
        // At the top row, near center → '─'
        let g = arc_glyph(40, 1, 80, 1, 40);
        assert_eq!(g, '─');
        // Left edge → '·'
        let g2 = arc_glyph(0, 5, 80, 1, 40);
        assert_eq!(g2, '·');
        // Left third → '╭'
        let g3 = arc_glyph(20, 5, 80, 1, 40);
        assert_eq!(g3, '╭');
        // Right third → '╮'
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

    // ── scene_for_sky_observatory compact path ────────────────────────────────

    #[test]
    fn scene_for_sky_observatory_compact_for_tiny_area() {
        let bundle = crate::test_support::sample_bundle();
        // Below minimum size → compact scene (short lines)
        let scene =
            scene_for_sky_observatory(&bundle, crate::test_support::test_motion_context(), 10, 4);
        assert!(!scene.lines.is_empty());
    }

    #[test]
    fn scene_for_sky_observatory_full_for_normal_area() {
        let bundle = crate::test_support::sample_bundle();
        let scene =
            scene_for_sky_observatory(&bundle, crate::test_support::test_motion_context(), 60, 16);
        assert!(!scene.lines.is_empty());
        assert!(scene.context_line.is_some());
        assert!(scene.label.contains("Sky Observatory"));
    }

    // ── WeatherCategory symbol dispatch via symbol_for_code ──────────────────

    #[test]
    fn symbol_for_unknown_code_returns_question_mark() {
        // Weather code 255 is not a standard code → Unknown
        let sym = symbol_for_code(255);
        assert_eq!(sym, '?');
    }
}
