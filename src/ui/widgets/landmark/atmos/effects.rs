use super::*;

pub(super) fn paint_ambient_sky_life(canvas: &mut [Vec<char>], ctx: AmbientSkyLifeContext) {
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

pub(super) fn draw_ambient_cloud(
    canvas: &mut [Vec<char>],
    cx: usize,
    cy: usize,
    cloud_w: usize,
    rows: usize,
    canvas_w: usize,
    horizon_y: usize,
) {
    // Multi-row cloud with density falloff from center
    // Top row: lighter/thinner  ░░▒▒▒░░
    // Middle row: denser         ▒▓▓▓▓▓▒
    // Bottom row: wispy          ░░▒░░
    let patterns = cloud_patterns(rows);

    let cloud = CloudGeometry {
        cx,
        cy,
        width: cloud_w,
        canvas_width: canvas_w,
        horizon_y,
    };
    for (row_idx, pattern) in patterns.iter().enumerate() {
        if !paint_cloud_row(canvas, pattern, row_idx, &cloud) {
            break;
        }
    }
}

struct CloudGeometry {
    cx: usize,
    cy: usize,
    width: usize,
    canvas_width: usize,
    horizon_y: usize,
}

fn cloud_patterns(rows: usize) -> &'static [&'static [char]] {
    if rows >= 3 {
        &[
            &[' ', '░', '░', '▒', '▒', '░', '░', ' '],
            &['░', '▒', '▓', '▓', '▓', '▓', '▒', '░'],
            &[' ', ' ', '░', '▒', '▒', '░', ' ', ' '],
        ]
    } else {
        &[
            &[' ', '░', '▒', '▒', '▒', '░', ' '],
            &['░', '▒', '▓', '▓', '▒', '░', ' '],
        ]
    }
}

fn paint_cloud_row(
    canvas: &mut [Vec<char>],
    pattern: &[char],
    row_idx: usize,
    cloud: &CloudGeometry,
) -> bool {
    let y = cloud.cy + row_idx;
    if y >= cloud.horizon_y || y >= canvas.len() {
        return false;
    }
    let pat_len = pattern.len();
    for col in 0..cloud.width {
        let x = cloud.cx.wrapping_sub(cloud.width / 2).wrapping_add(col);
        if x >= cloud.canvas_width {
            continue;
        }
        let pat_idx = (col * pat_len) / cloud.width.max(1);
        let ch = pattern[pat_idx.min(pat_len - 1)];
        if ch != ' ' && canvas[y][x] == ' ' {
            canvas[y][x] = ch;
        }
    }
    true
}

pub(super) fn paint_rain(
    canvas: &mut [Vec<char>],
    precip_mm: f32,
    phase: usize,
    horizon_y: usize,
    w: usize,
) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    let density = rain_density(precip_mm);
    let drops = rain_drops_per_column(precip_mm);
    let ch = rain_glyph(precip_mm);
    for x in 0..w {
        if !(x + phase).is_multiple_of(density) {
            continue;
        }
        paint_rain_column(canvas, x, phase, horizon_y, drops, ch);
    }
    paint_rain_splashes(canvas, horizon_y, w, phase, density);
}

fn rain_density(precip_mm: f32) -> usize {
    if precip_mm >= 5.0 {
        2
    } else if precip_mm >= 1.0 {
        3
    } else {
        5
    }
}

fn rain_drops_per_column(precip_mm: f32) -> usize {
    if precip_mm >= 5.0 {
        3
    } else if precip_mm >= 1.0 {
        2
    } else {
        1
    }
}

fn rain_glyph(precip_mm: f32) -> char {
    if precip_mm >= 3.0 { '/' } else { '╱' }
}

fn paint_rain_column(
    canvas: &mut [Vec<char>],
    x: usize,
    phase: usize,
    horizon_y: usize,
    drops: usize,
    ch: char,
) {
    let h = canvas.len();
    for d in 0..drops {
        let y_offset = (phase + x * 3 + d * 4) % horizon_y.max(2);
        let y = 1 + y_offset;
        if y < h
            && y < horizon_y
            && let Some(cell) = canvas.get_mut(y).and_then(|row| row.get_mut(x))
            && matches!(*cell, ' ' | '·')
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
    density: usize,
) {
    if horizon_y >= canvas.len() {
        return;
    }
    for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(width) {
        if (x + phase / 2).is_multiple_of(density + 1) && matches!(*cell, '─' | ' ') {
            *cell = '.';
        }
    }
}

pub(super) fn paint_snowfall(canvas: &mut [Vec<char>], phase: usize, horizon_y: usize, w: usize) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    let h = canvas.len();
    let flakes = ['·', '*', '✧', '·', '·', '*'];
    for layer in 0..3 {
        paint_snow_layer(canvas, phase, horizon_y, w, h, &flakes, layer);
    }
    paint_snow_accumulation(canvas, horizon_y, w, h);
}

#[allow(clippy::needless_range_loop)]
pub(super) fn paint_fog_banks(
    canvas: &mut [Vec<char>],
    phase: usize,
    horizon_y: usize,
    w: usize,
    h: usize,
) {
    if w == 0 {
        return;
    }
    paint_fog_bands(canvas, phase, horizon_y, w, h);
    paint_upper_mist(canvas, phase, horizon_y, w, h);
}

