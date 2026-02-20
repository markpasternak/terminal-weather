#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

#[must_use]
pub fn canvas_to_lines(canvas: Vec<Vec<char>>, width: usize) -> Vec<String> {
    canvas
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .map(|line| fit_line(&line, width))
        .collect()
}

#[must_use]
pub fn fit_lines(mut lines: Vec<String>, width: usize, height: usize) -> Vec<String> {
    lines = lines
        .into_iter()
        .map(|line| fit_line(&line, width))
        .take(height)
        .collect::<Vec<_>>();
    while lines.len() < height {
        lines.push(" ".repeat(width));
    }
    lines
}

#[must_use]
pub fn fit_line(line: &str, width: usize) -> String {
    let mut out = line.chars().take(width).collect::<String>();
    let len = out.chars().count();
    if len < width {
        out.push_str(&" ".repeat(width - len));
    }
    out
}

#[must_use]
pub fn fit_lines_centered(lines: Vec<String>, width: usize, height: usize) -> Vec<String> {
    let trimmed = lines
        .into_iter()
        .map(|line| fit_line(&line, width))
        .take(height)
        .collect::<Vec<_>>();
    if trimmed.len() >= height {
        return trimmed;
    }

    let pad = (height - trimmed.len()) / 2;
    let mut out = Vec::with_capacity(height);
    for _ in 0..pad {
        out.push(" ".repeat(width));
    }
    out.extend(trimmed);
    while out.len() < height {
        out.push(" ".repeat(width));
    }
    out
}

pub fn paint_char(canvas: &mut [Vec<char>], x: isize, y: isize, ch: char, force: bool) {
    let (Ok(ux), Ok(uy)) = (usize::try_from(x), usize::try_from(y)) else {
        return;
    };
    let Some(row) = canvas.get_mut(uy) else {
        return;
    };
    let Some(cell) = row.get_mut(ux) else {
        return;
    };
    if force || matches!(*cell, ' ' | '·') {
        *cell = ch;
    }
}

#[must_use]
pub fn compass_arrow(deg: f32) -> char {
    const ARROWS: [char; 8] = ['↑', '↗', '→', '↘', '↓', '↙', '←', '↖'];
    ARROWS[compass_sector(deg)]
}

#[must_use]
pub fn compass_short(deg: f32) -> &'static str {
    const LABELS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    LABELS[compass_sector(deg)]
}

fn compass_sector(deg: f32) -> usize {
    ((deg.rem_euclid(360.0) / 45.0) + 0.5).floor() as usize % 8
}
