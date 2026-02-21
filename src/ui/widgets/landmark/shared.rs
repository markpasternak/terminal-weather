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

#[cfg(test)]
mod tests {
    use super::*;

    // ── fit_line ─────────────────────────────────────────────────────────────

    #[test]
    fn fit_line_pads_short_line() {
        let out = fit_line("hi", 6);
        assert_eq!(out.len(), 6);
        assert_eq!(&out[..2], "hi");
    }

    #[test]
    fn fit_line_truncates_long_line() {
        let out = fit_line("abcdefgh", 4);
        assert_eq!(out.chars().count(), 4);
        assert_eq!(out, "abcd");
    }

    #[test]
    fn fit_line_exact_length_unchanged() {
        let out = fit_line("abc", 3);
        assert_eq!(out, "abc");
    }

    // ── fit_lines ────────────────────────────────────────────────────────────

    #[test]
    fn fit_lines_pads_to_target_height() {
        let lines = vec!["a".to_string()];
        let out = fit_lines(lines, 5, 3);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].chars().count(), 5);
        assert_eq!(out[1].chars().count(), 5); // padded
    }

    #[test]
    fn fit_lines_truncates_excess_rows() {
        let lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = fit_lines(lines, 4, 2);
        assert_eq!(out.len(), 2);
    }

    // ── fit_lines_centered ───────────────────────────────────────────────────

    #[test]
    fn fit_lines_centered_centers_vertically() {
        let lines = vec!["x".to_string()];
        let out = fit_lines_centered(lines, 4, 5);
        assert_eq!(out.len(), 5);
        // Should have padding rows before and after
        let non_blank = out.iter().filter(|l| l.trim() != "").count();
        assert_eq!(non_blank, 1);
    }

    #[test]
    fn fit_lines_centered_full_height_no_padding() {
        let lines = vec!["a".to_string(), "b".to_string()];
        let out = fit_lines_centered(lines, 3, 2);
        assert_eq!(out.len(), 2);
    }

    // ── canvas_to_lines ──────────────────────────────────────────────────────

    #[test]
    fn canvas_to_lines_produces_correct_rows() {
        let canvas = vec![vec!['a', 'b', 'c'], vec!['d', 'e', 'f']];
        let lines = canvas_to_lines(canvas, 3);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "abc");
        assert_eq!(lines[1], "def");
    }

    // ── paint_char ───────────────────────────────────────────────────────────

    #[test]
    fn paint_char_writes_to_valid_cell() {
        let mut canvas = vec![vec![' '; 5]; 5];
        paint_char(&mut canvas, 2, 2, 'X', true);
        assert_eq!(canvas[2][2], 'X');
    }

    #[test]
    fn paint_char_does_not_overwrite_non_space_without_force() {
        let mut canvas = vec![vec!['A'; 5]; 5];
        paint_char(&mut canvas, 1, 1, 'Z', false);
        assert_eq!(canvas[1][1], 'A'); // not overwritten
    }

    #[test]
    fn paint_char_force_overwrites_any_cell() {
        let mut canvas = vec![vec!['A'; 5]; 5];
        paint_char(&mut canvas, 1, 1, 'Z', true);
        assert_eq!(canvas[1][1], 'Z');
    }

    #[test]
    fn paint_char_ignores_negative_coords() {
        let mut canvas = vec![vec![' '; 5]; 5];
        paint_char(&mut canvas, -1, 0, 'X', true);
        // No panic, canvas unchanged
        assert!(canvas.iter().flatten().all(|c| *c == ' '));
    }

    #[test]
    fn paint_char_ignores_out_of_bounds() {
        let mut canvas = vec![vec![' '; 5]; 5];
        paint_char(&mut canvas, 10, 10, 'X', true);
        assert!(canvas.iter().flatten().all(|c| *c == ' '));
    }

    // ── compass_arrow ────────────────────────────────────────────────────────

    #[test]
    fn compass_arrow_covers_all_eight_directions() {
        let cases = [
            (0.0, '↑'),
            (45.0, '↗'),
            (90.0, '→'),
            (135.0, '↘'),
            (180.0, '↓'),
            (225.0, '↙'),
            (270.0, '←'),
            (315.0, '↖'),
        ];
        for (deg, expected) in cases {
            assert_eq!(compass_arrow(deg), expected, "deg={deg}");
        }
    }

    #[test]
    fn compass_arrow_wraps_360() {
        assert_eq!(compass_arrow(360.0), compass_arrow(0.0));
    }

    // ── compass_short ────────────────────────────────────────────────────────

    #[test]
    fn compass_short_returns_cardinal_labels() {
        assert_eq!(compass_short(0.0), "N");
        assert_eq!(compass_short(90.0), "E");
        assert_eq!(compass_short(180.0), "S");
        assert_eq!(compass_short(270.0), "W");
    }
}
