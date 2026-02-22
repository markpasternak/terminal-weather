use crate::domain::weather::WeatherCategory;

use super::{
    WeatherEffectsContext, paint_fog_banks, paint_hail, paint_heat_shimmer, paint_ice_glaze,
    paint_lightning_bolts, paint_rain, paint_snowfall,
};

pub(super) fn paint_weather_effects(canvas: &mut [Vec<char>], ctx: WeatherEffectsContext) {
    let is_freezing = matches!(ctx.weather_code, 56 | 57 | 66 | 67);
    let has_hail = matches!(ctx.weather_code, 96 | 99);
    match ctx.category {
        WeatherCategory::Clear => paint_clear_effects(canvas, ctx),
        WeatherCategory::Rain => paint_rain_effects(canvas, ctx, is_freezing),
        WeatherCategory::Snow => paint_snowfall(canvas, ctx.phase, ctx.horizon_y, ctx.width),
        WeatherCategory::Fog => {
            paint_fog_banks(canvas, ctx.phase, ctx.horizon_y, ctx.width, ctx.height);
        }
        WeatherCategory::Thunder => paint_thunder_effects(canvas, ctx, has_hail),
        _ => {}
    }
}

fn paint_clear_effects(canvas: &mut [Vec<char>], ctx: WeatherEffectsContext) {
    if ctx.is_day && ctx.temp_c >= 26.0 {
        paint_heat_shimmer(canvas, ctx.phase, ctx.horizon_y, ctx.width);
    }
}

fn paint_rain_effects(canvas: &mut [Vec<char>], ctx: WeatherEffectsContext, is_freezing: bool) {
    paint_rain(canvas, ctx.precip_mm, ctx.phase, ctx.horizon_y, ctx.width);
    if is_freezing {
        paint_ice_glaze(canvas, ctx.horizon_y, ctx.width);
    }
}

fn paint_thunder_effects(canvas: &mut [Vec<char>], ctx: WeatherEffectsContext, has_hail: bool) {
    paint_rain(
        canvas,
        ctx.precip_mm.max(1.0),
        ctx.phase,
        ctx.horizon_y,
        ctx.width,
    );
    paint_lightning_bolts(canvas, ctx.phase, ctx.horizon_y, ctx.width);
    if has_hail {
        paint_hail(canvas, ctx.phase, ctx.horizon_y, ctx.width);
    }
}
