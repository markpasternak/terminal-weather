use chrono::Timelike;

use crate::domain::weather::ForecastBundle;
use crate::ui::animation::{SeededMotion, UiMotionContext};
use crate::ui::widgets::landmark::shared::paint_char;

use super::effects::draw_ambient_cloud;

pub(super) fn is_night(bundle: &ForecastBundle) -> bool {
    !bundle.current.is_day
}

pub(super) fn paint_starfield(
    canvas: &mut [Vec<char>],
    width: usize,
    horizon_y: usize,
    motion: UiMotionContext,
    cloud_pct: f32,
) {
    if width == 0 || horizon_y < 2 || cloud_pct >= 92.0 {
        return;
    }

    let sky_h = horizon_y.saturating_sub(1);
    let star_count = star_count(width, sky_h, cloud_pct);
    if star_count == 0 {
        return;
    }

    let glyphs = ['*', '✶', '✦', '*', '✶', '✦'];
    for i in 0..star_count {
        paint_star(
            canvas,
            width,
            horizon_y,
            sky_h,
            motion.lane("starfield"),
            motion.elapsed_seconds,
            i,
            &glyphs,
        );
    }
}

fn star_count(width: usize, sky_h: usize, cloud_pct: f32) -> usize {
    let base_count = (width * sky_h / 8).max(12);
    let visibility_factor = (1.0 - cloud_pct / 100.0).clamp(0.08, 1.0);
    ((base_count as f32) * visibility_factor).round() as usize
}

#[allow(clippy::too_many_arguments)]
fn paint_star(
    canvas: &mut [Vec<char>],
    width: usize,
    horizon_y: usize,
    sky_h: usize,
    seed: SeededMotion,
    elapsed_seconds: f32,
    i: usize,
    glyphs: &[char],
) {
    let x = ((seed.unit(i as u64 + 3) * width as f32) as usize).min(width.saturating_sub(1));
    let y =
        1 + ((seed.unit(i as u64 + 9) * sky_h.max(1) as f32) as usize).min(sky_h.saturating_sub(1));
    if y >= horizon_y || x >= width || canvas[y][x] != ' ' {
        return;
    }
    let twinkle = seed.pulse(elapsed_seconds, 0.45, i as u64);
    let glyph_idx = (((seed.unit(i as u64 + 14) + twinkle * 0.25) * glyphs.len() as f32) as usize)
        .min(glyphs.len() - 1);
    canvas[y][x] = glyphs[glyph_idx];
}

pub(super) fn place_celestial_body(
    canvas: &mut [Vec<char>],
    is_day: bool,
    moon_visible: bool,
    hour: usize,
    horizon_y: usize,
    width: usize,
) {
    if width < 4 || horizon_y < 3 || (!is_day && !moon_visible) {
        return;
    }

    let (x, y) = celestial_position(hour, horizon_y, width);
    let glyph = if is_day { '◉' } else { '◕' };
    paint_char(canvas, x as isize, y as isize, glyph, true);
    paint_celestial_glow(canvas, width, x, y);
}

fn celestial_position(hour: usize, horizon_y: usize, width: usize) -> (usize, usize) {
    let t = hour as f32 / 23.0;
    let x = (t * (width.saturating_sub(3)) as f32).round() as usize + 1;
    let arc = 1.0 - 4.0 * (t - 0.5) * (t - 0.5);
    let sky_height = horizon_y.saturating_sub(2);
    let y = (horizon_y as f32 - 1.0 - arc * sky_height as f32 * 0.8)
        .round()
        .clamp(1.0, (horizon_y - 1) as f32) as usize;
    (x, y)
}

fn paint_celestial_glow(canvas: &mut [Vec<char>], width: usize, x: usize, y: usize) {
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        paint_char(canvas, x as isize + dx, y as isize + dy, '░', false);
    }
    if width >= 50 {
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
            paint_char(canvas, x as isize + dx, y as isize + dy, '░', false);
        }
    }
}

pub(super) fn moon_visible(bundle: &ForecastBundle, hour: usize) -> bool {
    if bundle.current.is_day {
        return false;
    }
    let Some(day) = bundle.daily.first() else {
        return true;
    };
    let (Some(sunrise), Some(sunset)) = (day.sunrise, day.sunset) else {
        return true;
    };
    let sunrise_h = hm_to_hour_f32(&sunrise);
    let sunset_h = hm_to_hour_f32(&sunset);
    let hour = hour as f32;

    if sunset_h > sunrise_h {
        hour >= sunset_h || hour < sunrise_h
    } else {
        // Handles edge cases where provided times invert (e.g. polar regions).
        hour >= sunset_h && hour < sunrise_h
    }
}

fn hm_to_hour_f32(dt: &chrono::NaiveDateTime) -> f32 {
    dt.hour() as f32 + dt.minute() as f32 / 60.0
}

pub(super) fn paint_cloud_layer(
    canvas: &mut [Vec<char>],
    cloud_pct: f32,
    wind_speed: f32,
    seed: SeededMotion,
    elapsed_seconds: f32,
    horizon_y: usize,
    width: usize,
) {
    if cloud_pct < 5.0 || horizon_y < 4 {
        return;
    }

    let cloud_count = cloud_count(cloud_pct);
    let max_cloud_w = max_cloud_width(cloud_pct, width);
    let cloud_rows = cloud_row_count(cloud_pct);
    let drift = (elapsed_seconds * (2.0 + wind_speed.max(3.0) / 6.0)).round() as usize;
    let sky_band = horizon_y.saturating_sub(2);

    for i in 0..cloud_count {
        let lane_seed = cloud_lane_seed(seed, i);
        let base_x = cloud_base_x(lane_seed, i, width, max_cloud_w, drift);
        let base_y = cloud_base_y(lane_seed, i, sky_band);
        if base_y >= horizon_y.saturating_sub(1) {
            continue;
        }
        let cloud_w = cloud_width(lane_seed, i, max_cloud_w);
        draw_ambient_cloud(
            canvas, base_x, base_y, cloud_w, cloud_rows, width, horizon_y,
        );
    }
}

fn cloud_count(cloud_pct: f32) -> usize {
    ((cloud_pct / 15.0).ceil() as usize).clamp(1, 8)
}

fn max_cloud_width(cloud_pct: f32, width: usize) -> usize {
    if cloud_pct > 80.0 {
        width / 2
    } else if cloud_pct > 50.0 {
        width / 3
    } else {
        width / 5
    }
    .clamp(6, 40)
}

fn cloud_row_count(cloud_pct: f32) -> usize {
    if cloud_pct > 70.0 { 3 } else { 2 }
}

fn cloud_lane_seed(seed: SeededMotion, index: usize) -> SeededMotion {
    seed.lane(match index % 3 {
        0 => "high",
        1 => "mid",
        _ => "low",
    })
}

fn cloud_base_x(
    lane_seed: SeededMotion,
    index: usize,
    width: usize,
    max_cloud_w: usize,
    drift: usize,
) -> usize {
    (((lane_seed.unit(index as u64 + 5) * (width + max_cloud_w) as f32) as usize) + drift)
        % (width + max_cloud_w)
}

fn cloud_base_y(lane_seed: SeededMotion, index: usize, sky_band: usize) -> usize {
    1 + ((lane_seed.unit(index as u64 + 9) * sky_band.max(1) as f32) as usize)
}

fn cloud_width(lane_seed: SeededMotion, index: usize, max_cloud_w: usize) -> usize {
    (max_cloud_w / 2)
        + ((lane_seed.unit(index as u64 + 11) * (max_cloud_w / 2 + 1) as f32) as usize)
}
