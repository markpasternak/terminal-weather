use chrono::Timelike;

use crate::domain::weather::ForecastBundle;
use crate::ui::widgets::landmark::shared::paint_char;

use super::effects::draw_ambient_cloud;

pub(super) fn is_night(bundle: &ForecastBundle) -> bool {
    !bundle.current.is_day
}

pub(super) fn paint_starfield(
    canvas: &mut [Vec<char>],
    width: usize,
    horizon_y: usize,
    phase: usize,
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
        paint_star(canvas, width, horizon_y, sky_h, phase, i, &glyphs);
    }
}

fn star_count(width: usize, sky_h: usize, cloud_pct: f32) -> usize {
    let base_count = (width * sky_h / 8).max(12);
    let visibility_factor = (1.0 - cloud_pct / 100.0).clamp(0.08, 1.0);
    ((base_count as f32) * visibility_factor).round() as usize
}

fn paint_star(
    canvas: &mut [Vec<char>],
    width: usize,
    horizon_y: usize,
    sky_h: usize,
    phase: usize,
    i: usize,
    glyphs: &[char],
) {
    let seed = i.wrapping_mul(7919).wrapping_add(31);
    let x = seed % width;
    let y = 1 + (seed / width) % sky_h;
    if y >= horizon_y || x >= width || canvas[y][x] != ' ' {
        return;
    }
    if (i + phase / 4).is_multiple_of(7) {
        return;
    }
    let glyph_idx = (seed / 3) % glyphs.len();
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
    phase: usize,
    horizon_y: usize,
    width: usize,
) {
    if cloud_pct < 5.0 || horizon_y < 4 {
        return;
    }

    // Number and size of clouds scale with cloud coverage.
    let cloud_count = ((cloud_pct / 15.0).ceil() as usize).clamp(1, 8);
    let max_cloud_w = if cloud_pct > 80.0 {
        width / 2
    } else if cloud_pct > 50.0 {
        width / 3
    } else {
        width / 5
    }
    .clamp(6, 40);
    let cloud_rows = if cloud_pct > 70.0 { 3 } else { 2 };

    // Wind drift offset.
    let drift = (phase as f32 * wind_speed.max(3.0) / 40.0) as usize;

    let sky_band = horizon_y.saturating_sub(2);
    for i in 0..cloud_count {
        let seed = i.wrapping_mul(4001).wrapping_add(17);
        let base_x = (seed.wrapping_mul(13) + drift) % (width + max_cloud_w);
        let base_y = 1 + (seed % sky_band.max(1));
        if base_y >= horizon_y.saturating_sub(1) {
            continue;
        }
        let cloud_w = (max_cloud_w / 2) + (seed % (max_cloud_w / 2 + 1));
        draw_ambient_cloud(
            canvas, base_x, base_y, cloud_w, cloud_rows, width, horizon_y,
        );
    }
}
