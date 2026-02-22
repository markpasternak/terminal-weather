use super::*;

pub(in super::super) fn paint_ambient_sky_life(
    canvas: &mut [Vec<char>],
    ctx: AmbientSkyLifeContext,
) {
    if !ambient_sky_life_allowed(ctx) {
        return;
    }
    let sky_rows = ctx.horizon_y.saturating_sub(2);
    if sky_rows <= 1 {
        return;
    }
    paint_ambient_birds(canvas, ctx, sky_rows);
}

pub(in super::super) fn paint_star_reflections(
    canvas: &mut [Vec<char>],
    width: usize,
    horizon_y: usize,
    height: usize,
) {
    // Mirror a sparse subset of stars from above the horizon to below it.
    let reflect_depth = (height - horizon_y).min(horizon_y.saturating_sub(1));
    if reflect_depth == 0 || width == 0 {
        return;
    }
    for dy in 1..=reflect_depth {
        let Some((source_y, target_y)) = reflection_rows(dy, horizon_y, height) else {
            continue;
        };
        for x in (0..width).step_by(3) {
            if should_reflect_star(canvas, source_y, target_y, x) {
                canvas[target_y][x] = '✶';
            }
        }
    }
}

fn ambient_sky_life_allowed(ctx: AmbientSkyLifeContext) -> bool {
    ctx.animate
        && ctx.is_day
        && ctx.phase > 0
        && matches!(ctx.category, WeatherCategory::Clear)
        && ctx.cloud_pct <= 25.0
        && ctx.wind_speed <= 10.0
        && ctx.width >= 70
        && ctx.horizon_y >= 8
}

fn reflection_rows(dy: usize, horizon_y: usize, height: usize) -> Option<(usize, usize)> {
    let source_y = horizon_y.saturating_sub(dy);
    let target_y = horizon_y + dy;
    if source_y == 0 || target_y >= height {
        None
    } else {
        Some((source_y, target_y))
    }
}

fn should_reflect_star(canvas: &[Vec<char>], source_y: usize, target_y: usize, x: usize) -> bool {
    matches!(canvas[source_y][x], '*' | '✶' | '✦') && matches!(canvas[target_y][x], '█' | '▅')
}

fn paint_ambient_birds(canvas: &mut [Vec<char>], ctx: AmbientSkyLifeContext, sky_rows: usize) {
    let bird_count = (ctx.width / 90).clamp(1, 2);
    let bird_cycle = (ctx.width * 8).max(1);
    let phase = ctx.phase / 8;
    for i in 0..bird_count {
        let x = ((phase + i * 29) % bird_cycle) as isize - 2;
        let lane = 1 + ((i * 2 + phase / 23) % sky_rows.saturating_sub(2).max(1));
        let wing = if ((phase / 12) + i).is_multiple_of(2) {
            '^'
        } else {
            '~'
        };
        // Render a tiny 3-cell bird sprite so it reads as a bird instead of a lone moving glyph.
        paint_char(canvas, x, lane as isize, '<', false);
        paint_char(canvas, x + 1, lane as isize, wing, false);
        paint_char(canvas, x + 2, lane as isize, '>', false);
    }
}
