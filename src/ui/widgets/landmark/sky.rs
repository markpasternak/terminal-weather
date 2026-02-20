#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use chrono::{Datelike, Timelike};

use crate::domain::weather::{ForecastBundle, WeatherCategory, weather_code_to_category};
use crate::ui::widgets::landmark::compact::compact_condition_scene;
use crate::ui::widgets::landmark::shared::{canvas_to_lines, paint_char};
use crate::ui::widgets::landmark::{LandmarkScene, tint_for_category};

const MOON_PHASE_THRESHOLDS: &[(f32, char)] = &[
    (0.06, '●'),
    (0.19, '◔'),
    (0.31, '◑'),
    (0.44, '◕'),
    (0.56, '○'),
    (0.69, '◖'),
    (0.81, '◐'),
    (0.94, '◗'),
    (1.0, '●'),
];

#[allow(clippy::needless_range_loop)]
#[must_use]
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

    let canvas = build_sky_observatory_canvas(bundle, frame_tick, animate, w, h);
    let (sunrise_h, sunset_h) = sun_window(bundle);
    let now_h = current_hour(bundle);
    LandmarkScene {
        label: "Sky Observatory · Sun/Moon Arc".to_string(),
        lines: canvas_to_lines(canvas, w),
        tint: tint_for_category(category),
        context_line: Some(sky_context_line(
            sunrise_h,
            sunset_h,
            now_h,
            bundle.current.is_day,
        )),
    }
}

fn sky_context_line(sunrise_h: f32, sunset_h: f32, now_h: f32, is_day: bool) -> String {
    if is_day {
        let remaining_mins = ((sunset_h - now_h) * 60.0).max(0.0).round() as i32;
        let hours = remaining_mins / 60;
        let mins = remaining_mins % 60;
        if remaining_mins <= 0 {
            "Sunset passing · twilight".to_string()
        } else {
            format!("{hours}h {mins:02}m of daylight remaining")
        }
    } else {
        let remaining_mins = if now_h > sunset_h {
            // After sunset, time until next sunrise (assume ~24h cycle)
            ((24.0 - now_h + sunrise_h) * 60.0).round() as i32
        } else {
            // Before sunrise
            ((sunrise_h - now_h) * 60.0).round() as i32
        };
        let hours = remaining_mins.max(0) / 60;
        let mins = remaining_mins.max(0) % 60;
        format!("{hours}h {mins:02}m until sunrise")
    }
}

#[allow(clippy::needless_range_loop)]
fn build_sky_observatory_canvas(
    bundle: &ForecastBundle,
    frame_tick: u64,
    animate: bool,
    width: usize,
    height: usize,
) -> Vec<Vec<char>> {
    let mut canvas = vec![vec![' '; width]; height];
    let (arc_top, arc_bottom) = arc_bounds(height);
    draw_arc(&mut canvas, width, arc_top, arc_bottom);

    let (sunrise_h, sunset_h) = sun_window(bundle);
    let now_h = current_hour(bundle);
    let day_span = (sunset_h - sunrise_h).max(0.1);
    let progress = ((now_h - sunrise_h) / day_span).clamp(0.0, 1.0);
    let marker_x = (progress * (width.saturating_sub(1)) as f32).round() as usize;
    let marker_y = locate_arc_y(marker_x, width, arc_top, arc_bottom);
    draw_celestial_icon(
        &mut canvas,
        marker_x,
        marker_y,
        bundle.current.is_day,
        moon_phase(bundle),
        width,
        height,
    );
    paint_cardinal_markers(&mut canvas, width, arc_top, arc_bottom, marker_x);
    paint_sun_event_markers(&mut canvas, width, arc_top, arc_bottom, sunrise_h, sunset_h);
    paint_night_stars(
        &mut canvas,
        width,
        arc_bottom,
        bundle.current.is_day,
        animate,
        frame_tick,
    );

    let strip_y = height.saturating_sub(3);
    let precip_y = height.saturating_sub(2);
    let summary_y = height.saturating_sub(1);
    paint_horizon_strip(&mut canvas, strip_y, width);
    plot_hourly_strip(bundle, &mut canvas, strip_y, precip_y, width);
    write_summary_line(&mut canvas, summary_y, width, sunrise_h, sunset_h, now_h);

    canvas
}

fn arc_bounds(height: usize) -> (usize, usize) {
    (
        1usize,
        (height.saturating_mul(55) / 100).clamp(4, height.saturating_sub(4)),
    )
}

#[allow(clippy::needless_range_loop)]
fn draw_arc(canvas: &mut [Vec<char>], width: usize, top: usize, bottom: usize) {
    let mid_x = width / 2;
    for x in 0..width {
        let y = locate_arc_y(x, width, top, bottom);
        let ch = arc_glyph(x, y, width, top, mid_x);
        canvas[y][x] = ch;
    }
}

fn arc_glyph(x: usize, y: usize, width: usize, top: usize, mid_x: usize) -> char {
    if y == top || (y == top + 1 && (mid_x.wrapping_sub(x)) <= 2) {
        return '─';
    }
    if x <= width / 6 || x >= width * 5 / 6 {
        return '·';
    }
    if x <= width / 3 {
        return '╭';
    }
    if x >= width * 2 / 3 {
        return '╮';
    }
    '·'
}

