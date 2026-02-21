pub(in super::super) fn paint_rain(
    canvas: &mut [Vec<char>],
    precip_mm: f32,
    phase: usize,
    horizon_y: usize,
    w: usize,
) {
    if w == 0 || horizon_y < 3 {
        return;
    }
    let profile = rain_profile(precip_mm);
    for x in 0..w {
        if !(x + phase).is_multiple_of(profile.density) {
            continue;
        }
        paint_rain_column(canvas, x, phase, horizon_y, profile.drops, profile.glyph);
    }
    paint_rain_splashes(canvas, horizon_y, w, phase, profile.density);
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

pub(in super::super) fn paint_snowfall(
    canvas: &mut [Vec<char>],
    phase: usize,
    horizon_y: usize,
    w: usize,
) {
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

pub(in super::super) fn paint_hail(
    canvas: &mut [Vec<char>],
    phase: usize,
    horizon_y: usize,
    w: usize,
) {
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
        if y < height && y < horizon_y && matches!(canvas[y][x], ' ' | '░' | '▒' | '▓') {
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
