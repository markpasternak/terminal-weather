use chrono::{Datelike, Timelike};

use crate::domain::weather::{ForecastBundle, WeatherCategory, weather_code_to_category};
use crate::ui::widgets::landmark::compact::compact_condition_scene;
use crate::ui::widgets::landmark::shared::{canvas_to_lines, paint_char};
use crate::ui::widgets::landmark::{LandmarkScene, tint_for_category};

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

    let (sunrise_h, sunset_h) = bundle.daily.first().map_or((6.0, 18.0), |day| {
        (
            day.sunrise.map_or(6.0, hm_to_hour_f32),
            day.sunset.map_or(18.0, hm_to_hour_f32),
        )
    });

    let now_h = bundle
        .hourly
        .first()
        .map_or(12.0, |hour| hm_to_hour_f32(hour.time));
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
        canvas[y][0] = 'E';
    }
    if w > 1 {
        let y = locate_arc_y(w - 1, w, arc_top, arc_bottom);
        canvas[y][w - 1] = 'W';
    }

    if !bundle.current.is_day {
        let star_count = (w / 5).max(6);
        let phase = if animate { frame_tick as usize } else { 0 };
        for i in 0..star_count {
            let x = ((i * 7 + phase) % w).min(w - 1);
            let y = 1 + ((i * 5 + phase) % arc_bottom.max(2));
            if canvas[y][x] == ' ' {
                canvas[y][x] = if i % 2 == 0 { '*' } else { '·' };
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

fn hm_to_hour_f32<T: Timelike>(value: T) -> f32 {
    value.hour() as f32 + value.minute() as f32 / 60.0
}
