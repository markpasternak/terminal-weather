use crate::ui::animation::{MotionMode, SeededMotion};

pub(in super::super) fn paint_lightning_bolts(
    canvas: &mut [Vec<char>],
    phase: usize,
    elapsed_seconds: f32,
    seed: SeededMotion,
    motion_mode: MotionMode,
    horizon_y: usize,
    w: usize,
) {
    if !lightning_visible(w, horizon_y, elapsed_seconds, seed, motion_mode) {
        return;
    }
    let bolt_count = if motion_mode.is_cinematic() && seed.pulse(elapsed_seconds, 0.9, 0) > 0.72 {
        2
    } else {
        1
    };
    for b in 0..bolt_count {
        let start_x = (((seed.unit(b as u64 + 5) * (w.saturating_sub(3)) as f32) as usize)
            + phase % (w / 5 + 1))
            .min(w.saturating_sub(3));
        draw_lightning_bolt(canvas, start_x, horizon_y, w);
    }
}

pub(in super::super) fn paint_heat_shimmer(
    canvas: &mut [Vec<char>],
    phase: usize,
    horizon_y: usize,
    w: usize,
) {
    // Subtle rising heat distortion near the terrain on clear hot days
    if horizon_y < 3 {
        return;
    }
    let shimmer_band = horizon_y.saturating_sub(2)..horizon_y;
    for y in shimmer_band {
        if y >= canvas.len() {
            break;
        }
        paint_heat_shimmer_row(&mut canvas[y], phase, w);
    }
}

pub(in super::super) fn paint_ice_glaze(canvas: &mut [Vec<char>], horizon_y: usize, w: usize) {
    // Ice crystals on terrain surface for freezing rain/drizzle
    if horizon_y >= canvas.len() {
        return;
    }
    paint_surface_ice(canvas, horizon_y, w);
    paint_above_ice(canvas, horizon_y, w);
}

fn paint_heat_shimmer_row(row: &mut [char], phase: usize, width: usize) {
    let phase = phase / 4;
    for (x, cell) in row.iter_mut().enumerate().take(width) {
        if !(x + phase).is_multiple_of(6) {
            continue;
        }
        let wave = ((x as f32 * 0.18) + phase as f32 * 0.05).sin();
        if wave > 0.85 && *cell == ' ' {
            *cell = '~';
        }
    }
}

fn lightning_visible(
    width: usize,
    horizon_y: usize,
    elapsed_seconds: f32,
    seed: SeededMotion,
    motion_mode: MotionMode,
) -> bool {
    if width < 6 || horizon_y < 5 || !motion_mode.allows_animation() {
        return false;
    }

    let pulse = seed.pulse(elapsed_seconds, 0.75, 1);
    let threshold = if motion_mode.is_cinematic() {
        0.88
    } else {
        0.94
    };
    pulse > threshold
}

fn draw_lightning_bolt(canvas: &mut [Vec<char>], start_x: usize, horizon_y: usize, width: usize) {
    let mut x = start_x;
    for (y, row) in canvas
        .iter_mut()
        .enumerate()
        .take(horizon_y.saturating_sub(1))
        .skip(1)
    {
        if x >= width {
            break;
        }
        row[x] = if y % 2 == 0 { '╲' } else { '╱' };
        if y % 2 == 0 && x + 1 < width {
            x += 1;
        } else {
            x = x.saturating_sub(1);
        }
    }
}

fn paint_surface_ice(canvas: &mut [Vec<char>], horizon_y: usize, width: usize) {
    for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(width) {
        if !x.is_multiple_of(2) {
            continue;
        }
        if matches!(*cell, '─' | '.' | ',' | ' ' | '∴') {
            *cell = '❆';
        }
    }
}

fn paint_above_ice(canvas: &mut [Vec<char>], horizon_y: usize, width: usize) {
    let above = horizon_y.saturating_sub(1);
    if above >= canvas.len() {
        return;
    }
    for (x, cell) in canvas[above].iter_mut().enumerate().take(width) {
        if x.is_multiple_of(4) && *cell == ' ' {
            *cell = '░';
        }
    }
}
