use super::super::{moon_visible, paint_starfield, place_celestial_body};
use super::test_support::{blank_canvas, bundle_for_category, motion};

#[test]
fn paint_starfield_small_canvas_no_panic() {
    let mut canvas = blank_canvas(4, 4);
    // horizon_y < 2 → early return
    paint_starfield(&mut canvas, 4, 1, motion(), 0.0);
    // w == 0 → early return
    paint_starfield(&mut canvas, 0, 4, motion(), 0.0);
}

#[test]
fn paint_starfield_normal_canvas_writes_stars() {
    let mut canvas = blank_canvas(40, 16);
    paint_starfield(&mut canvas, 40, 10, motion(), 10.0);
    let stars = canvas
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '*' | '✶' | '✦'))
        .count();
    assert!(stars > 0, "expected stars in canvas");
}

#[test]
fn paint_starfield_overcast_skips_stars() {
    let mut canvas = blank_canvas(40, 16);
    paint_starfield(&mut canvas, 40, 10, motion(), 95.0);
    let stars = canvas
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '*' | '✶' | '✦'))
        .count();
    assert_eq!(stars, 0);
}

#[test]
fn place_celestial_body_small_canvas_no_panic() {
    let mut canvas = blank_canvas(4, 4);
    // w < 4 or horizon_y < 3 → early return
    place_celestial_body(&mut canvas, true, false, 12, 2, 4);
    place_celestial_body(&mut canvas, true, false, 12, 4, 3);
}

#[test]
fn place_celestial_body_normal_canvas_writes_symbol() {
    let mut canvas = blank_canvas(40, 16);
    place_celestial_body(&mut canvas, true, false, 12, 10, 40);
    let has_char = canvas.iter().flatten().any(|c| *c != ' ');
    assert!(has_char, "expected celestial body drawn");
}

#[test]
fn place_celestial_body_night_skips_when_moon_not_visible() {
    let mut canvas = blank_canvas(40, 16);
    place_celestial_body(&mut canvas, false, false, 12, 10, 40);
    let has_char = canvas.iter().flatten().any(|c| *c != ' ');
    assert!(!has_char, "night celestial glyph should not be drawn");
}

#[test]
fn place_celestial_body_night_renders_when_moon_visible() {
    let mut canvas = blank_canvas(40, 16);
    place_celestial_body(&mut canvas, false, true, 22, 10, 40);
    let has_char = canvas.iter().flatten().any(|c| *c != ' ');
    assert!(has_char, "moon should render when visible");
}

#[test]
fn moon_visible_uses_sun_window() {
    use chrono::NaiveDateTime;

    fn fixture_dt(value: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").expect("valid fixture time")
    }

    let day_bundle = bundle_for_category(0, true);
    assert!(!moon_visible(&day_bundle, 23));

    let mut night_bundle = bundle_for_category(0, false);
    night_bundle.daily[0].sunrise = Some(fixture_dt("2026-02-12T06:10"));
    night_bundle.daily[0].sunset = Some(fixture_dt("2026-02-12T17:40"));
    assert!(moon_visible(&night_bundle, 23));
    assert!(moon_visible(&night_bundle, 2));
    assert!(!moon_visible(&night_bundle, 12));
}
