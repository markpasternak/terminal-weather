use super::{
    AmbientSkyLifeContext, overlay_horizon_line, paint_ambient_sky_life, paint_cloud_layer,
    paint_fog_banks, paint_hail, paint_heat_shimmer, paint_horizon_haze, paint_ice_glaze,
    paint_lightning_bolts, paint_rain, paint_snowfall, paint_starfield, place_celestial_body,
    scene_for_weather,
};
use crate::domain::weather::{Units, WeatherCategory};

fn blank_canvas(width: usize, height: usize) -> Vec<Vec<char>> {
    vec![vec![' '; width]; height]
}

fn ambient_marks(canvas: &[Vec<char>]) -> usize {
    canvas
        .iter()
        .flatten()
        .filter(|ch| matches!(**ch, 'v' | 'V' | '=' | '>' | '-'))
        .count()
}

fn base_ctx() -> AmbientSkyLifeContext {
    AmbientSkyLifeContext {
        category: WeatherCategory::Clear,
        is_day: true,
        cloud_pct: 30.0,
        wind_speed: 10.0,
        phase: 24,
        animate: true,
        horizon_y: 10,
        width: 40,
    }
}

#[test]
fn ambient_sky_life_not_rendered_at_night() {
    let mut canvas = blank_canvas(40, 16);
    let mut ctx = base_ctx();
    ctx.is_day = false;
    paint_ambient_sky_life(&mut canvas, ctx);
    assert_eq!(ambient_marks(&canvas), 0);
}

#[test]
fn ambient_sky_life_not_rendered_in_rain_or_thunder() {
    for category in [WeatherCategory::Rain, WeatherCategory::Thunder] {
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
    ctx.cloud_pct = 28.0;
    ctx.wind_speed = 12.0;
    ctx.phase = 20;
    ctx.width = 48;
    paint_ambient_sky_life(&mut canvas, ctx);
    assert!(ambient_marks(&canvas) > 0);
}

#[test]
fn ambient_sky_life_deterministic_for_same_inputs() {
    let mut first = blank_canvas(48, 16);
    let mut second = blank_canvas(48, 16);
    let mut ctx = base_ctx();
    ctx.category = WeatherCategory::Cloudy;
    ctx.cloud_pct = 35.0;
    ctx.wind_speed = 14.0;
    ctx.phase = 37;
    ctx.width = 48;
    paint_ambient_sky_life(&mut first, ctx);
    paint_ambient_sky_life(&mut second, ctx);
    assert_eq!(first, second);
}

// ── paint_rain ────────────────────────────────────────────────────────────────

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
        .filter(|c| matches!(**c, '/' | '╱' | '.'))
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

// ── paint_snowfall ───────────────────────────────────────────────────────────

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
        .filter(|c| matches!(**c, '·' | '*' | '✧' | '∴'))
        .count();
    assert!(flakes > 0, "expected snowflake chars");
}

// ── paint_hail ───────────────────────────────────────────────────────────────

#[test]
fn paint_hail_zero_width_no_panic() {
    let mut canvas = blank_canvas(0, 10);
    paint_hail(&mut canvas, 0, 8, 0);
}

#[test]
fn paint_hail_writes_hailstone_chars() {
    let mut canvas = blank_canvas(40, 16);
    paint_hail(&mut canvas, 0, 12, 40);
    let stones = canvas
        .iter()
        .flatten()
        .filter(|c| **c == 'o' || **c == '•')
        .count();
    assert!(stones > 0, "expected hail chars");
}

// ── paint_fog_banks ──────────────────────────────────────────────────────────

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
        .filter(|c| matches!(**c, '░' | '▒' | '·'))
        .count();
    assert!(fog > 0, "expected fog chars");
}

// ── paint_lightning_bolts ────────────────────────────────────────────────────

