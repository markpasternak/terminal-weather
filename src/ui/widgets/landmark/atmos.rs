#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::manual_midpoint
)]

use chrono::Timelike;

use crate::domain::weather::{ForecastBundle, WeatherCategory, weather_code_to_category};
use crate::ui::widgets::landmark::compact::compact_condition_scene;
use crate::ui::widgets::landmark::shared::{canvas_to_lines, paint_char};
use crate::ui::widgets::landmark::{LandmarkScene, scene_name, tint_for_category};

mod effects;

use effects::{
    draw_ambient_cloud, paint_ambient_sky_life, paint_fog_banks, paint_hail, paint_heat_shimmer,
    paint_ice_glaze, paint_lightning_bolts, paint_rain, paint_snowfall,
};

#[must_use]
pub fn scene_for_weather(
    bundle: &ForecastBundle,
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

    paint_hud_badge(&mut canvas, bundle, w);

    LandmarkScene {
        label: format!(
            "Atmos Canvas · {}",
            scene_name(category, bundle.current.is_day)
        ),
        lines: canvas_to_lines(canvas, w),
        tint: tint_for_category(category),
        context_line: Some(atmos_context_line(bundle, category)),
    }
}

fn atmos_context_line(bundle: &ForecastBundle, category: WeatherCategory) -> String {
    let precip_hours: Vec<(usize, f32)> = bundle
        .hourly
        .iter()
        .take(24)
        .enumerate()
        .filter_map(|(i, h)| h.precipitation_mm.filter(|&mm| mm > 0.1).map(|mm| (i, mm)))
        .collect();

    let total_precip: f32 = precip_hours.iter().map(|(_, mm)| mm).sum();

    if !precip_hours.is_empty() {
        let last_rain_hour = precip_hours.last().map_or(0, |(i, _)| *i);
        let now_hour = bundle.hourly.first().map_or(12, |h| h.time.hour() as usize);
        let clearing = (now_hour + last_rain_hour + 1) % 24;
        format!(
            "Rain clearing by {:02}:00 · {:.0}mm expected",
            clearing, total_precip
        )
    } else {
        match category {
            WeatherCategory::Snow => "Snowfall active · dress warm".to_string(),
            WeatherCategory::Fog => "Low visibility · fog advisory".to_string(),
            WeatherCategory::Thunder => "Thunderstorm active · stay indoors".to_string(),
            WeatherCategory::Clear if bundle.current.is_day => {
                let uv = bundle
                    .daily
                    .first()
                    .and_then(|d| d.uv_index_max)
                    .unwrap_or(0.0);
                if uv > 5.0 {
                    format!("Clear skies · UV {uv:.1} high — sunscreen advised")
                } else {
                    "Clear skies · enjoy the day".to_string()
                }
            }
            WeatherCategory::Clear => "Clear night · great for stargazing".to_string(),
            _ => {
                let temp = bundle.current.temperature_2m_c.round() as i32;
                format!("Currently {temp}°C · conditions stable")
            }
        }
    }
}

