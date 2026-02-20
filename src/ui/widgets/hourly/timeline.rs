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
    value.map_or_else(|| "--".to_string(), |v| format!("{v:.0} km/h"))
}

fn format_chart_percent(value: Option<f32>) -> String {
    value.map_or_else(|| "--".to_string(), |v| format!("{v:.0}%"))
}

struct TimelineSeries {
    temps: Vec<Option<f32>>,
    precips: Vec<f32>,
    times: Vec<chrono::NaiveDateTime>,
}

fn timeline_series(slice: &[&HourlyForecast], units: Units) -> TimelineSeries {
    TimelineSeries {
        temps: slice
            .iter()
            .map(|h| h.temperature_2m_c.map(|t| convert_temp(t, units)))
            .collect::<Vec<_>>(),
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
) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Temp  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline_optional(&series.temps, cols),
                Style::default().fg(theme.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled("Tick  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                hour_tick_line(&series.times, cols),
                Style::default().fg(theme.popup_muted_text),
            ),
        ]),
        Line::from(vec![
            Span::styled("Rain  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                barline(&series.precips, cols),
                Style::default().fg(theme.info),
            ),
        ]),
    ];

    if height >= 4 {
        lines.push(Line::from(vec![
            Span::styled("Hour  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                hour_label_line(&series.times, cols),
                Style::default().fg(theme.text),
            ),
        ]));
    }
    lines.truncate(height as usize);
    lines
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

fn barline(values: &[f32], width: usize) -> String {
    const BLOCKS: [char; 8] = ['·', '▁', '▂', '▃', '▄', '▅', '▆', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let max = values.iter().copied().fold(0.0f32, f32::max).max(0.001);
    (0..width)
        .map(|col| {
            let idx = sample_index(col, width, values.len());
            let value = values[idx].max(0.0);
            let level = ((value / max) * (BLOCKS.len() as f32 - 1.0)).round() as usize;
            BLOCKS[level.min(BLOCKS.len() - 1)]
        })
        .collect()
}

fn hour_tick_line(times: &[chrono::NaiveDateTime], width: usize) -> String {
    if times.is_empty() || width == 0 {
        return String::new();
    }
    let mut out = vec![' '; width];
    let mut previous_day = times[0].ordinal();
    for (col, slot) in out.iter_mut().enumerate().take(width) {
        let idx = sample_index(col, width, times.len());
        let hour = times[idx].hour();
        let day = times[idx].ordinal();
        *slot = if day != previous_day {
            previous_day = day;
            '|'
        } else if hour.is_multiple_of(6) {
            '┆'
        } else {
            ' '
        };
    }
    out.into_iter().collect()
}

fn hour_label_line(times: &[chrono::NaiveDateTime], width: usize) -> String {
    if times.is_empty() || width == 0 {
        return String::new();
    }
    let mut out = vec![' '; width];
    for col in 0..width {
        let idx = sample_index(col, width, times.len());
        let hour = times[idx].hour();
        if hour.is_multiple_of(6) {
            let label = format!("{hour:02}");
            let start = col.saturating_sub(1).min(width.saturating_sub(label.len()));
            for (offset, ch) in label.chars().enumerate() {
                if start + offset < width {
                    out[start + offset] = ch;
                }
            }
        }
    }
    out.into_iter().collect()
}
