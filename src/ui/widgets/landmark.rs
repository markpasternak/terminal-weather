#![allow(
    clippy::needless_range_loop,
    clippy::manual_is_multiple_of,
    clippy::collapsible_if,
    clippy::implicit_saturating_sub
)]

use chrono::{Datelike, Timelike};

use crate::domain::weather::{ForecastBundle, WeatherCategory, weather_code_to_category};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandmarkTint {
    Warm,
    Cool,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct LandmarkScene {
    pub label: String,
    pub lines: Vec<String>,
    pub tint: LandmarkTint,
}

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
        .map(|hour| hour.time.hour() as usize)
        .unwrap_or(12)
        % 24;
    place_celestial_body(&mut canvas, bundle.current.is_day, hour, horizon_y, w);

    // --- Layer 4: Clouds ---
    paint_cloud_layer(&mut canvas, cloud_pct, wind_speed, phase, horizon_y, w);

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
        for y in 0..h {
            if y < top {
                continue;
            }
            canvas[y][x] = if y == top {
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
        for x in 0..w {
            if terrain_top[x] > horizon_y {
                canvas[horizon_y][x] = '─';
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
    for x in 0..w {
        if canvas[haze_y][x] == ' ' {
            canvas[haze_y][x] = if x.is_multiple_of(3) { '·' } else { ' ' };
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
        let twinkle = (i + phase / 4) % 7 == 0;
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
        if (x + phase) % density != 0 {
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
                if canvas[y][x] == ' ' || canvas[y][x] == '·' {
                    canvas[y][x] = ch;
                }
            }
        }
    }
    // Splash marks on terrain
    for x in 0..w {
        if (x + phase / 2) % (density + 1) == 0 && horizon_y < h {
            if canvas[horizon_y][x] == '─' || canvas[horizon_y][x] == ' ' {
                canvas[horizon_y][x] = '~';
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
    for x in 0..w {
        let top = horizon_y.saturating_sub(1);
        if top < h && (canvas[top][x] == '▁' || canvas[top][x] == ' ') {
            canvas[top][x] = '∴';
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
            if (x + y + phase / 3) % 7 == 0 && canvas[y][x] == ' ' {
                canvas[y][x] = '·';
            }
        }
    }
}

fn paint_lightning_bolts(canvas: &mut [Vec<char>], phase: usize, horizon_y: usize, w: usize) {
    if w < 6 || horizon_y < 5 {
        return;
    }
    let h = canvas.len();
    // Flash visible only every few frames for dramatic effect
    let flash_on = (phase / 3) % 5 == 0;
    if !flash_on {
        return;
    }
    // Draw 1-2 jagged lightning bolts
    let bolt_count = 1 + (phase / 7) % 2;
    for b in 0..bolt_count {
        let start_x = (w / 3 + b * w / 3 + phase % (w / 4 + 1)).min(w.saturating_sub(3));
        let mut x = start_x;
        for y in 1..horizon_y.saturating_sub(1) {
            if y >= h || x >= w {
                break;
            }
            let ch = if y % 2 == 0 { '╲' } else { '╱' };
            canvas[y][x] = ch;
            // Zigzag
            if y % 2 == 0 && x + 1 < w {
                x += 1;
            } else if x > 0 {
                x -= 1;
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
        for x in 0..w {
            let wave = ((x + phase) as f32 * 0.4).sin();
            if wave > 0.6 && canvas[y][x] == ' ' {
                canvas[y][x] = '~';
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
        if x.is_multiple_of(2) {
            let y = horizon_y;
            if y < canvas.len() && matches!(canvas[y][x], '─' | '~' | ' ') {
                canvas[y][x] = '❆';
            }
        }
        // Ice accumulation just above terrain
        let above = horizon_y.saturating_sub(1);
        if above < canvas.len() && x.is_multiple_of(4) && canvas[above][x] == ' ' {
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
        if (x + phase) % 4 != 0 {
            continue;
        }
        let y = 1 + (phase + x * 3) % horizon_y.max(2);
        if y < h && y < horizon_y {
            if canvas[y][x] == ' ' || canvas[y][x] == '·' {
                canvas[y][x] = 'o';
            }
        }
    }
    // Bounce marks on terrain
    if horizon_y < h {
        for x in 0..w {
            if (x + phase / 2) % 5 == 0 && canvas[horizon_y][x] == '─' {
                canvas[horizon_y][x] = '•';
            }
        }
    }
}

pub fn scene_for_gauge_cluster(bundle: &ForecastBundle, width: u16, height: u16) -> LandmarkScene {
    let w = width as usize;
    let h = height as usize;
    let category = weather_code_to_category(bundle.current.weather_code);

    let current = &bundle.current;
    let temp_c = current.temperature_2m_c.round() as i32;
    let humidity = current.relative_humidity_2m.clamp(0.0, 100.0);
    let pressure = current.pressure_msl_hpa;
    let wind = current.wind_speed_10m.max(0.0);
    let gust = current.wind_gusts_10m.max(0.0);
    let uv = bundle
        .daily
        .first()
        .and_then(|day| day.uv_index_max)
        .unwrap_or(0.0);
    let vis_km = (current.visibility_m / 1000.0).max(0.0);

    let meter_w = w.saturating_sub(26).clamp(10, 56);
    let pressure_norm = ((pressure - 970.0) / 70.0).clamp(0.0, 1.0);
    let uv_norm = (uv / 12.0).clamp(0.0, 1.0);
    let temp_norm = ((temp_c as f32 + 20.0) / 60.0).clamp(0.0, 1.0);
    let vis_norm = (vis_km / 12.0).clamp(0.0, 1.0);
    let precip_now = current.precipitation_mm.max(0.0);
    let cloud = current.cloud_cover.clamp(0.0, 100.0);
    let left_col_width = if w >= 86 {
        w.saturating_mul(58) / 100
    } else if w >= 74 {
        w.saturating_mul(62) / 100
    } else {
        w
    };

    let sunrise = bundle
        .daily
        .first()
        .and_then(|d| d.sunrise)
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string());
    let sunset = bundle
        .daily
        .first()
        .and_then(|d| d.sunset)
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string());

    let temp_track = bundle
        .hourly
        .iter()
        .take(24)
        .filter_map(|h| h.temperature_2m_c)
        .collect::<Vec<_>>();
    let precip_track = bundle
        .hourly
        .iter()
        .take(24)
        .map(|h| h.precipitation_mm.unwrap_or(0.0))
        .collect::<Vec<_>>();
    let gust_track = bundle
        .hourly
        .iter()
        .take(24)
        .map(|h| h.wind_gusts_10m.unwrap_or(0.0))
        .collect::<Vec<_>>();
    let trend_width = w.saturating_sub(left_col_width + 12).clamp(8, 28);

    let left_lines = [
        "Live instruments".to_string(),
        format!("Temp   {} {:>4}C", meter(temp_norm, meter_w), temp_c),
        format!(
            "Hum    {} {:>4.0}%",
            meter(humidity / 100.0, meter_w),
            humidity
        ),
        format!(
            "Press  {} {:>4.0}hPa",
            meter(pressure_norm, meter_w),
            pressure
        ),
        format!("UV     {} {:>4.1}", meter(uv_norm, meter_w), uv),
        format!("Vis    {} {:>4.1}km", meter(vis_norm, meter_w), vis_km),
        format!(
            "Wind   {:>2} {:>4.0} km/h  gust {:>3.0}",
            compass_arrow(current.wind_direction_10m),
            wind,
            gust
        ),
    ];
    let mut lines = if w >= 74 && h >= 9 {
        let right_lines = [
            "Overview".to_string(),
            format!("Condition {}", scene_name(category, bundle.current.is_day)),
            format!("Cloud {:>3.0}%   Rain now {:>3.1}mm", cloud, precip_now),
            format!("Sun arc {sunrise} -> {sunset}"),
            format!("T24 {}", sparkline_blocks(&temp_track, trend_width)),
            format!("R24 {}", sparkline_blocks(&precip_track, trend_width)),
            format!("G24 {}", sparkline_blocks(&gust_track, trend_width)),
            format!("Visibility {:>4.1}km", vis_km),
            "Compass rose".to_string(),
            "    N".to_string(),
            format!(
                "  W {} E   dir {}",
                if compass_arrow(current.wind_direction_10m) == '←' {
                    '◉'
                } else {
                    '+'
                },
                compass_short(current.wind_direction_10m)
            ),
            "    S".to_string(),
        ];

        let mut merged = Vec::with_capacity(left_lines.len().max(right_lines.len()));
        for idx in 0..left_lines.len().max(right_lines.len()) {
            let left = left_lines.get(idx).map(String::as_str).unwrap_or("");
            let right = right_lines.get(idx).map(String::as_str).unwrap_or("");
            merged.push(format!(
                "{left:<left_col_width$}  {right}",
                left_col_width = left_col_width
            ));
        }
        merged
    } else {
        left_lines.to_vec()
    };

    if h >= 12 && w < 74 {
        lines.push("".to_string());
        lines.push("Compass".to_string());
        lines.push("    N".to_string());
        lines.push(format!(
            "  W {} E   dir {}",
            if compass_arrow(current.wind_direction_10m) == '←' {
                '◉'
            } else {
                '+'
            },
            compass_short(current.wind_direction_10m)
        ));
        lines.push("    S".to_string());
    }

    LandmarkScene {
        label: "Gauge Cluster · Live Instruments".to_string(),
        lines: if h >= 12 {
            fit_lines_centered(lines, w, h)
        } else {
            fit_lines(lines, w, h)
        },
        tint: tint_for_category(category),
    }
}

#[allow(clippy::needless_range_loop)]
pub fn scene_for_sky_observatory(
    bundle: &ForecastBundle,
    frame_tick: u64,
    animate: bool,
    width: u16,
    height: u16,
) -> LandmarkScene {
    let w = width as usize;
    let h = height as usize;
    let category = weather_code_to_category(bundle.current.weather_code);
    if w < 24 || h < 8 {
        return compact_condition_scene(category, bundle.current.is_day, width, height);
    }

    let mut canvas = vec![vec![' '; w]; h];
    let arc_top = 1usize;
    let arc_bottom = (h.saturating_mul(55) / 100).clamp(4, h.saturating_sub(4));
    let mid = (w.saturating_sub(1)) as f32 / 2.0;
    let radius = (w as f32 * 0.46).max(1.0);

    for x in 0..w {
        let dx = (x as f32 - mid) / radius;
        let y = (arc_bottom as f32 - (1.0 - dx * dx).max(0.0) * (arc_bottom - arc_top) as f32)
            .round()
            .clamp(arc_top as f32, arc_bottom as f32) as usize;
        canvas[y][x] = '·';
    }

    let (sunrise_h, sunset_h) = bundle
        .daily
        .first()
        .map(|day| {
            (
                day.sunrise
                    .map(|t| t.hour() as f32 + t.minute() as f32 / 60.0)
                    .unwrap_or(6.0),
                day.sunset
                    .map(|t| t.hour() as f32 + t.minute() as f32 / 60.0)
                    .unwrap_or(18.0),
            )
        })
        .unwrap_or((6.0, 18.0));

    let now_h = bundle
        .hourly
        .first()
        .map(|h| h.time.hour() as f32 + h.time.minute() as f32 / 60.0)
        .unwrap_or(12.0);
    let day_span = (sunset_h - sunrise_h).max(0.1);
    let progress = ((now_h - sunrise_h) / day_span).clamp(0.0, 1.0);
    let marker_x = (progress * (w.saturating_sub(1)) as f32).round() as usize;

    let marker_y = locate_arc_y(marker_x, w, arc_top, arc_bottom);
    draw_celestial_icon(
        &mut canvas,
        marker_x,
        marker_y,
        bundle.current.is_day,
        moon_phase(bundle),
        w,
        h,
    );
    if marker_x > 0 {
        let y = locate_arc_y(0, w, arc_top, arc_bottom);
        canvas[y][0] = '⌂';
    }
    if w > 1 {
        let y = locate_arc_y(w - 1, w, arc_top, arc_bottom);
        canvas[y][w - 1] = '⌂';
    }

    if !bundle.current.is_day {
        let star_count = (w / 5).max(6);
        let phase = if animate { frame_tick as usize } else { 0 };
        for i in 0..star_count {
            let x = ((i * 7 + phase) % w).min(w - 1);
            let y = 1 + ((i * 5 + phase) % arc_bottom.max(2));
            if canvas[y][x] == ' ' {
                canvas[y][x] = if i.is_multiple_of(2) { '*' } else { '·' };
            }
        }
    }

    let strip_y = h.saturating_sub(3);
    let precip_y = h.saturating_sub(2);
    let summary_y = h.saturating_sub(1);
    let horizon = '─';
    for cell in canvas[strip_y.saturating_sub(1)].iter_mut().take(w) {
        if *cell == ' ' {
            *cell = horizon;
        }
    }

    let slice = bundle.hourly.iter().take(w.min(24)).collect::<Vec<_>>();
    for (i, hour) in slice.iter().enumerate() {
        let x = ((i as f32 / slice.len().max(1) as f32) * (w.saturating_sub(1)) as f32).round()
            as usize;
        let code = hour.weather_code.unwrap_or(bundle.current.weather_code);
        canvas[strip_y][x] = symbol_for_code(code);
        if let Some(mm) = hour.precipitation_mm {
            canvas[precip_y][x] = if mm >= 2.5 {
                '█'
            } else if mm >= 1.0 {
                '▓'
            } else if mm >= 0.2 {
                '▒'
            } else if mm > 0.0 {
                '░'
            } else {
                '·'
            };
        } else {
            canvas[precip_y][x] = '·';
        }
    }

    let sunrise_txt = format_time_hm(sunrise_h);
    let sunset_txt = format_time_hm(sunset_h);
    let summary = format!(
        "sun {} -> {}  now {}",
        sunrise_txt,
        sunset_txt,
        format_time_hm(now_h)
    );
    for (idx, ch) in summary.chars().enumerate().take(w) {
        canvas[summary_y][idx] = ch;
    }

    LandmarkScene {
        label: "Sky Observatory · Sun/Moon Arc".to_string(),
        lines: canvas_to_lines(canvas, w),
        tint: tint_for_category(category),
    }
}

fn compact_condition_scene(
    category: WeatherCategory,
    is_day: bool,
    width: u16,
    height: u16,
) -> LandmarkScene {
    let mut lines = match (category, is_day) {
        (WeatherCategory::Clear, true) => vec![
            "   \\  |  /   ".to_string(),
            " --   O   -- ".to_string(),
            "   /  |  \\   ".to_string(),
        ],
        (WeatherCategory::Clear, false) => vec![
            "    _..._    ".to_string(),
            "  .:::::::.  ".to_string(),
            "   ':::::'   ".to_string(),
        ],
        (WeatherCategory::Cloudy, _) => vec![
            "    .--.     ".to_string(),
            " .-(____)-.  ".to_string(),
            "    (__)     ".to_string(),
        ],
        (WeatherCategory::Rain, _) => vec![
            "    .--.     ".to_string(),
            " .-(____)-.  ".to_string(),
            "   / / / /   ".to_string(),
        ],
        (WeatherCategory::Snow, _) => vec![
            "    .--.     ".to_string(),
            " .-(____)-.  ".to_string(),
            "   *  *  *   ".to_string(),
        ],
        (WeatherCategory::Thunder, _) => vec![
            "    .--.     ".to_string(),
            " .-(____)-.  ".to_string(),
            "    /\\/\\/    ".to_string(),
        ],
        (WeatherCategory::Fog, _) => vec![
            "  ~~~~~~~~~~ ".to_string(),
            " ~ ~~~~~~~~ ~".to_string(),
            "  ~~~~~~~~~~ ".to_string(),
        ],
        _ => vec![
            "    .--.     ".to_string(),
            "   ( ?? )    ".to_string(),
            "    '--'     ".to_string(),
        ],
    };

    if usize::from(height) > lines.len() && width >= 10 {
        lines.push(format!(
            "{:^w$}",
            compact_scene_label(category, is_day),
            w = usize::from(width)
        ));
    }

    LandmarkScene {
        label: format!("Atmos Canvas · {}", scene_name(category, is_day)),
        lines: fit_lines(lines, width as usize, height as usize),
        tint: tint_for_category(category),
    }
}

fn compact_scene_label(category: WeatherCategory, is_day: bool) -> &'static str {
    match (category, is_day) {
        (WeatherCategory::Clear, true) => "CLEAR",
        (WeatherCategory::Clear, false) => "CLEAR NIGHT",
        (WeatherCategory::Cloudy, _) => "CLOUDY",
        (WeatherCategory::Rain, _) => "RAIN",
        (WeatherCategory::Snow, _) => "SNOW",
        (WeatherCategory::Fog, _) => "FOG",
        (WeatherCategory::Thunder, _) => "THUNDER",
        (WeatherCategory::Unknown, _) => "WEATHER",
    }
}

fn canvas_to_lines(canvas: Vec<Vec<char>>, width: usize) -> Vec<String> {
    canvas
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .map(|line| fit_line(&line, width))
        .collect()
}

fn fit_lines(mut lines: Vec<String>, width: usize, height: usize) -> Vec<String> {
    lines = lines
        .into_iter()
        .map(|line| fit_line(&line, width))
        .take(height)
        .collect::<Vec<_>>();
    while lines.len() < height {
        lines.push(" ".repeat(width));
    }
    lines
}

fn fit_line(line: &str, width: usize) -> String {
    let mut out = line.chars().take(width).collect::<String>();
    let len = out.chars().count();
    if len < width {
        out.push_str(&" ".repeat(width - len));
    }
    out
}

fn locate_arc_y(x: usize, width: usize, top: usize, bottom: usize) -> usize {
    let mid = (width.saturating_sub(1)) as f32 / 2.0;
    let radius = (width as f32 * 0.46).max(1.0);
    let dx = (x as f32 - mid) / radius;
    (bottom as f32 - (1.0 - dx * dx).max(0.0) * (bottom - top) as f32)
        .round()
        .clamp(top as f32, bottom as f32) as usize
}

fn draw_celestial_icon(
    canvas: &mut [Vec<char>],
    x: usize,
    y: usize,
    is_day: bool,
    moon_symbol: char,
    width: usize,
    height: usize,
) {
    let large = width >= 44 && height >= 11;
    let huge = width >= 70 && height >= 14;
    let center = if is_day {
        if large { '☀' } else { '◉' }
    } else {
        moon_symbol
    };
    paint_char(canvas, x as isize, y as isize, center, true);

    if is_day && large {
        for (dx, dy) in [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ] {
            paint_char(canvas, x as isize + dx, y as isize + dy, '✶', false);
        }
        if huge {
            for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
                paint_char(canvas, x as isize + dx, y as isize + dy, '✶', false);
            }
        }
    } else if !is_day && large {
        for (dx, dy) in [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ] {
            paint_char(canvas, x as isize + dx, y as isize + dy, '·', false);
        }
        if huge {
            for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
                paint_char(canvas, x as isize + dx, y as isize + dy, '·', false);
            }
        }
    }
}

fn paint_char(canvas: &mut [Vec<char>], x: isize, y: isize, ch: char, force: bool) {
    if canvas.is_empty() || canvas[0].is_empty() {
        return;
    }
    if x < 0 || y < 0 {
        return;
    }
    let ux = x as usize;
    let uy = y as usize;
    if uy >= canvas.len() || ux >= canvas[0].len() {
        return;
    }

    let current = canvas[uy][ux];
    if force || matches!(current, ' ' | '·') {
        canvas[uy][ux] = ch;
    }
}

fn symbol_for_code(code: u8) -> char {
    match weather_code_to_category(code) {
        WeatherCategory::Clear => 'o',
        WeatherCategory::Cloudy => '~',
        WeatherCategory::Rain => '/',
        WeatherCategory::Snow => '*',
        WeatherCategory::Fog => '=',
        WeatherCategory::Thunder => '!',
        WeatherCategory::Unknown => '?',
    }
}

fn meter(norm: f32, width: usize) -> String {
    let width = width.max(4);
    let fill = (norm.clamp(0.0, 1.0) * width as f32).round() as usize;
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for idx in 0..width {
        let ch = if idx < fill {
            '█'
        } else if idx == fill {
            '▓'
        } else if idx == fill.saturating_add(1) {
            '▒'
        } else {
            '·'
        };
        bar.push(ch);
    }
    bar.push(']');
    bar
}

fn sparkline_blocks(values: &[f32], width: usize) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max - min).max(0.001);
    (0..width)
        .map(|idx| {
            let src = (idx * values.len() / width).min(values.len().saturating_sub(1));
            let norm = ((values[src] - min) / span).clamp(0.0, 1.0);
            BARS[(norm * (BARS.len() - 1) as f32).round() as usize]
        })
        .collect()
}

fn fit_lines_centered(lines: Vec<String>, width: usize, height: usize) -> Vec<String> {
    let trimmed = lines
        .into_iter()
        .map(|line| fit_line(&line, width))
        .take(height)
        .collect::<Vec<_>>();
    if trimmed.len() >= height {
        return trimmed;
    }

    let pad = (height - trimmed.len()) / 2;
    let mut out = Vec::with_capacity(height);
    for _ in 0..pad {
        out.push(" ".repeat(width));
    }
    out.extend(trimmed);
    while out.len() < height {
        out.push(" ".repeat(width));
    }
    out
}

fn compass_arrow(deg: f32) -> char {
    let d = deg.rem_euclid(360.0);
    match d as i32 {
        23..=67 => '↗',
        68..=112 => '→',
        113..=157 => '↘',
        158..=202 => '↓',
        203..=247 => '↙',
        248..=292 => '←',
        293..=337 => '↖',
        _ => '↑',
    }
}

fn compass_short(deg: f32) -> &'static str {
    let d = deg.rem_euclid(360.0);
    match d as i32 {
        23..=67 => "NE",
        68..=112 => "E",
        113..=157 => "SE",
        158..=202 => "S",
        203..=247 => "SW",
        248..=292 => "W",
        293..=337 => "NW",
        _ => "N",
    }
}

fn moon_phase(bundle: &ForecastBundle) -> char {
    let day = bundle.daily.first().map(|d| d.date.ordinal()).unwrap_or(1) as f32;
    let phase = (day % 29.53) / 29.53;
    match phase {
        p if p < 0.06 => '●',
        p if p < 0.19 => '◔',
        p if p < 0.31 => '◑',
        p if p < 0.44 => '◕',
        p if p < 0.56 => '○',
        p if p < 0.69 => '◖',
        p if p < 0.81 => '◐',
        p if p < 0.94 => '◗',
        _ => '●',
    }
}

fn format_time_hm(hour_f: f32) -> String {
    let total = (hour_f * 60.0).round().max(0.0) as i32;
    let h = (total / 60).rem_euclid(24);
    let m = total % 60;
    format!("{h:02}:{m:02}")
}

fn tint_for_category(category: WeatherCategory) -> LandmarkTint {
    match category {
        WeatherCategory::Clear => LandmarkTint::Warm,
        WeatherCategory::Cloudy | WeatherCategory::Fog => LandmarkTint::Neutral,
        WeatherCategory::Rain | WeatherCategory::Snow | WeatherCategory::Thunder => {
            LandmarkTint::Cool
        }
        WeatherCategory::Unknown => LandmarkTint::Neutral,
    }
}

fn scene_name(category: WeatherCategory, is_day: bool) -> &'static str {
    match (category, is_day) {
        (WeatherCategory::Clear, true) => "Clear sky",
        (WeatherCategory::Clear, false) => "Clear night",
        (WeatherCategory::Cloudy, _) => "Cloudy",
        (WeatherCategory::Rain, _) => "Rain",
        (WeatherCategory::Snow, _) => "Snow",
        (WeatherCategory::Fog, _) => "Fog",
        (WeatherCategory::Thunder, _) => "Thunderstorm",
        (WeatherCategory::Unknown, _) => "Unknown",
    }
}