pub(super) fn paint_lightning_bolts(
    canvas: &mut [Vec<char>],
    phase: usize,
    horizon_y: usize,
    w: usize,
) {
    if !lightning_visible(w, horizon_y, phase) {
        return;
    }
    let bolt_count = 1 + (phase / 7) % 2;
    for b in 0..bolt_count {
        let start_x = (w / 3 + b * w / 3 + phase % (w / 4 + 1)).min(w.saturating_sub(3));
        draw_lightning_bolt(canvas, start_x, horizon_y, w);
    }
}

pub(super) fn paint_heat_shimmer(
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
        for (x, cell) in canvas[y].iter_mut().enumerate().take(w) {
            let wave = ((x + phase) as f32 * 0.4).sin();
            if wave > 0.6 && *cell == ' ' {
                *cell = if (x + phase).is_multiple_of(3) {
                    '.'
                } else {
                    ','
                };
            }
        }
    }
}

pub(super) fn paint_ice_glaze(canvas: &mut [Vec<char>], horizon_y: usize, w: usize) {
    // Ice crystals on terrain surface for freezing rain/drizzle
    if horizon_y >= canvas.len() {
        return;
    }
    paint_surface_ice(canvas, horizon_y, w);
    paint_above_ice(canvas, horizon_y, w);
}

pub(super) fn paint_hail(canvas: &mut [Vec<char>], phase: usize, horizon_y: usize, w: usize) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    let h = canvas.len();
    paint_hailstones(canvas, phase, horizon_y, w, h);
    paint_hail_bounce_marks(canvas, phase, horizon_y, w, h);
}

fn paint_hailstones(canvas: &mut [Vec<char>], phase: usize, horizon_y: usize, w: usize, h: usize) {
    for x in 0..w {
        if !(x + phase).is_multiple_of(4) {
            continue;
        }
        let y = 1 + (phase + x * 3) % horizon_y.max(2);
        if let Some(cell) = hailstone_cell(canvas, x, y, horizon_y, h) {
            *cell = 'o';
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
    if matches!(*cell, ' ' | '·') {
        Some(cell)
    } else {
        None
    }
}

fn paint_hail_bounce_marks(
    canvas: &mut [Vec<char>],
    phase: usize,
    horizon_y: usize,
    width: usize,
    height: usize,
) {
    if horizon_y >= height {
        return;
    }
    for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(width) {
        if (x + phase / 2).is_multiple_of(5) && *cell == '─' {
            *cell = '•';
        }
    }
}

fn paint_snow_layer(
    canvas: &mut [Vec<char>],
    phase: usize,
    horizon_y: usize,
    width: usize,
    height: usize,
    flakes: &[char],
    layer: usize,
) {
    let speed = layer + 1;
    let spacing = 3 + layer;
    for x in 0..width {
        if !(x + layer * 7).is_multiple_of(spacing) {
            continue;
        }
        let y_off = (phase * speed / 2 + x * 5 + layer * 11) % horizon_y.max(2);
        let y = 1 + y_off;
        if y < height && y < horizon_y && canvas[y][x] == ' ' {
            let flake = flakes[(x + layer + phase) % flakes.len()];
            canvas[y][x] = flake;
        }
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
            *cell = '∴';
        }
    }
}

fn paint_fog_bands(
    canvas: &mut [Vec<char>],
    phase: usize,
    horizon_y: usize,
    width: usize,
    height: usize,
) {
    let density_chars = ['░', '░', '▒', '░'];
    for band in 0..4 {
        let base_y = horizon_y.saturating_sub(3) + band;
        if base_y >= height {
            continue;
        }
        let drift = (phase + band * 7) % width;
        let row = &mut canvas[base_y];
        for (x, cell) in row.iter_mut().enumerate().take(width) {
            let shifted = (x + drift) % width;
            let wave = ((shifted as f32 / width as f32) * std::f32::consts::PI * 3.0).sin();
            if wave > -0.2 && *cell == ' ' {
                let idx = ((wave + 1.0) / 2.0 * (density_chars.len() - 1) as f32).round() as usize;
                *cell = density_chars[idx.min(density_chars.len() - 1)];
            }
        }
    }
}

fn paint_upper_mist(
    canvas: &mut [Vec<char>],
    phase: usize,
    horizon_y: usize,
    width: usize,
    height: usize,
) {
    let upper = horizon_y.saturating_sub(3).min(height);
    for (y, row) in canvas.iter_mut().enumerate().take(upper).skip(2) {
        for (x, cell) in row.iter_mut().enumerate().take(width) {
            if (x + y + phase / 3).is_multiple_of(7) && *cell == ' ' {
                *cell = '·';
            }
        }
    }
}

fn lightning_visible(width: usize, horizon_y: usize, phase: usize) -> bool {
    width >= 6 && horizon_y >= 5 && (phase / 3).is_multiple_of(5)
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
        if matches!(*cell, '─' | '.' | ',' | ' ') {
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
            *cell = '·';
        }
    }
}
