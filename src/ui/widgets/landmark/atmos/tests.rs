use super::{AmbientSkyLifeContext, paint_ambient_sky_life};
use crate::domain::weather::WeatherCategory;

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
