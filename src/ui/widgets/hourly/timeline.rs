use super::*;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct TimelineStats {
    pub(super) wind_avg: Option<f32>,
    pub(super) precip_prob_max: Option<f32>,
    pub(super) cloud_avg: Option<f32>,
}

pub(super) fn render_temp_precip_timeline(
    frame: &mut Frame,
    area: Rect,
    slice: &[&HourlyForecast],
    theme: Theme,
    units: Units,
) -> TimelineStats {
    if area.height == 0 || area.width < 12 {
        return TimelineStats::default();
    }

    let cols = area.width.saturating_sub(7) as usize;
    let series = timeline_series(slice, units);
    let lines = timeline_lines(&series, cols, area.height, theme);
    frame.render_widget(Paragraph::new(lines), area);
    timeline_stats(slice)
}

pub(super) fn render_chart_metrics(
    frame: &mut Frame,
    area: Rect,
    stats: TimelineStats,
    theme: Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let wind = format_chart_wind(stats.wind_avg);
    let precip = format_chart_percent(stats.precip_prob_max);
    let cloud = format_chart_percent(stats.cloud_avg);

    let line = Line::from(vec![
        Span::styled("Wind ", Style::default().fg(theme.muted_text)),
        Span::styled(wind, Style::default().fg(theme.success)),
        Span::raw("  "),
        Span::styled("P% max ", Style::default().fg(theme.muted_text)),
        Span::styled(precip, Style::default().fg(theme.warning)),
        Span::raw("  "),
        Span::styled("Cloud ", Style::default().fg(theme.muted_text)),
        Span::styled(cloud, Style::default().fg(theme.info)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn format_chart_wind(value: Option<f32>) -> String {
    value.map_or_else(
        || "--".to_string(),
        |v| format!("{} m/s", crate::domain::weather::round_wind_speed(v)),
    )
}

fn format_chart_percent(value: Option<f32>) -> String {
    value.map_or_else(|| "--".to_string(), |v| format!("{v:.0}%"))
}

struct TimelineSeries {
    temps: Vec<Option<f32>>,
    temp_unit: &'static str,
    precips: Vec<f32>,
    times: Vec<chrono::NaiveDateTime>,
}

type TimelineLine = Line<'static>;

fn timeline_series(slice: &[&HourlyForecast], units: Units) -> TimelineSeries {
    TimelineSeries {
        temps: slice
            .iter()
            .map(|h| h.temperature_2m_c.map(|t| convert_temp(t, units)))
            .collect::<Vec<_>>(),
        temp_unit: units.symbol(),
        precips: slice
            .iter()
            .map(|h| h.precipitation_mm.unwrap_or(0.0).max(0.0))
            .collect::<Vec<_>>(),
        times: slice.iter().map(|h| h.time).collect::<Vec<_>>(),
    }
}

fn timeline_lines(
    series: &TimelineSeries,
    cols: usize,
    height: u16,
    theme: Theme,
) -> Vec<TimelineLine> {
    let mut lines = vec![
        temp_timeline_line(series, cols, theme),
        rain_timeline_line(series, cols, theme),
    ];
    if height >= 3 {
        lines.push(time_axis_timeline_line(series, cols, theme));
    }
    lines.truncate(height as usize);
    lines
}

fn timeline_row(
    label: &'static str,
    value: String,
    label_color: Color,
    value_color: Color,
) -> TimelineLine {
    Line::from(vec![
        Span::styled(label, Style::default().fg(label_color)),
        Span::styled(value, Style::default().fg(value_color)),
    ])
}

fn temp_timeline_line(series: &TimelineSeries, cols: usize, theme: Theme) -> TimelineLine {
    const SPACING: usize = 2;
    let range = temp_range_label(&series.temps, series.temp_unit);
    let chart_width = cols.saturating_sub(range.len() + SPACING);
    let chart = sparkline_optional(&series.temps, chart_width.max(1));
    let value = if chart_width == 0 {
        truncate_to_width(&range, cols)
    } else {
        format!("{chart}  {range}")
    };

    timeline_row("Temp  ", value, theme.muted_text, theme.accent)
}

fn rain_timeline_line(series: &TimelineSeries, cols: usize, theme: Theme) -> TimelineLine {
    const PRECIP_CAP_MM: f32 = 2.0;
    const SPACING: usize = 2;
    let range = precip_range_label(&series.precips);
    let chart_width = cols.saturating_sub(range.len() + SPACING);
    let chart = barline_capped(&series.precips, chart_width.max(1), PRECIP_CAP_MM);
    let value = if chart_width == 0 {
        truncate_to_width(&range, cols)
    } else {
        format!("{chart}  {range}")
    };

    timeline_row("Precip", value, theme.muted_text, theme.info)
}

fn time_axis_timeline_line(series: &TimelineSeries, cols: usize, theme: Theme) -> TimelineLine {
    timeline_row(
        "Time  ",
        time_axis_line(&series.times, cols),
        theme.muted_text,
        theme.text,
    )
}

fn timeline_stats(slice: &[&HourlyForecast]) -> TimelineStats {
    TimelineStats {
        wind_avg: average(slice.iter().filter_map(|h| h.wind_speed_10m)),
        precip_prob_max: slice
            .iter()
            .filter_map(|h| h.precipitation_probability)
            .max_by(f32::total_cmp),
        cloud_avg: average(slice.iter().filter_map(|h| h.cloud_cover)),
    }
}

fn average(values: impl Iterator<Item = f32>) -> Option<f32> {
    let mut total = 0.0f32;
    let mut count = 0u32;
    for value in values {
        total += value;
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some(total / count as f32)
    }
}

fn sample_index(col: usize, width: usize, sample_len: usize) -> usize {
    if width <= 1 || sample_len <= 1 {
        return 0;
    }
    col.saturating_mul(sample_len.saturating_sub(1)) / width.saturating_sub(1)
}

fn sample_column(sample_idx: usize, width: usize, sample_len: usize) -> usize {
    if width <= 1 || sample_len <= 1 {
        return 0;
    }
    sample_idx.saturating_mul(width.saturating_sub(1)) / sample_len.saturating_sub(1)
}

fn sparkline_optional(values: &[Option<f32>], width: usize) -> String {
    const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }

    let concrete = values.iter().flatten().copied().collect::<Vec<_>>();
    if concrete.is_empty() {
        return "·".repeat(width);
    }
    let min = concrete.iter().copied().fold(f32::INFINITY, f32::min);
    let max = concrete.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max - min).max(0.001);

    (0..width)
        .map(|col| {
            let idx = sample_index(col, width, values.len());
            let value = values[idx];
            match value {
                Some(v) => {
                    let level = (((v - min) / span) * (BLOCKS.len() as f32 - 1.0)).round() as usize;
                    BLOCKS[level.min(BLOCKS.len() - 1)]
                }
                None => '·',
            }
        })
        .collect()
}

fn barline_capped(values: &[f32], width: usize, cap_mm_per_hour: f32) -> String {
    const BLOCKS: [char; 8] = ['·', '▁', '▂', '▃', '▄', '▅', '▆', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let max = cap_mm_per_hour.max(0.001);
    (0..width)
        .map(|col| {
            let idx = sample_index(col, width, values.len());
            let value = values[idx].max(0.0).min(max);
            let level = ((value / max) * (BLOCKS.len() as f32 - 1.0)).round() as usize;
            BLOCKS[level.min(BLOCKS.len() - 1)]
        })
        .collect()
}

fn temp_range_label(values: &[Option<f32>], unit: &str) -> String {
    let concrete = values.iter().flatten().copied().collect::<Vec<_>>();
    if concrete.is_empty() {
        return format!("--..--{unit}");
    }
    let min = concrete.iter().copied().fold(f32::INFINITY, f32::min);
    let max = concrete.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    format!("{}..{}{unit}", round_temp(min), round_temp(max))
}

fn precip_range_label(values: &[f32]) -> String {
    if values.is_empty() {
        return "--..--mm/h".to_string();
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    format!("{min:.1}..{max:.1}mm/h")
}

fn truncate_to_width(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn time_axis_line(times: &[chrono::NaiveDateTime], width: usize) -> String {
    if times.is_empty() || width == 0 {
        return String::new();
    }
    let mut out = vec![' '; width];
    let mut previous_day = times[0].ordinal();
    let mut previous_col = None;

    for (idx, time) in times.iter().enumerate() {
        let Some(col) = unique_axis_column(idx, width, times.len(), &mut previous_col) else {
            continue;
        };

        let hour = time.hour();
        let day = time.ordinal();
        let day_changed = day != previous_day;
        out[col] = axis_marker(hour, day_changed);
        if day_changed {
            previous_day = day;
        }

        if hour.is_multiple_of(6) {
            let label = format!("{hour:02}");
            let start = axis_label_start(day_changed, col, width, label.len());
            write_axis_label(&mut out, start, &label);
        }
    }
    out.into_iter().collect()
}

fn unique_axis_column(
    sample_idx: usize,
    width: usize,
    sample_len: usize,
    previous_col: &mut Option<usize>,
) -> Option<usize> {
    let col = sample_column(sample_idx, width, sample_len);
    if *previous_col == Some(col) {
        return None;
    }
    *previous_col = Some(col);
    Some(col)
}

fn axis_marker(hour: u32, day_changed: bool) -> char {
    if day_changed {
        '|'
    } else if hour.is_multiple_of(6) {
        '┆'
    } else if hour.is_multiple_of(3) {
        '·'
    } else {
        ' '
    }
}

fn axis_label_start(day_changed: bool, col: usize, width: usize, label_len: usize) -> usize {
    if day_changed && col + 1 < width {
        col.saturating_add(1).min(width.saturating_sub(label_len))
    } else {
        col.saturating_sub(1).min(width.saturating_sub(label_len))
    }
}

fn write_axis_label(out: &mut [char], start: usize, label: &str) {
    for (offset, ch) in label.chars().enumerate() {
        if start + offset < out.len() {
            out[start + offset] = ch;
        }
    }
}

#[cfg(test)]
mod tests;
