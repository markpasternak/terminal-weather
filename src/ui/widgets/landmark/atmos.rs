#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::manual_midpoint
)]

use chrono::Timelike;

use crate::domain::weather::{ForecastBundle, Units, WeatherCategory, weather_code_to_category};
use crate::ui::widgets::landmark::compact::compact_condition_scene;
use crate::ui::widgets::landmark::shared::{canvas_to_lines, paint_char};
use crate::ui::widgets::landmark::{LandmarkScene, scene_name, tint_for_category};

mod effects;
mod effects_dispatch;
mod hud;
mod sky_layers;
mod terrain;

use effects::{
    paint_ambient_sky_life, paint_fog_banks, paint_hail, paint_heat_shimmer, paint_ice_glaze,
    paint_lightning_bolts, paint_rain, paint_snowfall, paint_star_reflections,
};
use effects_dispatch::paint_weather_effects;
use hud::{atmos_context_line, paint_hud_badge};
use sky_layers::{
    is_night, moon_visible, paint_cloud_layer, paint_starfield, place_celestial_body,
};
use terrain::{compute_terrain, paint_horizon_haze, paint_terrain};

#[must_use]
pub fn scene_for_weather(
    bundle: &ForecastBundle,
    units: Units,
    frame_tick: u64,
    animate: bool,
    width: u16,
    height: u16,
) -> LandmarkScene {
    let w = width as usize;
    let h = height as usize;
    let category = weather_code_to_category(bundle.current.weather_code);

    if w < 22 || h < 8 {
        return compact_condition_scene(category, bundle.current.is_day, width, height);
    }

    let Some(mut canvas) = build_atmos_canvas(bundle, frame_tick, animate, category, w, h) else {
        return compact_condition_scene(category, bundle.current.is_day, width, height);
    };

    paint_hud_badge(&mut canvas, bundle, units, w);

    LandmarkScene {
        label: format!(
            "Atmos Canvas · {}",
            scene_name(category, bundle.current.is_day)
        ),
        lines: canvas_to_lines(canvas, w),
        tint: tint_for_category(category),
        context_line: Some(atmos_context_line(bundle, units, category)),
    }
}

fn build_atmos_canvas(
    bundle: &ForecastBundle,
    frame_tick: u64,
    animate: bool,
    category: WeatherCategory,
    width: usize,
    height: usize,
) -> Option<Vec<Vec<char>>> {
    let temps = bundle
        .hourly
        .iter()
        .take(24)
        .filter_map(|hour| hour.temperature_2m_c)
        .collect::<Vec<_>>();
    if temps.len() < 2 {
        return None;
    }

    let mut canvas = vec![vec![' '; width]; height];
    let phase = animation_phase(animate, frame_tick);
    let horizon_y =
        paint_base_atmos_layers(&mut canvas, bundle, &temps, phase, category, width, height);
    paint_effect_atmos_layers(
        &mut canvas,
        bundle,
        AtmosLayerContext {
            category,
            animate,
            phase,
            horizon_y,
            width,
            height,
        },
    );
    Some(canvas)
}

fn paint_base_atmos_layers(
    canvas: &mut [Vec<char>],
    bundle: &ForecastBundle,
    temps: &[f32],
    phase: usize,
    category: WeatherCategory,
    width: usize,
    height: usize,
) -> usize {
    let horizon_y = (height.saturating_mul(72) / 100).clamp(4, height.saturating_sub(2));
    let terrain_amp = (horizon_y / 4).clamp(1, 6);
    let min_temp = temps.iter().copied().fold(f32::INFINITY, f32::min);
    let max_temp = temps.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max_temp - min_temp).max(0.2);

    let terrain_top = compute_terrain(width, temps, min_temp, span, horizon_y, terrain_amp);
    paint_terrain(canvas, &terrain_top, horizon_y, width, height);
    paint_horizon_haze(canvas, horizon_y, width);
    if is_night(bundle) && matches!(category, WeatherCategory::Clear | WeatherCategory::Cloudy) {
        paint_starfield(
            canvas,
            width,
            horizon_y,
            phase,
            bundle.current.cloud_cover.clamp(0.0, 100.0),
        );
    }

    let hour = bundle
        .hourly
        .first()
        .map_or(12, |hour| hour.time.hour() as usize)
        % 24;
    place_celestial_body(
        canvas,
        bundle.current.is_day,
        moon_visible(bundle, hour),
        hour,
        horizon_y,
        width,
    );

    // Nighttime star reflections below the horizon (water/ground reflection).
    if is_night(bundle) && matches!(category, WeatherCategory::Clear | WeatherCategory::Cloudy) {
        paint_star_reflections(canvas, width, horizon_y, height);
    }

    horizon_y
}

#[derive(Debug, Clone, Copy)]
struct AtmosLayerContext {
    category: WeatherCategory,
    animate: bool,
    phase: usize,
    horizon_y: usize,
    width: usize,
    height: usize,
}

fn paint_effect_atmos_layers(
    canvas: &mut [Vec<char>],
    bundle: &ForecastBundle,
    ctx: AtmosLayerContext,
) {
    let cloud_pct = bundle.current.cloud_cover.clamp(0.0, 100.0);
    let wind_speed = bundle.current.wind_speed_10m.max(0.0);
    paint_cloud_layer(
        canvas,
        cloud_pct,
        wind_speed,
        ctx.phase,
        ctx.horizon_y,
        ctx.width,
    );
    paint_ambient_sky_life(
        canvas,
        AmbientSkyLifeContext {
            category: ctx.category,
            is_day: bundle.current.is_day,
            cloud_pct,
            wind_speed,
            phase: ctx.phase,
            animate: ctx.animate,
            horizon_y: ctx.horizon_y,
            width: ctx.width,
        },
    );

    paint_weather_effects(
        canvas,
        WeatherEffectsContext {
            category: ctx.category,
            is_day: bundle.current.is_day,
            weather_code: bundle.current.weather_code,
            temp_c: bundle.current.temperature_2m_c,
            precip_mm: bundle.current.precipitation_mm.max(0.0),
            phase: ctx.phase,
            horizon_y: ctx.horizon_y,
            width: ctx.width,
            height: ctx.height,
        },
    );
}

#[derive(Debug, Clone, Copy)]
struct WeatherEffectsContext {
    category: WeatherCategory,
    is_day: bool,
    weather_code: u8,
    temp_c: f32,
    precip_mm: f32,
    phase: usize,
    horizon_y: usize,
    width: usize,
    height: usize,
}

fn animation_phase(animate: bool, frame_tick: u64) -> usize {
    if animate {
        ((frame_tick / 6) % 512) as usize
    } else {
        0
    }
}

#[derive(Clone, Copy)]
struct AmbientSkyLifeContext {
    category: WeatherCategory,
    is_day: bool,
    cloud_pct: f32,
    wind_speed: f32,
    phase: usize,
    animate: bool,
    horizon_y: usize,
    width: usize,
}

#[cfg(test)]
mod tests;