fn paint_sun_event_markers(
    canvas: &mut [Vec<char>],
    width: usize,
    arc_top: usize,
    arc_bottom: usize,
    _sunrise_h: f32,
    _sunset_h: f32,
) {
    // Plot sunrise ↑ and sunset ↓ on the arc at their X positions
    // Sunrise is at progress 0.0, sunset at progress 1.0 of the day span
    let sunrise_x = 0usize;
    let sunset_x = width.saturating_sub(1);
    let sunrise_y = locate_arc_y(sunrise_x, width, arc_top, arc_bottom);
    let sunset_y = locate_arc_y(sunset_x, width, arc_top, arc_bottom);

    // Place markers just above the arc position
    if sunrise_y > 0 {
        paint_char(
            canvas,
            sunrise_x as isize,
            (sunrise_y - 1) as isize,
            '↑',
            true,
        );
    }
    if sunset_y > 0 {
        paint_char(
            canvas,
            sunset_x as isize,
            (sunset_y - 1) as isize,
            '↓',
            true,
        );
    }
}

fn sun_window(bundle: &ForecastBundle) -> (f32, f32) {
    bundle.daily.first().map_or((6.0, 18.0), |day| {
        (
            day.sunrise.map_or(6.0, |t| hm_to_hour_f32(&t)),
            day.sunset.map_or(18.0, |t| hm_to_hour_f32(&t)),
        )
    })
}

fn current_hour(bundle: &ForecastBundle) -> f32 {
    bundle
        .hourly
        .first()
        .map_or(12.0, |hour| hm_to_hour_f32(&hour.time))
}

fn paint_cardinal_markers(
    canvas: &mut [Vec<char>],
    width: usize,
    top: usize,
    bottom: usize,
    marker_x: usize,
) {
    if marker_x > 0 {
        let y = locate_arc_y(0, width, top, bottom);
        canvas[y][0] = 'E';
    }
    if width > 1 {
        let y = locate_arc_y(width - 1, width, top, bottom);
        canvas[y][width - 1] = 'W';
    }
}

fn paint_night_stars(
    canvas: &mut [Vec<char>],
    width: usize,
    arc_bottom: usize,
    is_day: bool,
    animate: bool,
    frame_tick: u64,
) {
    if is_day {
        return;
    }
    let star_count = (width / 5).max(6);
    let phase = if animate { frame_tick as usize } else { 0 };
    for i in 0..star_count {
        let x = ((i * 7 + phase) % width).min(width - 1);
        let y = 1 + ((i * 5 + phase) % arc_bottom.max(2));
        if canvas[y][x] == ' ' {
            canvas[y][x] = if i % 2 == 0 { '*' } else { '·' };
        }
    }
}

fn paint_horizon_strip(canvas: &mut [Vec<char>], strip_y: usize, width: usize) {
    for cell in canvas[strip_y.saturating_sub(1)].iter_mut().take(width) {
        if *cell == ' ' {
            *cell = '─';
        }
    }
}

fn plot_hourly_strip(
    bundle: &ForecastBundle,
    canvas: &mut [Vec<char>],
    strip_y: usize,
    precip_y: usize,
    width: usize,
) {
    let slice = bundle.hourly.iter().take(width.min(24)).collect::<Vec<_>>();
    for (i, hour) in slice.iter().enumerate() {
        let x = ((i as f32 / slice.len().max(1) as f32) * (width.saturating_sub(1)) as f32).round()
            as usize;
        let code = hour.weather_code.unwrap_or(bundle.current.weather_code);
        canvas[strip_y][x] = symbol_for_code(code);
        canvas[precip_y][x] = precip_symbol(hour.precipitation_mm);
    }
}

fn precip_symbol(mm: Option<f32>) -> char {
    let Some(mm) = mm else {
        return '·';
    };
    if mm >= 2.5 {
        '█'
    } else if mm >= 1.0 {
        '▓'
    } else if mm >= 0.2 {
        '▒'
    } else if mm > 0.0 {
        '░'
    } else {
        '·'
    }
}

fn write_summary_line(
    canvas: &mut [Vec<char>],
    summary_y: usize,
    width: usize,
    sunrise_h: f32,
    sunset_h: f32,
    now_h: f32,
) {
    let summary = format!(
        "sun {} -> {}  now {}",
        format_time_hm(sunrise_h),
        format_time_hm(sunset_h),
        format_time_hm(now_h)
    );
    for (idx, ch) in summary.chars().enumerate().take(width) {
        canvas[summary_y][idx] = ch;
    }
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
    let center = center_symbol(is_day, large, moon_symbol);
    paint_char(canvas, x as isize, y as isize, center, true);

    if large {
        let halo = if is_day { '✶' } else { '·' };
        paint_halo(canvas, x, y, halo, huge);
    }
}

fn center_symbol(is_day: bool, large: bool, moon_symbol: char) -> char {
    if is_day {
        if large { '☀' } else { '◉' }
    } else {
        moon_symbol
    }
}

fn paint_halo(canvas: &mut [Vec<char>], x: usize, y: usize, halo: char, huge: bool) {
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
        paint_char(canvas, x as isize + dx, y as isize + dy, halo, false);
    }
    if huge {
        for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
            paint_char(canvas, x as isize + dx, y as isize + dy, halo, false);
        }
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

fn moon_phase(bundle: &ForecastBundle) -> char {
    let day = bundle.daily.first().map_or(1, |d| d.date.ordinal()) as f32;
    let phase = (day % 29.53) / 29.53;
    for (threshold, symbol) in MOON_PHASE_THRESHOLDS {
        if phase < *threshold {
            return *symbol;
        }
    }
    '●'
}

fn format_time_hm(hour_f: f32) -> String {
    let total = (hour_f * 60.0).round().max(0.0) as i32;
    let h = (total / 60).rem_euclid(24);
    let m = total % 60;
    format!("{h:02}:{m:02}")
}

fn hm_to_hour_f32<T: Timelike>(value: &T) -> f32 {
    value.hour() as f32 + value.minute() as f32 / 60.0
}
