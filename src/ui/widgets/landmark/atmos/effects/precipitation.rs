use crate::ui::animation::{MotionMode, SeededMotion};

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn paint_rain(
    canvas: &mut [Vec<char>],
    precip_mm: f32,
    phase: usize,
    elapsed_seconds: f32,
    seed: SeededMotion,
    motion_mode: MotionMode,
    horizon_y: usize,
    w: usize,
) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    let phase = phase / 3;
    let profile = rain_profile(precip_mm);
    let columns = rain_columns(profile.density, motion_mode);
    for x in 0..w {
        let lane_shift = (seed.unit(x as u64 + 5) * columns as f32) as usize;
        if !(x + phase + lane_shift).is_multiple_of(columns) {
            continue;
        }
        paint_rain_column(
            canvas,
            x,
            phase,
            elapsed_seconds,
            seed,
            horizon_y,
            profile.drops,
            profile.glyph,
        );
    }
    paint_rain_splashes(canvas, horizon_y, w, phase, seed, columns);
}

struct RainProfile {
    density: usize,
    drops: usize,
    glyph: char,
}

fn rain_profile(precip_mm: f32) -> RainProfile {
    let (density, drops, glyph) = if precip_mm >= 5.0 {
        (2, 3, '/')
    } else if precip_mm >= 1.0 {
        (3, 2, '╱')
    } else {
        (5, 1, '╱')
    };
    RainProfile {
        density,
        drops,
        glyph,
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_rain_column(
    canvas: &mut [Vec<char>],
    x: usize,
    phase: usize,
    elapsed_seconds: f32,
    seed: SeededMotion,
    horizon_y: usize,
    drops: usize,
    ch: char,
) {
    let h = canvas.len();
    for d in 0..drops {
        let fall_offset = (elapsed_seconds * (10.0 + d as f32 * 1.8)).round() as usize;
        let y_offset = phase
            .wrapping_add(fall_offset)
            .wrapping_add(x * 3)
            .wrapping_add(d * 4)
            .wrapping_add((seed.unit((x + d) as u64 + 17) * 7.0) as usize)
            % horizon_y.max(2);
        let y = 1 + y_offset;
        if y < h
            && y < horizon_y
            && let Some(cell) = canvas.get_mut(y).and_then(|row| row.get_mut(x))
            && matches!(*cell, ' ' | '·' | '░' | '▒' | '▓')
        {
            *cell = ch;
        }
    }
}

fn paint_rain_splashes(
    canvas: &mut [Vec<char>],
    horizon_y: usize,
    width: usize,
    phase: usize,
    seed: SeededMotion,
    density: usize,
) {
    if horizon_y >= canvas.len() {
        return;
    }
    for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(width) {
        let lane = (seed.unit(x as u64 + 41) * 3.0) as usize;
        let cadence = density.saturating_add(1);
        if x.wrapping_add(phase / 5)
            .wrapping_add(lane)
            .is_multiple_of(cadence)
            && matches!(*cell, '─' | ' ')
        {
            *cell = '≈';
        }
    }
}

pub(in super::super) fn paint_snowfall(
    canvas: &mut [Vec<char>],
    phase: usize,
    elapsed_seconds: f32,
    seed: SeededMotion,
    motion_mode: MotionMode,
    horizon_y: usize,
    w: usize,
) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    let ctx = SnowLayerContext {
        phase: phase / 20,
        elapsed_seconds,
        seed,
        horizon_y,
        width: w,
        height: canvas.len(),
    };
    let flakes = ['*', '✶', '✦', '❅', '❆', '*'];
    let layer_count = if motion_mode.is_cinematic() { 3 } else { 2 };
    for layer in 0..layer_count {
        paint_snow_layer(canvas, ctx, &flakes, layer);
    }
    paint_snow_accumulation(canvas, horizon_y, w, ctx.height);
}

pub(in super::super) fn paint_hail(
    canvas: &mut [Vec<char>],
    phase: usize,
    elapsed_seconds: f32,
    seed: SeededMotion,
    motion_mode: MotionMode,
    horizon_y: usize,
    w: usize,
) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    let phase = phase / 2;
    let h = canvas.len();
    paint_hailstones(
        canvas,
        phase,
        elapsed_seconds,
        seed,
        motion_mode,
        horizon_y,
        w,
        h,
    );
    paint_hail_bounce_marks(canvas, phase, seed, horizon_y, w, h);
}

