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
    let temp_unit = match units {
        Units::Celsius => "C",
        Units::Fahrenheit => "F",
    };

    TimelineSeries {
        temps: slice
            .iter()
            .map(|h| h.temperature_2m_c.map(|t| convert_temp(t, units)))
            .collect::<Vec<_>>(),
        temp_unit,
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
        let col = sample_column(idx, width, times.len());
        if previous_col == Some(col) {
            continue;
        }
        previous_col = Some(col);

        let hour = time.hour();
        let day = time.ordinal();
        let day_changed = day != previous_day;
        out[col] = if day_changed {
            previous_day = day;
            '|'
        } else if hour.is_multiple_of(6) {
            '┆'
        } else if hour.is_multiple_of(3) {
            '·'
        } else {
            ' '
        };

        if hour.is_multiple_of(6) {
            let label = format!("{hour:02}");
            let start = if day_changed && col + 1 < width {
                col.saturating_add(1).min(width.saturating_sub(label.len()))
            } else {
                col.saturating_sub(1).min(width.saturating_sub(label.len()))
            };
            for (offset, ch) in label.chars().enumerate() {
                if start + offset < width {
                    out[start + offset] = ch;
                }
            }
        }
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cli::ThemeArg,
        domain::weather::WeatherCategory,
        ui::theme::{ColorCapability, theme_for},
    };
    use chrono::{NaiveDate, NaiveDateTime};

    #[test]
    fn timeline_lines_use_single_time_axis_with_anchors() {
        let theme = theme_for(
            WeatherCategory::Cloudy,
            true,
            ColorCapability::Basic16,
            ThemeArg::Aurora,
        );
        let series = TimelineSeries {
            temps: vec![Some(-2.0), Some(4.0), Some(1.0)],
            temp_unit: "C",
            precips: vec![0.0, 0.8, 1.6],
            times: vec![dt(2026, 2, 22, 0), dt(2026, 2, 22, 6), dt(2026, 2, 22, 12)],
        };

        let lines = timeline_lines(&series, 56, 6, theme);
        assert_eq!(lines.len(), 3);

        let text = lines.iter().map(line_text).collect::<Vec<_>>();
        assert!(text[0].starts_with("Temp  "));
        assert!(text[1].starts_with("Precip"));
        assert!(text[2].starts_with("Time  "));
        assert!(text[0].contains('C'));
        assert!(text[0].contains(".."));
        assert!(text[1].contains("mm/h"));
        assert!(!text.iter().any(|line| line.starts_with("Tick  ")));
        assert!(!text.iter().any(|line| line.starts_with("Hour  ")));
        assert!(!text.iter().any(|line| line.starts_with("Shift ")));
    }

    #[test]
    fn time_axis_labels_are_not_repeated_per_column() {
        let times = (0..13)
            .map(|hour| dt(2026, 2, 22, hour))
            .collect::<Vec<_>>();
        let axis = time_axis_line(&times, 80);

        assert_eq!(axis.matches("00").count(), 1);
        assert_eq!(axis.matches("06").count(), 1);
        assert_eq!(axis.matches("12").count(), 1);
    }

    #[test]
    fn time_axis_marks_day_boundaries() {
        let times = (18..31)
            .map(|h| {
                let day = if h < 24 { 22 } else { 23 };
                let hour = h % 24;
                dt(2026, 2, day, hour)
            })
            .collect::<Vec<_>>();
        let axis = time_axis_line(&times, 64);
        assert!(axis.contains('|'));
    }

    #[test]
    fn capped_precip_barline_uses_absolute_intensity_scale() {
        let bars = barline_capped(&[0.2, 1.0, 2.5], 3, 2.0);
        let chars = bars.chars().collect::<Vec<_>>();

        assert_eq!(chars.len(), 3);
        assert_ne!(chars[1], '█');
        assert_eq!(chars[2], '█');
    }

    fn dt(year: i32, month: u32, day: u32, hour: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("valid date")
            .and_hms_opt(hour, 0, 0)
            .expect("valid time")
    }

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>()
            .join("")
    }
}
