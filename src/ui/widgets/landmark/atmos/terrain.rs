pub(super) fn compute_terrain(
    width: usize,
    temps: &[f32],
    min_temp: f32,
    span: f32,
    horizon_y: usize,
    amp: usize,
) -> Vec<usize> {
    let mut tops = vec![horizon_y; width];
    for (x, top) in tops.iter_mut().enumerate() {
        let t = if width <= 1 {
            0.0
        } else {
            x as f32 / (width - 1) as f32
        };
        let pos = t * (temps.len().saturating_sub(1)) as f32;
        let left = pos.floor() as usize;
        let right = pos.ceil().min((temps.len() - 1) as f32) as usize;
        let frac = pos - left as f32;
        let sample = if right > left {
            temps[left] * (1.0 - frac) + temps[right] * frac
        } else {
            temps[left]
        };

        // Add gentle rolling hills with a sine overlay for organic feel.
        let norm = ((sample - min_temp) / span).clamp(0.0, 1.0);
        let hill = (t * std::f32::consts::PI * 2.3).sin() * 0.3 + 0.7;
        let peak = (norm * hill * amp as f32).round() as usize;
        *top = horizon_y.saturating_sub(peak).max(1);
    }
    tops
}

pub(super) fn paint_terrain(
    canvas: &mut [Vec<char>],
    terrain_top: &[usize],
    horizon_y: usize,
    width: usize,
    height: usize,
) {
    for (x, &top) in terrain_top.iter().enumerate().take(width) {
        for (y, row) in canvas.iter_mut().enumerate().take(height) {
            if y < top {
                continue;
            }
            row[x] = terrain_glyph(y, top, horizon_y);
        }
    }
    overlay_horizon_line(canvas, terrain_top, horizon_y, width, height);
}

fn terrain_glyph(y: usize, top: usize, horizon_y: usize) -> char {
    if y == top {
        '▁'
    } else if y == top + 1 {
        '▃'
    } else if y <= horizon_y {
        '▅'
    } else {
        '█'
    }
}

pub(super) fn overlay_horizon_line(
    canvas: &mut [Vec<char>],
    terrain_top: &[usize],
    horizon_y: usize,
    width: usize,
    height: usize,
) {
    if horizon_y >= height {
        return;
    }
    for (x, cell) in canvas[horizon_y].iter_mut().enumerate().take(width) {
        if terrain_top[x] > horizon_y {
            *cell = '─';
        }
    }
}

pub(super) fn paint_horizon_haze(canvas: &mut [Vec<char>], horizon_y: usize, width: usize) {
    // Thin atmospheric haze band just above the horizon.
    let haze_y = horizon_y.saturating_sub(1);
    if haze_y == 0 {
        return;
    }
    for (x, cell) in canvas[haze_y].iter_mut().enumerate().take(width) {
        if *cell == ' ' {
            *cell = if x % 3 == 0 { '░' } else { ' ' };
        }
    }
}
