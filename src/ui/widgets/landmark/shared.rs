pub fn canvas_to_lines(canvas: Vec<Vec<char>>, width: usize) -> Vec<String> {
    canvas
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .map(|line| fit_line(&line, width))
        .collect()
}

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

pub fn fit_line(line: &str, width: usize) -> String {
    let mut out = line.chars().take(width).collect::<String>();
    let len = out.chars().count();
    if len < width {
        out.push_str(&" ".repeat(width - len));
    }
    out
}

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
    if canvas.is_empty() || canvas[0].is_empty() {
        return;
    }
    if x < 0 || y < 0 {
        return;
    }
    let ux = x as usize;
    let uy = y as usize;
    if uy >= canvas.len() || ux >= canvas[0].len() {
        return;
    }

    let current = canvas[uy][ux];
    if force || matches!(current, ' ' | '·') {
        canvas[uy][ux] = ch;
    }
}

pub fn compass_arrow(deg: f32) -> char {
    let d = deg.rem_euclid(360.0);
    match d as i32 {
        23..=67 => '↗',
        68..=112 => '→',
        113..=157 => '↘',
        158..=202 => '↓',
        203..=247 => '↙',
        248..=292 => '←',
        293..=337 => '↖',
        _ => '↑',
    }
}

pub fn compass_short(deg: f32) -> &'static str {
    let d = deg.rem_euclid(360.0);
    match d as i32 {
        23..=67 => "NE",
        68..=112 => "E",
        113..=157 => "SE",
        158..=202 => "S",
        203..=247 => "SW",
        248..=292 => "W",
        293..=337 => "NW",
        _ => "N",
    }
}
