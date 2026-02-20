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

#[test]
fn ambient_sky_life_not_rendered_at_night() {
    let mut canvas = blank_canvas(40, 16);
    paint_ambient_sky_life(
        &mut canvas,
        AmbientSkyLifeContext {
            category: WeatherCategory::Clear,
            is_day: false,
            cloud_pct: 30.0,
            wind_speed: 10.0,
            phase: 24,
            animate: true,
            horizon_y: 10,
            width: 40,
        },
    );
    assert_eq!(ambient_marks(&canvas), 0);
}

#[test]
fn ambient_sky_life_not_rendered_in_rain_or_thunder() {
    for category in [WeatherCategory::Rain, WeatherCategory::Thunder] {
        let mut canvas = blank_canvas(40, 16);
        paint_ambient_sky_life(
            &mut canvas,
            AmbientSkyLifeContext {
                category,
                is_day: true,
                cloud_pct: 30.0,
                wind_speed: 10.0,
                phase: 24,
                animate: true,
                horizon_y: 10,
                width: 40,
            },
        );
        assert_eq!(ambient_marks(&canvas), 0);
    }
}

#[test]
fn ambient_sky_life_renders_when_clear_day_and_phase_nonzero() {
    let mut canvas = blank_canvas(48, 16);
    paint_ambient_sky_life(
        &mut canvas,
        AmbientSkyLifeContext {
            category: WeatherCategory::Clear,
            is_day: true,
            cloud_pct: 28.0,
            wind_speed: 12.0,
            phase: 20,
            animate: true,
            horizon_y: 10,
            width: 48,
        },
    );
    assert!(ambient_marks(&canvas) > 0);
}

#[test]
fn ambient_sky_life_deterministic_for_same_inputs() {
    let mut first = blank_canvas(48, 16);
    let mut second = blank_canvas(48, 16);
    paint_ambient_sky_life(
        &mut first,
        AmbientSkyLifeContext {
            category: WeatherCategory::Cloudy,
            is_day: true,
            cloud_pct: 35.0,
            wind_speed: 14.0,
            phase: 37,
            animate: true,
            horizon_y: 10,
            width: 48,
        },
    );
    paint_ambient_sky_life(
        &mut second,
        AmbientSkyLifeContext {
            category: WeatherCategory::Cloudy,
            is_day: true,
            cloud_pct: 35.0,
            wind_speed: 14.0,
            phase: 37,
            animate: true,
            horizon_y: 10,
            width: 48,
        },
    );
    assert_eq!(first, second);
}
