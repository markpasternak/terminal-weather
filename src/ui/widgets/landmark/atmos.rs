use chrono::Timelike;

use crate::domain::weather::{ForecastBundle, WeatherCategory, weather_code_to_category};
use crate::ui::widgets::landmark::compact::compact_condition_scene;
use crate::ui::widgets::landmark::shared::{canvas_to_lines, paint_char};
use crate::ui::widgets::landmark::{LandmarkScene, scene_name, tint_for_category};

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

    let temps = bundle
        .hourly
        .iter()
        .take(24)
        .filter_map(|hour| hour.temperature_2m_c)
        .collect::<Vec<_>>();
    if temps.len() < 2 {
        return compact_condition_scene(category, bundle.current.is_day, width, height);
    }

    let cloud_pct = bundle.current.cloud_cover.clamp(0.0, 100.0);
    let precip_mm = bundle.current.precipitation_mm.max(0.0);
    let wind_speed = bundle.current.wind_speed_10m.max(0.0);

    let mut canvas = vec![vec![' '; w]; h];

    // --- Layer 1: Horizon line and terrain ---
    let horizon_y = (h.saturating_mul(72) / 100).clamp(4, h.saturating_sub(2));
    let terrain_amp = (horizon_y / 4).clamp(1, 6);
    let min_temp = temps.iter().copied().fold(f32::INFINITY, f32::min);
    let max_temp = temps.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max_temp - min_temp).max(0.2);

    let terrain_top = compute_terrain(w, &temps, min_temp, span, horizon_y, terrain_amp);
    paint_terrain(&mut canvas, &terrain_top, horizon_y, w, h);

    // --- Layer 2: Atmospheric haze near horizon ---
    paint_horizon_haze(&mut canvas, horizon_y, w);

    // --- Layer 3: Sky content ---
    let phase = if animate {
        (frame_tick % 512) as usize
    } else {
        0
    };

    if !bundle.current.is_day {
        paint_starfield(&mut canvas, w, horizon_y, phase);
    }

    // Celestial body
    let hour = bundle
        .hourly
        .first()
        .map_or(12, |hour| hour.time.hour() as usize)
        % 24;
    place_celestial_body(&mut canvas, bundle.current.is_day, hour, horizon_y, w);

    // --- Layer 4: Clouds ---
    paint_cloud_layer(&mut canvas, cloud_pct, wind_speed, phase, horizon_y, w);
    paint_ambient_sky_life(
        &mut canvas,
        AmbientSkyLifeContext {
            category,
            is_day: bundle.current.is_day,
            cloud_pct,
            wind_speed,
            phase,
            animate,
            horizon_y,
            width: w,
        },
    );

    // --- Layer 5: Weather phenomena ---
    let code = bundle.current.weather_code;
    let is_freezing = matches!(code, 56 | 57 | 66 | 67);
    let has_hail = matches!(code, 96 | 99);
    match category {
        WeatherCategory::Clear => {
            // Subtle atmospheric shimmer on clear days
            if bundle.current.is_day {
                paint_heat_shimmer(&mut canvas, phase, horizon_y, w);
            }
        }
        WeatherCategory::Rain => {
            paint_rain(&mut canvas, precip_mm, phase, horizon_y, w);
            if is_freezing {
                paint_ice_glaze(&mut canvas, horizon_y, w);
            }
        }
        WeatherCategory::Snow => paint_snowfall(&mut canvas, phase, horizon_y, w),
        WeatherCategory::Fog => paint_fog_banks(&mut canvas, phase, horizon_y, w, h),
        WeatherCategory::Thunder => {
            paint_rain(&mut canvas, precip_mm.max(1.0), phase, horizon_y, w);
            paint_lightning_bolts(&mut canvas, phase, horizon_y, w);
            if has_hail {
                paint_hail(&mut canvas, phase, horizon_y, w);
            }
        }
        _ => {}
    }

    LandmarkScene {
        label: format!(
            "Atmos Canvas · {}",
            scene_name(category, bundle.current.is_day)
        ),
        lines: canvas_to_lines(canvas, w),
        tint: tint_for_category(category),
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
    // Smooth terrain edge using graduated block characters
    for (x, &top) in terrain_top.iter().enumerate().take(w) {
        for (y, row) in canvas.iter_mut().enumerate().take(h) {
            if y < top {
                continue;
            }
            row[x] = if y == top {
                '▁'
            } else if y == top + 1 {
                '▃'
            } else if y <= horizon_y {
                '▅'
            } else {
                '█'
            };
        }
    }
    // Horizon line overlay
    if horizon_y < h {
        for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(w) {
            if terrain_top[x] > horizon_y {
                *cell = '─';
            }
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

fn paint_ambient_sky_life(canvas: &mut [Vec<char>], ctx: AmbientSkyLifeContext) {
    if !ctx.animate
        || !ctx.is_day
        || ctx.phase == 0
        || !matches!(
            ctx.category,
            WeatherCategory::Clear | WeatherCategory::Cloudy
        )
        || ctx.cloud_pct > 85.0
        || ctx.width < 32
        || ctx.horizon_y < 6
    {
        return;
    }

    let sky_rows = ctx.horizon_y.saturating_sub(2);
    if sky_rows <= 1 {
        return;
    }

    let bird_count = (ctx.width / 24).clamp(2, 5);
    let bird_cycle = ctx.width + 12;
    for i in 0..bird_count {
        let speed = 1 + (i % 3);
        let x = ((ctx.phase * speed + i * 17) % bird_cycle) as isize - 6;
        let lane = 1 + ((i * 3 + ctx.phase / 19) % sky_rows.saturating_sub(1).max(1));
        let glyph = if ((ctx.phase / 6) + i).is_multiple_of(2) {
            'v'
        } else {
            'V'
        };
        paint_char(canvas, x, lane as isize, glyph, false);
    }

    let plane_cycle = 220;
    let plane_window = 90;
    let window_phase = ctx.phase % plane_cycle;
    if window_phase >= plane_window {
        return;
    }

    let plane = ['=', '=', '>'];
    let plane_len = plane.len();
    let progress = window_phase as f32 / (plane_window.saturating_sub(1)) as f32;
    let wind_drift = ((ctx.wind_speed / 18.0).round() as isize).clamp(0, 3);
    let plane_span = ctx.width + plane_len + 8;
    let plane_x =
        (progress * plane_span as f32).round() as isize - (plane_len as isize + 4) + wind_drift;
    let lane_count = ctx.horizon_y.saturating_sub(4).max(1);
    let plane_y = 1 + ((ctx.phase / plane_cycle + ctx.width / 9) % lane_count);

    for (idx, ch) in plane.iter().enumerate() {
        paint_char(canvas, plane_x + idx as isize, plane_y as isize, *ch, false);
    }

    let contrail_len = (1 + (ctx.wind_speed as usize / 20)).clamp(1, 3);
    for step in 1..=contrail_len {
        paint_char(
            canvas,
            plane_x - step as isize,
            plane_y as isize,
            '-',
            false,
        );
    }
}

fn draw_ambient_cloud(
    canvas: &mut [Vec<char>],
    cx: usize,
    cy: usize,
    cloud_w: usize,
    rows: usize,
    canvas_w: usize,
    horizon_y: usize,
) {
    // Multi-row cloud with density falloff from center
    // Top row: lighter/thinner  ░░▒▒▒░░
    // Middle row: denser         ▒▓▓▓▓▓▒
    // Bottom row: wispy          ░░▒░░
    let patterns: &[&[char]] = if rows >= 3 {
        &[
            &[' ', '░', '░', '▒', '▒', '░', '░', ' '],
            &['░', '▒', '▓', '▓', '▓', '▓', '▒', '░'],
            &[' ', ' ', '░', '▒', '▒', '░', ' ', ' '],
        ]
    } else {
        &[
            &[' ', '░', '▒', '▒', '▒', '░', ' '],
            &['░', '▒', '▓', '▓', '▒', '░', ' '],
        ]
    };

    for (row_idx, pattern) in patterns.iter().enumerate() {
        let y = cy + row_idx;
        if y >= horizon_y || y >= canvas.len() {
            break;
        }
        let pat_len = pattern.len();
        for col in 0..cloud_w {
            let x = cx.wrapping_sub(cloud_w / 2).wrapping_add(col);
            if x >= canvas_w {
                continue;
            }
            let pat_idx = (col * pat_len) / cloud_w.max(1);
            let ch = pattern[pat_idx.min(pat_len - 1)];
            if ch != ' ' && canvas[y][x] == ' ' {
                canvas[y][x] = ch;
            }
        }
    }
}

fn paint_rain(canvas: &mut [Vec<char>], precip_mm: f32, phase: usize, horizon_y: usize, w: usize) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    // Rain density scales with precipitation
    let density = if precip_mm >= 5.0 {
        2 // heavy: every 2nd column
    } else if precip_mm >= 1.0 {
        3 // moderate: every 3rd
    } else {
        5 // light: every 5th
    };

    let h = canvas.len();
    for x in 0..w {
        if !(x + phase).is_multiple_of(density) {
            continue;
        }
        // Multiple rain drops per column for density
        let drops = if precip_mm >= 5.0 {
            3
        } else if precip_mm >= 1.0 {
            2
        } else {
            1
        };
        for d in 0..drops {
            let y_offset = (phase + x * 3 + d * 4) % horizon_y.max(2);
            let y = 1 + y_offset;
            if y < h && y < horizon_y {
                let ch = if precip_mm >= 3.0 { '/' } else { '╱' };
                if let Some(cell) = canvas.get_mut(y).and_then(|row| row.get_mut(x))
                    && matches!(*cell, ' ' | '·')
                {
                    *cell = ch;
                }
            }
        }
    }
    // Splash marks on terrain
    if horizon_y < h {
        for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(w) {
            if (x + phase / 2).is_multiple_of(density + 1) && matches!(*cell, '─' | ' ') {
                *cell = '.';
            }
        }
    }
}

fn paint_snowfall(canvas: &mut [Vec<char>], phase: usize, horizon_y: usize, w: usize) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    let h = canvas.len();
    let flakes = ['·', '*', '✧', '·', '·', '*'];
    // Snow drifts gently — multiple layers with different speeds
    for layer in 0..3 {
        let speed = layer + 1;
        let spacing = 3 + layer;
        for x in 0..w {
            if (x + layer * 7) % spacing != 0 {
                continue;
            }
            let y_off = (phase * speed / 2 + x * 5 + layer * 11) % horizon_y.max(2);
            let y = 1 + y_off;
            if y < h && y < horizon_y && canvas[y][x] == ' ' {
                let flake = flakes[(x + layer + phase) % flakes.len()];
                canvas[y][x] = flake;
            }
        }
    }
    // Snow accumulation on terrain
    let top = horizon_y.saturating_sub(1);
    if top < h {
        for cell in canvas[top].iter_mut().take(w) {
            if matches!(*cell, '▁' | ' ') {
                *cell = '∴';
            }
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn paint_fog_banks(canvas: &mut [Vec<char>], phase: usize, horizon_y: usize, w: usize, h: usize) {
    if w == 0 {
        return;
    }
    // Rolling fog bands across the full scene with varying density
    let band_count = 4;
    for band in 0..band_count {
        let base_y = horizon_y.saturating_sub(3) + band;
        if base_y >= h {
            continue;
        }
        let drift = (phase + band * 7) % w;
        let density_chars = ['░', '░', '▒', '░'];
        for x in 0..w {
            let shifted = (x + drift) % w;
            // Sine-wave fog density
            let wave = ((shifted as f32 / w as f32) * std::f32::consts::PI * 3.0).sin();
            if wave > -0.2 && canvas[base_y][x] == ' ' {
                let idx = ((wave + 1.0) / 2.0 * (density_chars.len() - 1) as f32).round() as usize;
                canvas[base_y][x] = density_chars[idx.min(density_chars.len() - 1)];
            }
        }
    }
    // Upper mist wisps
    for y in 2..horizon_y.saturating_sub(3) {
        if y >= h {
            break;
        }
        for x in 0..w {
            if (x + y + phase / 3).is_multiple_of(7) && canvas[y][x] == ' ' {
                canvas[y][x] = '·';
            }
        }
    }
}

fn paint_lightning_bolts(canvas: &mut [Vec<char>], phase: usize, horizon_y: usize, w: usize) {
    if w < 6 || horizon_y < 5 {
        return;
    }
    // Flash visible only every few frames for dramatic effect
    let flash_on = (phase / 3).is_multiple_of(5);
    if !flash_on {
        return;
    }
    // Draw 1-2 jagged lightning bolts
    let bolt_count = 1 + (phase / 7) % 2;
    for b in 0..bolt_count {
        let start_x = (w / 3 + b * w / 3 + phase % (w / 4 + 1)).min(w.saturating_sub(3));
        let mut x = start_x;
        for (y, row) in canvas
            .iter_mut()
            .enumerate()
            .take(horizon_y.saturating_sub(1))
            .skip(1)
        {
            if x >= w {
                break;
            }
            let ch = if y % 2 == 0 { '╲' } else { '╱' };
            row[x] = ch;
            // Zigzag
            if y % 2 == 0 && x + 1 < w {
                x += 1;
            } else {
                x = x.saturating_sub(1);
            }
        }
    }
}

fn paint_heat_shimmer(canvas: &mut [Vec<char>], phase: usize, horizon_y: usize, w: usize) {
    // Subtle rising heat distortion near the terrain on clear hot days
    if horizon_y < 3 {
        return;
    }
    let shimmer_band = horizon_y.saturating_sub(2)..horizon_y;
    for y in shimmer_band {
        if y >= canvas.len() {
            break;
        }
        for (x, cell) in canvas[y].iter_mut().enumerate().take(w) {
            let wave = ((x + phase) as f32 * 0.4).sin();
            if wave > 0.6 && *cell == ' ' {
                *cell = if (x + phase).is_multiple_of(3) {
                    '.'
                } else {
                    ','
                };
            }
        }
    }
}

fn paint_ice_glaze(canvas: &mut [Vec<char>], horizon_y: usize, w: usize) {
    // Ice crystals on terrain surface for freezing rain/drizzle
    if horizon_y >= canvas.len() {
        return;
    }
    for x in 0..w {
        if x % 2 == 0 {
            let y = horizon_y;
            if y < canvas.len() && matches!(canvas[y][x], '─' | '.' | ',' | ' ') {
                canvas[y][x] = '❆';
            }
        }
        // Ice accumulation just above terrain
        let above = horizon_y.saturating_sub(1);
        if above < canvas.len() && x % 4 == 0 && canvas[above][x] == ' ' {
            canvas[above][x] = '·';
        }
    }
}

fn paint_hail(canvas: &mut [Vec<char>], phase: usize, horizon_y: usize, w: usize) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    let h = canvas.len();
    // Hailstones fall straighter and harder than rain
    for x in 0..w {
        if !(x + phase).is_multiple_of(4) {
            continue;
        }
        let y = 1 + (phase + x * 3) % horizon_y.max(2);
        if y < h
            && y < horizon_y
            && let Some(cell) = canvas.get_mut(y).and_then(|row| row.get_mut(x))
            && matches!(*cell, ' ' | '·')
        {
            *cell = 'o';
        }
    }
    // Bounce marks on terrain
    if horizon_y < h {
        for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(w) {
            if (x + phase / 2).is_multiple_of(5) && *cell == '─' {
                *cell = '•';
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
