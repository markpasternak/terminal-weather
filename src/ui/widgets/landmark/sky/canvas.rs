use crate::ui::animation::UiMotionContext;
use crate::ui::widgets::landmark::shared::paint_char;

use super::glyphs::{arc_glyph, center_symbol};

pub(super) fn arc_bounds(height: usize, horizon_y: usize) -> (usize, usize) {
    let max_bottom = horizon_y
        .saturating_sub(2)
        .clamp(4, height.saturating_sub(4).max(4));
    (
        1usize,
        (height.saturating_mul(63) / 100).clamp(4, max_bottom),
    )
}

#[allow(clippy::needless_range_loop)]
pub(super) fn draw_arc(canvas: &mut [Vec<char>], width: usize, top: usize, bottom: usize) {
    let mid_x = width / 2;
    for x in 0..width {
        let y = locate_arc_y(x, width, top, bottom);
        let ch = arc_glyph(x, y, width, top, mid_x);
        canvas[y][x] = ch;
    }
}

pub(super) fn paint_sun_event_markers(
    canvas: &mut [Vec<char>],
    width: usize,
    arc_top: usize,
    arc_bottom: usize,
) {
    let sunrise_x = 0usize;
    let sunset_x = width.saturating_sub(1);
    let sunrise_y = locate_arc_y(sunrise_x, width, arc_top, arc_bottom);
    let sunset_y = locate_arc_y(sunset_x, width, arc_top, arc_bottom);

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

pub(super) fn paint_solar_noon_marker(canvas: &mut [Vec<char>], width: usize, arc_top: usize) {
    if width == 0 {
        return;
    }
    let noon_x = width / 2;
    let noon_y = arc_top.saturating_sub(1);
    paint_char(canvas, noon_x as isize, noon_y as isize, '┬', true);
}

pub(super) fn paint_celestial_guide(
    canvas: &mut [Vec<char>],
    x: usize,
    body_y: usize,
    horizon_y: usize,
) {
    if body_y >= horizon_y {
        return;
    }
    for y in body_y.saturating_add(1)..horizon_y {
        paint_char(canvas, x as isize, y as isize, '│', false);
    }
}

pub(super) fn paint_night_stars(
    canvas: &mut [Vec<char>],
    width: usize,
    arc_bottom: usize,
    horizon_y: usize,
    is_day: bool,
    motion: UiMotionContext,
) {
    if is_day {
        return;
    }
    let max_star_y = arc_bottom
        .saturating_sub(2)
        .min(horizon_y.saturating_sub(3))
        .max(2);
    if max_star_y <= 1 {
        return;
    }

    let star_count = (width / 5).max(6);
    for i in 0..star_count {
        let seed = motion.lane("night-stars");
        let x = (((i as f32 + 1.0) / (star_count as f32 + 1.0)) * width as f32)
            .round()
            .clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let y = 1
            + (((seed.unit(i as u64 + 7) * 0.70) * max_star_y as f32) as usize)
                .min(max_star_y.saturating_sub(1));
        let twinkle = seed.pulse(motion.elapsed_seconds, 0.6, i as u64);
        if canvas[y][x] == ' ' {
            canvas[y][x] = if twinkle > 0.82 {
                '✦'
            } else if twinkle > 0.60 {
                '*'
            } else {
                '·'
            };
        }
    }
}

pub(super) fn locate_arc_y(x: usize, width: usize, top: usize, bottom: usize) -> usize {
    let mid = (width.saturating_sub(1)) as f32 / 2.0;
    let radius = (width as f32 * 0.46).max(1.0);
    let dx = (x as f32 - mid) / radius;
    (bottom as f32 - (1.0 - dx * dx).max(0.0) * (bottom - top) as f32)
        .round()
        .clamp(top as f32, bottom as f32) as usize
}

pub(super) fn draw_celestial_icon(
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