fn paint_hud_badge(canvas: &mut [Vec<char>], bundle: &ForecastBundle, w: usize) {
    if w < 30 || canvas.len() < 4 {
        return;
    }
    let temp = bundle.current.temperature_2m_c.round() as i32;
    let icon = match weather_code_to_category(bundle.current.weather_code) {
        WeatherCategory::Clear if bundle.current.is_day => '☀',
        WeatherCategory::Clear => '☽',
        WeatherCategory::Cloudy => '☁',
        WeatherCategory::Rain => '☂',
        WeatherCategory::Snow => '❄',
        WeatherCategory::Fog => '≡',
        WeatherCategory::Thunder => '⚡',
        WeatherCategory::Unknown => '?',
    };
    let badge = format!("{temp}°C {icon}");
    let badge_len = badge.chars().count();
    let start_x = w.saturating_sub(badge_len + 1);
    for (i, ch) in badge.chars().enumerate() {
        paint_char(canvas, (start_x + i) as isize, 0, ch, true);
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
    let horizon_y = paint_base_atmos_layers(&mut canvas, bundle, &temps, phase, width, height);
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
    if is_night(bundle) {
        paint_starfield(canvas, width, horizon_y, phase);
    }

    let hour = bundle
        .hourly
        .first()
        .map_or(12, |hour| hour.time.hour() as usize)
        % 24;
    place_celestial_body(canvas, bundle.current.is_day, hour, horizon_y, width);

    // Nighttime star reflections below the horizon (water/ground reflection)
    if is_night(bundle) {
        paint_star_reflections(canvas, width, horizon_y, height);
    }

    horizon_y
}

#[allow(clippy::needless_range_loop)]
fn paint_star_reflections(canvas: &mut [Vec<char>], width: usize, horizon_y: usize, height: usize) {
    // Mirror a sparse subset of stars from above the horizon to below it
    let reflect_depth = (height - horizon_y).min(horizon_y.saturating_sub(1));
    if reflect_depth == 0 || width == 0 {
        return;
    }
    for dy in 1..=reflect_depth {
        let source_y = horizon_y.saturating_sub(dy);
        let target_y = horizon_y + dy;
        if source_y == 0 || target_y >= height {
            continue;
        }
        for x in 0..width {
            // Only reflect every 3rd star for a subtle effect
            if x % 3 != 0 {
                continue;
            }
            if matches!(canvas[source_y][x], '*' | '✦' | '✧')
                && matches!(canvas[target_y][x], '█' | '▅')
            {
                canvas[target_y][x] = '·';
            }
        }
    }
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
    precip_mm: f32,
    phase: usize,
    horizon_y: usize,
    width: usize,
    height: usize,
}

fn animation_phase(animate: bool, frame_tick: u64) -> usize {
    if animate {
        (frame_tick % 512) as usize
    } else {
        0
    }
}

fn is_night(bundle: &ForecastBundle) -> bool {
    !bundle.current.is_day
}

fn paint_weather_effects(canvas: &mut [Vec<char>], ctx: WeatherEffectsContext) {
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
    if ctx.is_day {
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

fn compute_terrain(
    w: usize,
    temps: &[f32],
    min_temp: f32,
    span: f32,
    horizon_y: usize,
    amp: usize,
) -> Vec<usize> {
    let mut tops = vec![horizon_y; w];
    for (x, top) in tops.iter_mut().enumerate() {
        let t = if w <= 1 {
            0.0
        } else {
            x as f32 / (w - 1) as f32
        };
        let pos = t * (temps.len().saturating_sub(1)) as f32;
        let left = pos.floor() as usize;
        let right = pos.ceil().min((temps.len() - 1) as f32) as usize;
        let frac = pos - left as f32;
        let sample = if right > left {
            temps[left] * (1.0 - frac) + temps[right] * frac
        } else {
            temps[left]
        };
        // Add gentle rolling hills with a sine overlay for organic feel
        let norm = ((sample - min_temp) / span).clamp(0.0, 1.0);
        let hill = (t * std::f32::consts::PI * 2.3).sin() * 0.3 + 0.7;
        let peak = (norm * hill * amp as f32).round() as usize;
        *top = horizon_y.saturating_sub(peak).max(1);
    }
    tops
}

fn paint_terrain(
    canvas: &mut [Vec<char>],
    terrain_top: &[usize],
    horizon_y: usize,
    w: usize,
    h: usize,
) {
    for (x, &top) in terrain_top.iter().enumerate().take(w) {
        for (y, row) in canvas.iter_mut().enumerate().take(h) {
            if y < top {
                continue;
            }
            row[x] = terrain_glyph(y, top, horizon_y);
        }
    }
    overlay_horizon_line(canvas, terrain_top, horizon_y, w, h);
}

fn terrain_glyph(y: usize, top: usize, horizon_y: usize) -> char {
    if y == top {
        '▁'
    } else if y == top + 1 {
        '▃'
    } else if y <= horizon_y {
        '▅'
    } else {
        '█'
    }
}

fn overlay_horizon_line(
    canvas: &mut [Vec<char>],
    terrain_top: &[usize],
    horizon_y: usize,
    width: usize,
    height: usize,
) {
    if horizon_y >= height {
        return;
    }
    for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(width) {
        if terrain_top[x] > horizon_y {
            *cell = '─';
        }
    }
}

fn paint_horizon_haze(canvas: &mut [Vec<char>], horizon_y: usize, w: usize) {
    // Thin atmospheric haze band just above the horizon
    let haze_y = horizon_y.saturating_sub(1);
    if haze_y == 0 {
        return;
    }
    for (x, cell) in canvas[haze_y].iter_mut().enumerate().take(w) {
        if *cell == ' ' {
            *cell = if x % 3 == 0 { '·' } else { ' ' };
        }
    }
}

fn paint_starfield(canvas: &mut [Vec<char>], w: usize, horizon_y: usize, phase: usize) {
    if w == 0 || horizon_y < 2 {
        return;
    }
    // Dense star field with depth layers
    let sky_h = horizon_y.saturating_sub(1);
    let star_count = (w * sky_h / 8).max(12);
    let glyphs = ['·', '·', '·', '*', '✦', '✧'];
    for i in 0..star_count {
        let seed = i.wrapping_mul(7919).wrapping_add(31);
        let x = seed % w;
        let y = 1 + (seed / w) % sky_h;
        if y >= horizon_y || x >= w {
            continue;
        }
        if canvas[y][x] != ' ' {
            continue;
        }
        // Twinkle: some stars blink based on phase
        let twinkle = (i + phase / 4).is_multiple_of(7);
        if twinkle {
            continue;
        }
        let glyph_idx = (seed / 3) % glyphs.len();
        canvas[y][x] = glyphs[glyph_idx];
    }
}

fn place_celestial_body(
    canvas: &mut [Vec<char>],
    is_day: bool,
    hour: usize,
    horizon_y: usize,
    w: usize,
) {
    if w < 4 || horizon_y < 3 {
        return;
    }
    // Position based on time of day along a parabolic arc
    let t = hour as f32 / 23.0;
    let x = (t * (w.saturating_sub(3)) as f32).round() as usize + 1;
    let arc = 1.0 - 4.0 * (t - 0.5) * (t - 0.5); // peaks at noon
    let sky_height = horizon_y.saturating_sub(2);
    let y = (horizon_y as f32 - 1.0 - arc * sky_height as f32 * 0.8)
        .round()
        .clamp(1.0, (horizon_y - 1) as f32) as usize;

    let body = if is_day { '◉' } else { '◐' };
    paint_char(canvas, x as isize, y as isize, body, true);

    // Glow around celestial body
    if is_day {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            paint_char(canvas, x as isize + dx, y as isize + dy, '·', false);
        }
        if w >= 50 {
            for (dx, dy) in [
                (-2, 0),
                (2, 0),
                (0, -2),
                (0, 2),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ] {
                paint_char(canvas, x as isize + dx, y as isize + dy, '·', false);
            }
        }
    }
}

fn paint_cloud_layer(
    canvas: &mut [Vec<char>],
    cloud_pct: f32,
    wind_speed: f32,
    phase: usize,
    horizon_y: usize,
    w: usize,
) {
    if cloud_pct < 5.0 || horizon_y < 4 {
        return;
    }

    // Number and size of clouds scale with cloud coverage
    let cloud_count = ((cloud_pct / 15.0).ceil() as usize).clamp(1, 8);
    let max_cloud_w = if cloud_pct > 80.0 {
        w / 2
    } else if cloud_pct > 50.0 {
        w / 3
    } else {
        w / 5
    }
    .clamp(6, 40);
    let cloud_rows = if cloud_pct > 70.0 { 3 } else { 2 };

    // Wind drift offset
    let drift = (phase as f32 * wind_speed.max(3.0) / 40.0) as usize;

    let sky_band = horizon_y.saturating_sub(2);
    for i in 0..cloud_count {
        let seed = i.wrapping_mul(4001).wrapping_add(17);
        let base_x = (seed.wrapping_mul(13) + drift) % (w + max_cloud_w);
        let base_y = 1 + (seed % sky_band.max(1));
        if base_y >= horizon_y.saturating_sub(1) {
            continue;
        }
        let cw = (max_cloud_w / 2) + (seed % (max_cloud_w / 2 + 1));
        draw_ambient_cloud(canvas, base_x, base_y, cw, cloud_rows, w, horizon_y);
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
