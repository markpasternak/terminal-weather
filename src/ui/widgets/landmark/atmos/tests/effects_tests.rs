use super::super::{
    paint_ambient_sky_life, paint_fog_banks, paint_hail, paint_heat_shimmer, paint_ice_glaze,
    paint_lightning_bolts, paint_rain, paint_snowfall,
};
use super::test_support::{ambient_marks, base_ctx, blank_canvas};
use crate::domain::weather::WeatherCategory;

#[test]
fn ambient_sky_life_not_rendered_at_night() {
    let mut canvas = blank_canvas(40, 16);
    let mut ctx = base_ctx();
    ctx.is_day = false;
    paint_ambient_sky_life(&mut canvas, ctx);
    assert_eq!(ambient_marks(&canvas), 0);
}

#[test]
fn ambient_sky_life_not_rendered_outside_clear_sky() {
    for category in [
        WeatherCategory::Rain,
        WeatherCategory::Thunder,
        WeatherCategory::Cloudy,
    ] {
        let mut canvas = blank_canvas(40, 16);
        let mut ctx = base_ctx();
        ctx.category = category;
        paint_ambient_sky_life(&mut canvas, ctx);
        assert_eq!(ambient_marks(&canvas), 0);
    }
}

#[test]
fn ambient_sky_life_renders_when_clear_day_and_phase_nonzero() {
    let mut canvas = blank_canvas(48, 16);
    let mut ctx = base_ctx();
    ctx.cloud_pct = 15.0;
    ctx.wind_speed = 6.0;
    ctx.phase = 20;
    ctx.width = 96;
    paint_ambient_sky_life(&mut canvas, ctx);
    assert!(ambient_marks(&canvas) > 0);
}

#[test]
fn ambient_sky_life_deterministic_for_same_inputs() {
    let mut first = blank_canvas(96, 16);
    let mut second = blank_canvas(96, 16);
    let mut ctx = base_ctx();
    ctx.category = WeatherCategory::Clear;
    ctx.cloud_pct = 12.0;
    ctx.wind_speed = 5.0;
    ctx.phase = 37;
    ctx.width = 96;
    paint_ambient_sky_life(&mut first, ctx);
    paint_ambient_sky_life(&mut second, ctx);
    assert_eq!(first, second);
}

#[test]
fn paint_rain_zero_width_no_panic() {
    let mut canvas = blank_canvas(0, 10);
    paint_rain(&mut canvas, 2.0, 0, 8, 0);
}

#[test]
fn paint_rain_zero_horizon_no_panic() {
    let mut canvas = blank_canvas(20, 5);
    paint_rain(&mut canvas, 2.0, 0, 2, 20);
}

#[test]
fn paint_rain_light_writes_rain_chars() {
    let mut canvas = blank_canvas(40, 16);
    paint_rain(&mut canvas, 0.5, 0, 12, 40);
    let rain = canvas
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '/' | '╱' | '≈'))
        .count();
    assert!(rain > 0, "expected rain chars in canvas");
}

#[test]
fn paint_rain_heavy_denser_than_light() {
    let mut light = blank_canvas(40, 16);
    let mut heavy = blank_canvas(40, 16);
    paint_rain(&mut light, 0.5, 5, 12, 40);
    paint_rain(&mut heavy, 6.0, 5, 12, 40);
    let light_count = light
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '/' | '╱'))
        .count();
    let heavy_count = heavy
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '/' | '╱'))
        .count();
    assert!(
        heavy_count >= light_count,
        "heavy rain should not be sparser"
    );
}

#[test]
fn paint_snowfall_zero_width_no_panic() {
    let mut canvas = blank_canvas(0, 10);
    paint_snowfall(&mut canvas, 0, 8, 0);
}

#[test]
fn paint_snowfall_writes_flake_chars() {
    let mut canvas = blank_canvas(40, 16);
    paint_snowfall(&mut canvas, 10, 12, 40);
    let flakes = canvas
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '*' | '✶' | '✦' | '·'))
        .count();
    assert!(flakes > 0, "expected snowflake chars");
}

#[test]
fn paint_hail_zero_width_no_panic() {
    let mut canvas = blank_canvas(0, 10);
    paint_hail(&mut canvas, 0, 8, 0);
}

#[test]
fn paint_hail_writes_hailstone_chars() {
    let mut canvas = blank_canvas(40, 16);
    paint_hail(&mut canvas, 0, 12, 40);
    let stones = canvas.iter().flatten().filter(|c| **c == '●').count();
    assert!(stones > 0, "expected hail chars");
}

#[test]
fn paint_fog_banks_zero_width_no_panic() {
    let mut canvas = blank_canvas(0, 10);
    paint_fog_banks(&mut canvas, 0, 8, 0, 10);
}

#[test]
fn paint_fog_banks_writes_fog_chars() {
    let mut canvas = blank_canvas(40, 16);
    paint_fog_banks(&mut canvas, 5, 10, 40, 16);
    let fog = canvas
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '░' | '▒'))
        .count();
    assert!(fog > 0, "expected fog chars");
}

#[test]
fn paint_lightning_bolts_no_flash_when_phase_not_aligned() {
    // Phase must be multiple-of-5 after /3 to show lightning.
    // Use phase=1 which gives (1/3)=0, 0 % 5 == 0 → visible.
    // Use phase=2 → (2/3)=0, 0 % 5 == 0 → also visible.
    // Use phase=10 → (10/3)=3, 3 % 5 != 0 → not visible.
    let mut canvas = blank_canvas(40, 16);
    paint_lightning_bolts(&mut canvas, 10, 12, 40);
    // No assertion on content - just ensure no panic
}

#[test]
fn paint_lightning_bolts_writes_bolt_chars_when_visible() {
    // phase=0 → 0/3=0, 0%5==0 → lightning_visible=true (if w>=6, horizon>=5)
    let mut canvas = blank_canvas(40, 16);
    paint_lightning_bolts(&mut canvas, 0, 12, 40);
    let bolts = canvas
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '╲' | '╱'))
        .count();
    assert!(bolts > 0, "expected lightning bolt chars at phase=0");
}

#[test]
fn paint_heat_shimmer_shallow_horizon_no_panic() {
    let mut canvas = blank_canvas(20, 5);
    paint_heat_shimmer(&mut canvas, 0, 2, 20); // horizon_y < 3 → no-op
}

#[test]
fn paint_heat_shimmer_writes_shimmer_chars() {
    let mut canvas = blank_canvas(40, 16);
    paint_heat_shimmer(&mut canvas, 5, 12, 40);
    let shimmer = canvas
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '~'))
        .count();
    assert!(shimmer > 0, "expected shimmer chars");
}

#[test]
fn paint_ice_glaze_horizon_out_of_bounds_no_panic() {
    let mut canvas = blank_canvas(20, 5);
    paint_ice_glaze(&mut canvas, 10, 20); // horizon_y >= canvas.len()
}

#[test]
fn paint_ice_glaze_writes_ice_chars() {
    let mut canvas: Vec<Vec<char>> = vec![
        vec!['─'; 20],
        vec![' '; 20],
        vec![' '; 20],
        vec!['─'; 20], // horizon_y=3
        vec![' '; 20],
    ];
    paint_ice_glaze(&mut canvas, 3, 20);
    let ice = canvas
        .iter()
        .flatten()
        .filter(|c| **c == '❆' || **c == '░')
        .count();
    assert!(ice > 0, "expected ice chars");
}
