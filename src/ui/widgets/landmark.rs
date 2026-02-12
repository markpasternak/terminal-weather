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

    let mut canvas = vec![vec![' '; w]; h];
    let horizon_y = (h.saturating_mul(62) / 100).clamp(3, h.saturating_sub(2));
    let terrain_amp = horizon_y.saturating_sub(2).clamp(2, h / 2);
    let min_temp = temps.iter().copied().fold(f32::INFINITY, f32::min);
    let max_temp = temps.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max_temp - min_temp).max(0.2);

    let mut terrain_top = vec![horizon_y; w];
    for (x, top) in terrain_top.iter_mut().enumerate().take(w) {
        let t = if w <= 1 {
            0.0
        } else {
            x as f32 / (w - 1) as f32
        };
        let pos = t * (temps.len().saturating_sub(1)) as f32;
        let left = pos.floor() as usize;
        let right = pos.ceil() as usize;
        let frac = pos - left as f32;
        let sample = if right > left {
            temps[left] * (1.0 - frac) + temps[right] * frac
        } else {
            temps[left]
        };
        let norm = ((sample - min_temp) / span).clamp(0.0, 1.0);
        let peak = (norm * terrain_amp as f32).round() as usize;
        *top = horizon_y.saturating_sub(peak).max(1);
    }

    for cell in canvas[horizon_y].iter_mut().take(w) {
        *cell = '─';
    }
    for (x, top) in terrain_top.iter().copied().enumerate() {
        for (y, row) in canvas.iter_mut().enumerate().take(h) {
            if y < top {
                continue;
            }
            row[x] = if y == top {
                '▀'
            } else if y == top + 1 {
                '▓'
            } else {
                '█'
            };
        }
    }

    let phase = if animate {
        (frame_tick % 128) as usize
    } else {
        0
    };
    let hour = bundle
        .hourly
        .first()
        .map(|hour| hour.time.hour() as usize)
        .unwrap_or(12)
        % 24;
    place_day_night_marker(&mut canvas, bundle.current.is_day, hour, horizon_y);
    paint_sky_detail(
        &mut canvas,
        category,
        bundle.current.is_day,
        phase,
        horizon_y,
    );

    LandmarkScene {
        label: format!(
            "Atmos Canvas · {}",
            scene_name(category, bundle.current.is_day)
        ),
        lines: canvas_to_lines(canvas, w),
        tint: tint_for_category(category),
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
    let lines = match (category, is_day) {
        (WeatherCategory::Clear, true) => vec![
            "   \\  |  / ".to_string(),
            " ---  O  ---".to_string(),
            "   /  |  \\ ".to_string(),
        ],
        (WeatherCategory::Clear, false) => vec![
            "    _..._   ".to_string(),
            "  .:::::::. ".to_string(),
            "   ':::::'  ".to_string(),
        ],
        (WeatherCategory::Rain, _) => vec![
            "  .-~~~~-.  ".to_string(),
            " (  ____ )  ".to_string(),
            "  / / / /   ".to_string(),
        ],
        (WeatherCategory::Snow, _) => vec![
            "  .-~~~~-.  ".to_string(),
            " (  ____ )  ".to_string(),
            "  *  *  *   ".to_string(),
        ],
        (WeatherCategory::Thunder, _) => vec![
            "  .-~~~~-.  ".to_string(),
            " (  ____ )  ".to_string(),
            "   /\\/\\/    ".to_string(),
        ],
        (WeatherCategory::Fog, _) => vec![
            "~~~~~~~~~~~~".to_string(),
            " ~~~~~~~~~~ ".to_string(),
            "~~~~~~~~~~~~".to_string(),
        ],
        _ => vec![
            "  .-~~-.    ".to_string(),
            " (  ..  )   ".to_string(),
            "  `-__-'    ".to_string(),
        ],
    };

    LandmarkScene {
        label: format!("Atmos Canvas · {}", scene_name(category, is_day)),
        lines: fit_lines(lines, width as usize, height as usize),
        tint: tint_for_category(category),
    }
}

