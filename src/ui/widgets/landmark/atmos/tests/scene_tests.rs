use super::super::{paint_horizon_haze, scene_for_weather, terrain::overlay_horizon_line};
use super::test_support::{blank_canvas, bundle_for_category, motion};
use crate::domain::weather::Units;

#[test]
fn scene_for_weather_compact_for_small_area() {
    let bundle = crate::test_support::sample_bundle();
    let scene = scene_for_weather(&bundle, Units::Celsius, motion(), 10, 5);
    assert!(!scene.lines.is_empty());
}

#[test]
fn scene_for_weather_renders_all_weather_categories() {
    let cases = [
        (0u8, true, "Clear day"),
        (61, true, "Rain"),
        (71, false, "Snow night"),
        (45, true, "Fog"),
        (95, true, "Thunder"),
        (3, true, "Cloudy"),
    ];
    for (code, is_day, label) in cases {
        let bundle = bundle_for_category(code, is_day);
        let scene = scene_for_weather(&bundle, Units::Celsius, motion(), 80, 24);
        assert!(
            !scene.lines.is_empty(),
            "scene for {label} (code={code}) should not be empty"
        );
        assert!(
            scene.context_line.is_some(),
            "scene for {label} should have a context line"
        );
    }
}

#[test]
fn scene_for_weather_night_vs_day_produces_different_output() {
    let day = bundle_for_category(0, true);
    let night = bundle_for_category(0, false);
    let day_scene = scene_for_weather(&day, Units::Celsius, motion(), 80, 24);
    let night_scene = scene_for_weather(&night, Units::Celsius, motion(), 80, 24);
    assert_ne!(
        day_scene.lines, night_scene.lines,
        "day vs night should differ"
    );
}

#[test]
fn scene_for_weather_freezing_rain_no_panic() {
    // Code 56 = freezing drizzle
    let bundle = bundle_for_category(56, true);
    let scene = scene_for_weather(&bundle, Units::Fahrenheit, motion(), 60, 20);
    assert!(!scene.lines.is_empty());
}

#[test]
fn scene_for_weather_hail_code_no_panic() {
    // Code 96 and 99 = thunderstorm with hail → exercises has_hail=true branch
    for code in [96u8, 99] {
        let bundle = bundle_for_category(code, true);
        let scene = scene_for_weather(&bundle, Units::Celsius, motion(), 60, 20);
        assert!(!scene.lines.is_empty());
    }
}

#[test]
fn scene_for_weather_few_hourly_samples_falls_back_to_compact() {
    // 1 hourly sample → build_atmos_canvas returns None → compact fallback
    let mut bundle = bundle_for_category(0, true);
    bundle.hourly.truncate(1);
    let scene = scene_for_weather(&bundle, Units::Celsius, motion(), 60, 20);
    assert!(!scene.lines.is_empty());
}

#[test]
fn overlay_horizon_line_no_panic_when_horizon_beyond_height() {
    let mut canvas = blank_canvas(20, 5);
    let terrain_top = vec![3usize; 20];
    // horizon_y >= height → should early return without panic
    overlay_horizon_line(&mut canvas, &terrain_top, 10, 20, 5);
}

#[test]
fn overlay_horizon_line_marks_terrain_above_horizon() {
    let mut canvas = blank_canvas(20, 10);
    let terrain_top = vec![3usize; 20]; // all columns: terrain top at 3 < horizon 7
    overlay_horizon_line(&mut canvas, &terrain_top, 7, 20, 10);
    // terrain_top[x] = 3 <= horizon_y = 7, so NO '─' chars placed
    // (condition: terrain_top[x] > horizon_y to place '─')
    // Change terrain_top to be > horizon for the other branch
    let terrain_above = vec![8usize; 20]; // above horizon_y=7
    let mut canvas2 = blank_canvas(20, 10);
    overlay_horizon_line(&mut canvas2, &terrain_above, 7, 20, 10);
    let dashes = canvas2.iter().flatten().filter(|c| **c == '─').count();
    assert!(dashes > 0, "should place '─' where terrain > horizon");
}

#[test]
fn paint_horizon_haze_with_zero_haze_y_no_panic() {
    let mut canvas = blank_canvas(20, 5);
    // horizon_y = 0 → haze_y = 0 → early return
    paint_horizon_haze(&mut canvas, 0, 20);
    // horizon_y = 1 → haze_y = 0 (saturating_sub) → early return
    paint_horizon_haze(&mut canvas, 1, 20);
}

#[test]
fn paint_horizon_haze_writes_dots_at_normal_horizon() {
    let mut canvas = blank_canvas(40, 16);
    paint_horizon_haze(&mut canvas, 10, 40);
    // Should write some '░' chars in the haze row
    let haze = canvas.iter().flatten().filter(|c| **c == '░').count();
    assert!(haze > 0, "expected haze marks");
}