#[allow(clippy::too_many_arguments)]
fn paint_hailstones(
    canvas: &mut [Vec<char>],
    phase: usize,
    elapsed_seconds: f32,
    seed: SeededMotion,
    motion_mode: MotionMode,
    horizon_y: usize,
    w: usize,
    h: usize,
) {
    let spacing = if motion_mode.is_cinematic() { 3 } else { 4 };
    for x in 0..w {
        if !(x + phase + (seed.unit(x as u64 + 3) * 2.0) as usize).is_multiple_of(spacing) {
            continue;
        }
        let y = 1
            + (phase
                + (elapsed_seconds * 12.0) as usize
                + x * 3
                + (seed.unit(x as u64 + 7) * 5.0) as usize)
                % horizon_y.max(2);
        if let Some(cell) = hailstone_cell(canvas, x, y, horizon_y, h) {
            *cell = '●';
        }
    }
}

fn hailstone_cell(
    canvas: &mut [Vec<char>],
    x: usize,
    y: usize,
    horizon_y: usize,
    height: usize,
) -> Option<&mut char> {
    if y >= height || y >= horizon_y {
        return None;
    }
    let cell = canvas.get_mut(y).and_then(|row| row.get_mut(x))?;
    if matches!(*cell, ' ' | '░' | '▒') {
        Some(cell)
    } else {
        None
    }
}

fn paint_hail_bounce_marks(
    canvas: &mut [Vec<char>],
    phase: usize,
    seed: SeededMotion,
    horizon_y: usize,
    width: usize,
    height: usize,
) {
    if horizon_y >= height {
        return;
    }
    for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(width) {
        if (x + phase / 4 + (seed.unit(x as u64 + 13) * 2.0) as usize).is_multiple_of(5)
            && *cell == '─'
        {
            *cell = '●';
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SnowLayerContext {
    phase: usize,
    elapsed_seconds: f32,
    seed: SeededMotion,
    horizon_y: usize,
    width: usize,
    height: usize,
}

fn paint_snow_layer(
    canvas: &mut [Vec<char>],
    ctx: SnowLayerContext,
    flakes: &[char],
    layer: usize,
) {
    let fall_speed = 1 + layer;
    let spacing = 5 + layer * 2;
    for x in 0..ctx.width {
        let drift_lane = (ctx.seed.unit((x + layer) as u64 + 31) * 3.0) as usize;
        if !(x + layer * 7 + drift_lane).is_multiple_of(spacing) {
            continue;
        }
        let sway =
            ((ctx.elapsed_seconds * (1.6 + layer as f32 * 0.4)).sin() * 2.0).round() as usize;
        let y_off = (ctx.phase * fall_speed + x * 5 + layer * 11 + sway) % ctx.horizon_y.max(2);
        let y = 1 + y_off;
        if y < ctx.height && y < ctx.horizon_y && matches!(canvas[y][x], ' ' | '░' | '▒' | '▓')
        {
            let flake = flakes[(x + layer + ctx.phase + drift_lane) % flakes.len()];
            canvas[y][x] = flake;
        }
    }
}

fn rain_columns(density: usize, motion_mode: MotionMode) -> usize {
    match motion_mode {
        MotionMode::Cinematic => density.max(2),
        MotionMode::Standard => (density + 1).max(3),
        MotionMode::Reduced => (density + 2).max(4),
        MotionMode::Off => (density + 4).max(8),
    }
}

fn paint_snow_accumulation(
    canvas: &mut [Vec<char>],
    horizon_y: usize,
    width: usize,
    height: usize,
) {
    let top = horizon_y.saturating_sub(1);
    if top >= height {
        return;
    }
    for cell in canvas[top].iter_mut().take(width) {
        if matches!(*cell, '▁' | ' ') {
            *cell = '·';
        }
    }
}