fn paint_sky_detail(
    canvas: &mut [Vec<char>],
    category: WeatherCategory,
    is_day: bool,
    frame_phase: usize,
    horizon_y: usize,
) {
    if canvas.is_empty() || canvas[0].is_empty() {
        return;
    }
    let h = canvas.len();
    let w = canvas[0].len();
    let sky_top = horizon_y.saturating_sub(1);
    let seed = (w as u64)
        .wrapping_mul(1_000_003)
        .wrapping_add(h as u64)
        .wrapping_add(frame_phase as u64);

    if !is_day {
        let stars = (w / 6).max(6);
        for i in 0..stars {
            let x = ((seed as usize + i * 11) % w).min(w - 1);
            let y = (1 + ((seed as usize / 7 + i * 5) % sky_top.max(2))).min(sky_top);
            if canvas[y][x] == ' ' {
                canvas[y][x] = if i.is_multiple_of(2) { '*' } else { '·' };
            }
        }
    }

    let cloud_cover = match category {
        WeatherCategory::Cloudy => 4,
        WeatherCategory::Fog => 5,
        WeatherCategory::Rain => 5,
        WeatherCategory::Snow => 5,
        WeatherCategory::Thunder => 6,
        _ => 2,
    };
    for i in 0..cloud_cover {
        let cx = ((seed as usize + i * 13 + frame_phase) % w).min(w.saturating_sub(1));
        let cy = 1 + (i * 2 % sky_top.max(2));
        draw_cloud(canvas, cx, cy.min(sky_top));
    }

    match category {
        WeatherCategory::Rain => draw_rain(canvas, frame_phase, horizon_y),
        WeatherCategory::Snow => draw_snow(canvas, frame_phase, horizon_y),
        WeatherCategory::Fog => draw_fog(canvas, frame_phase, horizon_y),
        WeatherCategory::Thunder => {
            draw_rain(canvas, frame_phase, horizon_y);
            draw_lightning(canvas, frame_phase, horizon_y);
        }
        _ => {}
    }
}

fn draw_cloud(canvas: &mut [Vec<char>], x: usize, y: usize) {
    let w = canvas[0].len();
    let cloud = ['(', '~', '~', ')'];
    for (idx, ch) in cloud.into_iter().enumerate() {
        let px = x.saturating_add(idx).min(w.saturating_sub(1));
        if canvas[y][px] == ' ' {
            canvas[y][px] = ch;
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn draw_rain(canvas: &mut [Vec<char>], frame_phase: usize, horizon_y: usize) {
    let h = canvas.len();
    let w = canvas[0].len();
    for x in 0..w {
        if (x + frame_phase).is_multiple_of(3) {
            let y = 1 + ((x + frame_phase) % horizon_y.max(2));
            if y < h && canvas[y][x] == ' ' {
                canvas[y][x] = '/';
            }
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn draw_snow(canvas: &mut [Vec<char>], frame_phase: usize, horizon_y: usize) {
    let h = canvas.len();
    let w = canvas[0].len();
    for x in 0..w {
        if (x + frame_phase).is_multiple_of(4) {
            let y = 1 + ((x * 2 + frame_phase) % horizon_y.max(2));
            if y < h && canvas[y][x] == ' ' {
                canvas[y][x] = if (x + y + frame_phase).is_multiple_of(2) {
                    '*'
                } else {
                    '·'
                };
            }
        }
    }
}

fn draw_fog(canvas: &mut [Vec<char>], frame_phase: usize, horizon_y: usize) {
    let h = canvas.len();
    let w = canvas[0].len();
    let band_start = horizon_y.saturating_sub(2);
    let band_end = (horizon_y + 2).min(h.saturating_sub(1));
    for (y, row) in canvas
        .iter_mut()
        .enumerate()
        .take(band_end + 1)
        .skip(band_start)
    {
        for (x, cell) in row.iter_mut().enumerate().take(w) {
            if (x + y + frame_phase).is_multiple_of(2) && *cell == ' ' {
                *cell = '░';
            }
        }
    }
}

fn draw_lightning(canvas: &mut [Vec<char>], frame_phase: usize, horizon_y: usize) {
    let h = canvas.len();
    let w = canvas[0].len();
    let x = ((w / 2) + frame_phase % (w / 3).max(1)).min(w.saturating_sub(2));
    let points = [
        (x, 1usize),
        (x.saturating_sub(1), (horizon_y / 3).max(2)),
        (x + 1, (horizon_y / 2).max(3)),
        (x, horizon_y.saturating_sub(1)),
    ];
    for (px, py) in points {
        if py < h && px < w {
            canvas[py][px] = '⚡';
        }
    }
}

fn place_day_night_marker(canvas: &mut [Vec<char>], is_day: bool, hour: usize, horizon_y: usize) {
    let h = canvas.len();
    let w = canvas[0].len();
    if w == 0 || h == 0 {
        return;
    }

    let x = ((hour as f32 / 23.0) * (w.saturating_sub(1) as f32)).round() as usize;
    let curve = ((hour as f32 / 23.0) - 0.5).abs() * 2.0;
    let y = (1.0 + curve * (horizon_y as f32 * 0.35)).round() as usize;
    let y = y.min(horizon_y.saturating_sub(1)).min(h.saturating_sub(1));
    canvas[y][x] = if is_day { '◉' } else { '◐' };
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