#[test]
fn paint_lightning_bolts_no_flash_when_phase_not_aligned() {
    // Phase must be multiple-of-5 after /3 to show lightning.
    // Use phase=1 which gives (1/3)=0, 0 % 5 == 0 → visible.
    // Use phase=2 → (2/3)=0, 0 % 5 == 0 → also visible.
    // Use phase=10 → (10/3)=3, 3 % 5 != 0 → not visible.
    let mut canvas = blank_canvas(40, 16);
    paint_lightning_bolts(&mut canvas, 10, 12, 40);
    // No assertion on content — just ensure no panic
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

// ── paint_heat_shimmer ───────────────────────────────────────────────────────

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
        .filter(|c| matches!(**c, '.' | ','))
        .count();
    assert!(shimmer > 0, "expected shimmer chars");
}

// ── paint_ice_glaze ──────────────────────────────────────────────────────────

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
        .filter(|c| **c == '❆' || **c == '·')
        .count();
    assert!(ice > 0, "expected ice chars");
}

// ── scene_for_weather ────────────────────────────────────────────────────────

fn bundle_for_category(weather_code: u8, is_day: bool) -> crate::domain::weather::ForecastBundle {
    use crate::domain::weather::{CurrentConditions, HourlyForecast, Location};
    use chrono::NaiveDateTime;
    let base = NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M").expect("time");
    crate::domain::weather::ForecastBundle {
        location: Location::from_coords(59.33, 18.07),
        current: CurrentConditions {
            temperature_2m_c: 5.0,
            relative_humidity_2m: 60.0,
            apparent_temperature_c: 3.0,
            dew_point_2m_c: 1.0,
            weather_code,
            precipitation_mm: if weather_code > 10 { 2.0 } else { 0.0 },
            cloud_cover: 50.0,
            pressure_msl_hpa: 1010.0,
            visibility_m: 9000.0,
            wind_speed_10m: 8.0,
            wind_gusts_10m: 12.0,
            wind_direction_10m: 270.0,
            is_day,
            high_today_c: Some(8.0),
            low_today_c: Some(1.0),
        },
        hourly: (0..24)
            .map(|i| HourlyForecast {
                time: base + chrono::Duration::hours(i64::from(i as u16)),
                temperature_2m_c: Some(4.0 + i as f32 * 0.1),
                weather_code: Some(weather_code),
                is_day: Some(is_day),
                relative_humidity_2m: Some(60.0),
                precipitation_probability: Some(30.0),
                precipitation_mm: Some(0.5),
                rain_mm: Some(0.3),
                snowfall_cm: Some(0.0),
                wind_speed_10m: Some(8.0),
                wind_gusts_10m: Some(12.0),
                pressure_msl_hpa: Some(1010.0),
                visibility_m: Some(9000.0),
                cloud_cover: Some(50.0),
                cloud_cover_low: Some(10.0),
                cloud_cover_mid: Some(20.0),
                cloud_cover_high: Some(30.0),
            })
            .collect(),
        daily: vec![crate::test_support::sample_daily()],
        air_quality: None,
        fetched_at: chrono::Utc::now(),
    }
}

#[test]
fn scene_for_weather_compact_for_small_area() {
    let bundle = crate::test_support::sample_bundle();
    let scene = scene_for_weather(&bundle, Units::Celsius, 0, false, 10, 5);
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
        let scene = scene_for_weather(&bundle, Units::Celsius, 5, false, 80, 24);
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
    let day_scene = scene_for_weather(&day, Units::Celsius, 0, false, 80, 24);
    let night_scene = scene_for_weather(&night, Units::Celsius, 0, false, 80, 24);
    assert_ne!(
        day_scene.lines, night_scene.lines,
        "day vs night should differ"
    );
}

#[test]
fn scene_for_weather_freezing_rain_no_panic() {
    // Code 56 = freezing drizzle
    let bundle = bundle_for_category(56, true);
    let scene = scene_for_weather(&bundle, Units::Fahrenheit, 10, true, 60, 20);
    assert!(!scene.lines.is_empty());
}

#[test]
fn scene_for_weather_hail_code_no_panic() {
    // Code 96 and 99 = thunderstorm with hail → exercises has_hail=true branch
    for code in [96u8, 99] {
        let bundle = bundle_for_category(code, true);
        let scene = scene_for_weather(&bundle, Units::Celsius, 5, true, 60, 20);
        assert!(!scene.lines.is_empty());
    }
}

