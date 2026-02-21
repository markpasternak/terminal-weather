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
    paint_ambient_plane(canvas, ctx);
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
                canvas[target_y][x] = '·';
            }
        }
    }
}

fn ambient_sky_life_allowed(ctx: AmbientSkyLifeContext) -> bool {
    ctx.animate
        && ctx.is_day
        && ctx.phase > 0
        && matches!(
            ctx.category,
            WeatherCategory::Clear | WeatherCategory::Cloudy
        )
        && ctx.cloud_pct <= 85.0
        && ctx.width >= 32
        && ctx.horizon_y >= 6
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
    matches!(canvas[source_y][x], '*' | '✦' | '✧') && matches!(canvas[target_y][x], '█' | '▅')
}

fn paint_ambient_birds(canvas: &mut [Vec<char>], ctx: AmbientSkyLifeContext, sky_rows: usize) {
    let bird_count = (ctx.width / 24).clamp(2, 5);
    let bird_cycle = ctx.width + 12;
    for i in 0..bird_count {
        let speed = 1 + (i % 3);
        let x = ((ctx.phase * speed + i * 17) % bird_cycle) as isize - 6;
        let lane = 1 + ((i * 3 + ctx.phase / 19) % sky_rows.saturating_sub(1).max(1));
        let glyph = if ((ctx.phase / 6) + i).is_multiple_of(2) {
            'v'
        } else {
            'V'
        };
        paint_char(canvas, x, lane as isize, glyph, false);
    }
}

fn paint_ambient_plane(canvas: &mut [Vec<char>], ctx: AmbientSkyLifeContext) {
    let plane_cycle = 220;
    let plane_window = 90;
    let window_phase = ctx.phase % plane_cycle;
    if window_phase >= plane_window {
        return;
    }

    let plane = ['=', '=', '>'];
    let plane_len = plane.len();
    let progress = window_phase as f32 / (plane_window.saturating_sub(1)) as f32;
    let wind_drift = ((ctx.wind_speed / 18.0).round() as isize).clamp(0, 3);
    let plane_span = ctx.width + plane_len + 8;
    let plane_x =
        (progress * plane_span as f32).round() as isize - (plane_len as isize + 4) + wind_drift;
    let lane_count = ctx.horizon_y.saturating_sub(4).max(1);
    let plane_y = 1 + ((ctx.phase / plane_cycle + ctx.width / 9) % lane_count);

    for (idx, ch) in plane.iter().enumerate() {
        paint_char(canvas, plane_x + idx as isize, plane_y as isize, *ch, false);
    }

    let contrail_len = (1 + (ctx.wind_speed as usize / 20)).clamp(1, 3);
    for step in 1..=contrail_len {
        paint_char(
            canvas,
            plane_x - step as isize,
            plane_y as isize,
            '-',
            false,
        );
    }
}
