#[allow(clippy::needless_range_loop)]
pub(in super::super) fn paint_fog_banks(
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
