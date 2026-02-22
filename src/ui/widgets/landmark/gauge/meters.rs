use crate::ui::widgets::shared::sparkline_blocks as shared_sparkline_blocks;

pub(super) fn meter_with_threshold(norm: f32, width: usize, threshold: Option<f32>) -> String {
    let width = width.max(4);
    let fill = (norm.clamp(0.0, 1.0) * width as f32).round() as usize;
    let thresh_pos = threshold.map(|t| (t.clamp(0.0, 1.0) * width as f32).round() as usize);
    let mut cells = vec!['·'; width];
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');

    for cell in cells.iter_mut().take(fill.min(width)) {
        *cell = '█';
    }
    if fill < width {
        cells[fill] = '▓';
        if fill + 1 < width {
            cells[fill + 1] = '▒';
        }
    }
    if let Some(pos) = thresh_pos.filter(|pos| *pos < width) {
        cells[pos] = '|';
    }

    for cell in cells {
        bar.push(cell);
    }
    bar.push(']');
    bar
}

pub(super) fn sparkline_annotated(values: &[f32], width: usize, _suffix: &str) -> String {
    shared_sparkline_blocks(values, width)
}

pub(super) fn temp_range_label(values: &[f32]) -> String {
    range_label(values, "°")
}

pub(super) fn precip_range_label(values: &[f32]) -> String {
    positive_max(values).map_or_else(String::new, |max| format!("{max:.0}mm"))
}

pub(super) fn gust_range_label(values: &[f32]) -> String {
    positive_max(values).map_or_else(String::new, |max| {
        format!("{}m/s", crate::domain::weather::round_wind_speed(max))
    })
}

pub(super) fn range_label(values: &[f32], suffix: &str) -> String {
    if values.is_empty() {
        return String::new();
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    format!("{min:.0}{suffix}–{max:.0}{suffix}")
}

fn positive_max(values: &[f32]) -> Option<f32> {
    let max = values.iter().copied().fold(0.0_f32, f32::max);
    (max > 0.0).then_some(max)
}