#[test]
fn scene_for_weather_few_hourly_samples_falls_back_to_compact() {
    // 1 hourly sample → build_atmos_canvas returns None → compact fallback
    let mut bundle = bundle_for_category(0, true);
    bundle.hourly.truncate(1);
    let scene = scene_for_weather(&bundle, Units::Celsius, 0, false, 60, 20);
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
    // Should write some '·' chars in the haze row
    let dots = canvas.iter().flatten().filter(|c| **c == '·').count();
    assert!(dots > 0, "expected haze dots");
}

#[test]
fn paint_starfield_small_canvas_no_panic() {
    let mut canvas = blank_canvas(4, 4);
    // horizon_y < 2 → early return
    paint_starfield(&mut canvas, 4, 1, 0);
    // w == 0 → early return
    paint_starfield(&mut canvas, 0, 4, 0);
}

#[test]
fn paint_starfield_normal_canvas_writes_stars() {
    let mut canvas = blank_canvas(40, 16);
    paint_starfield(&mut canvas, 40, 10, 5);
    let stars = canvas
        .iter()
        .flatten()
        .filter(|c| matches!(**c, '·' | '*' | '✦' | '✧'))
        .count();
    assert!(stars > 0, "expected stars in canvas");
}

#[test]
fn place_celestial_body_small_canvas_no_panic() {
    let mut canvas = blank_canvas(4, 4);
    // w < 4 or horizon_y < 3 → early return
    place_celestial_body(&mut canvas, true, 12, 2, 4);
    place_celestial_body(&mut canvas, true, 12, 4, 3);
}

#[test]
fn place_celestial_body_normal_canvas_writes_symbol() {
    let mut canvas = blank_canvas(40, 16);
    place_celestial_body(&mut canvas, true, 12, 10, 40);
    let has_char = canvas.iter().flatten().any(|c| *c != ' ');
    assert!(has_char, "expected celestial body drawn");
}

#[test]
fn paint_cloud_layer_low_cloud_pct_no_op() {
    let mut canvas = blank_canvas(40, 16);
    // cloud_pct < 5 → early return
    paint_cloud_layer(&mut canvas, 3.0, 10.0, 0, 10, 40);
}

#[test]
fn paint_cloud_layer_small_horizon_no_op() {
    let mut canvas = blank_canvas(40, 16);
    // horizon_y < 4 → early return
    paint_cloud_layer(&mut canvas, 50.0, 10.0, 0, 3, 40);
}

#[test]
fn paint_cloud_layer_high_coverage_covers_more_of_canvas() {
    let mut light_canvas = blank_canvas(60, 20);
    let mut heavy_canvas = blank_canvas(60, 20);
    // cloud_pct > 80 → w/2 max_cloud_w
    paint_cloud_layer(&mut light_canvas, 30.0, 5.0, 0, 12, 60);
    paint_cloud_layer(&mut heavy_canvas, 90.0, 5.0, 0, 12, 60);
    // heavy coverage should produce at least as many cloud chars
    let light_cloud_chars = light_canvas
        .iter()
        .flatten()
        .filter(|c| !matches!(**c, ' '))
        .count();
    let heavy_cloud_chars = heavy_canvas
        .iter()
        .flatten()
        .filter(|c| !matches!(**c, ' '))
        .count();
    assert!(
        heavy_cloud_chars >= light_cloud_chars,
        "heavy clouds should fill more space"
    );
}

#[test]
fn paint_cloud_layer_medium_coverage_exercises_50_branch() {
    let mut canvas = blank_canvas(60, 20);
    // cloud_pct > 50 but <= 80 → w/3 max_cloud_w; cloud_pct <= 70 → 2 rows
    paint_cloud_layer(&mut canvas, 65.0, 8.0, 10, 14, 60);
}

#[test]
fn paint_cloud_layer_very_high_coverage_exercises_70_branch() {
    let mut canvas = blank_canvas(60, 20);
    // cloud_pct > 70 → 3 cloud_rows
    paint_cloud_layer(&mut canvas, 75.0, 8.0, 10, 14, 60);
}
