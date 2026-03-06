use super::*;

#[derive(Debug, Clone)]
struct ChartScaleLabels {
    temp_max: String,
    temp_min: String,
    precip_peak: String,
}

#[derive(Debug, Clone, Copy)]
struct TimelinePlotLayout {
    left: usize,
    right: usize,
    plot: usize,
}

pub(super) fn expanded_timeline_lines(
    series: &TimelineSeries,
    width: usize,
    height: u16,
    theme: Theme,
) -> Vec<TimelineLine> {
    let labels = chart_scale_labels(series);
    let layout = timeline_plot_layout(width, &labels);
    if layout.plot < 12 {
        return super::compact_timeline_lines(series, width.saturating_sub(7), height, theme);
    }

    let temp_height = usize::from(height.saturating_sub(2));
    let temp_canvas = temperature_canvas(series, layout.plot, temp_height);
    let precip_band = compact_precip_band(&series.precips, layout.plot);
    let time_axis = time_axis_line(&series.times, layout.plot);
    let mut lines = Vec::with_capacity(height as usize);

    lines.extend(temperature_plot_lines(
        temp_canvas,
        temp_height,
        &labels,
        layout,
        theme,
    ));
    lines.extend(chart_footer_lines(
        precip_band,
        time_axis,
        &labels.precip_peak,
        layout,
        theme,
    ));
    lines
}

fn temperature_plot_lines(
    temp_canvas: Vec<String>,
    temp_height: usize,
    labels: &ChartScaleLabels,
    layout: TimelinePlotLayout,
    theme: Theme,
) -> Vec<TimelineLine> {
    temp_canvas
        .into_iter()
        .enumerate()
        .map(|(row_idx, plot_row)| {
            let label = if row_idx == 0 { "Temp" } else { "" };
            let scale = if row_idx == 0 {
                labels.temp_max.as_str()
            } else if row_idx + 1 == temp_height {
                labels.temp_min.as_str()
            } else {
                ""
            };
            timeline_plot_row(
                label,
                plot_row,
                scale,
                layout,
                theme.muted_text,
                theme.accent,
                theme.text,
            )
        })
        .collect()
}

fn chart_footer_lines(
    precip_band: String,
    time_axis: String,
    precip_peak: &str,
    layout: TimelinePlotLayout,
    theme: Theme,
) -> [TimelineLine; 2] {
    [
        timeline_plot_row(
            "Rain",
            precip_band,
            precip_peak,
            layout,
            theme.muted_text,
            theme.info,
            theme.info,
        ),
        timeline_plot_row(
            "Time",
            time_axis,
            "",
            layout,
            theme.muted_text,
            theme.text,
            theme.text,
        ),
    ]
}

fn chart_scale_labels(series: &TimelineSeries) -> ChartScaleLabels {
    let concrete = series.temps.iter().flatten().copied().collect::<Vec<_>>();
    let (temp_min, temp_max) = if concrete.is_empty() {
        ("--".to_string(), "--".to_string())
    } else {
        let min = concrete.iter().copied().fold(f32::INFINITY, f32::min);
        let max = concrete.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (
            format!("{}{}", round_temp(min), series.temp_unit),
            format!("{}{}", round_temp(max), series.temp_unit),
        )
    };
    let precip_peak = series
        .precips
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let precip_peak = if precip_peak.is_finite() {
        format!("{:.1}mm/h", precip_peak.max(0.0))
    } else {
        "--.-mm/h".to_string()
    };

    ChartScaleLabels {
        temp_max,
        temp_min,
        precip_peak,
    }
}

fn timeline_plot_layout(width: usize, labels: &ChartScaleLabels) -> TimelinePlotLayout {
    let left = 6;
    let right = [
        labels.temp_max.chars().count(),
        labels.temp_min.chars().count(),
        labels.precip_peak.chars().count(),
    ]
    .into_iter()
    .max()
    .unwrap_or(0)
    .max(7);
    let plot = width.saturating_sub(left + right + 1);
    TimelinePlotLayout { left, right, plot }
}

fn timeline_plot_row(
    label: &str,
    plot: String,
    scale: &str,
    layout: TimelinePlotLayout,
    label_color: Color,
    plot_color: Color,
    scale_color: Color,
) -> TimelineLine {
    Line::from(vec![
        Span::styled(
            format!("{label:<width$}", width = layout.left),
            Style::default().fg(label_color),
        ),
        Span::styled(plot, Style::default().fg(plot_color)),
        Span::raw(" "),
        Span::styled(
            format!("{scale:>width$}", width = layout.right),
            Style::default().fg(scale_color),
        ),
    ])
}

fn temperature_canvas(series: &TimelineSeries, width: usize, height: usize) -> Vec<String> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let concrete = series.temps.iter().flatten().copied().collect::<Vec<_>>();
    let mut grid = vec![vec![' '; width]; height];
    if concrete.is_empty() {
        return grid
            .into_iter()
            .map(|row| row.into_iter().collect())
            .collect();
    }

    let min = concrete.iter().copied().fold(f32::INFINITY, f32::min);
    let max = concrete.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max - min).max(0.001);

    for col in 0..width {
        let idx = sample_index(col, width, series.temps.len());
        let Some(value) = series.temps[idx] else {
            continue;
        };
        let row = scaled_plot_row(value, min, span, height);
        for canvas_row in grid.iter_mut().take(height).skip(row.saturating_add(1)) {
            canvas_row[col] = '░';
        }
        grid[row][col] = '█';
    }

    grid.into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

fn scaled_plot_row(value: f32, min: f32, span: f32, height: usize) -> usize {
    if height <= 1 {
        return 0;
    }
    let normalized = ((value - min) / span).clamp(0.0, 1.0);
    ((1.0 - normalized) * (height.saturating_sub(1) as f32)).round() as usize
}

fn compact_precip_band(values: &[f32], width: usize) -> String {
    const BLOCKS: [char; 8] = ['·', '▁', '▂', '▃', '▄', '▅', '▆', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }

    let max = values.iter().copied().fold(0.0, f32::max).max(1.0);
    (0..width)
        .map(|col| {
            let idx = sample_index(col, width, values.len());
            let value = values[idx].clamp(0.0, max);
            let level = ((value / max) * (BLOCKS.len() as f32 - 1.0)).round() as usize;
            BLOCKS[level.min(BLOCKS.len() - 1)]
        })
        .collect()
}
