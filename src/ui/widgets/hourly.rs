#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use chrono::{Datelike, Timelike};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::{
    app::state::AppState,
    cli::Cli,
    domain::weather::{
        Daypart, DaypartSummary, ForecastBundle, HourlyForecast, HourlyViewMode, Units,
        convert_temp, round_temp, summarize_dayparts, weather_code_to_category, weather_icon,
        weather_label_for_time,
    },
    ui::layout::visible_hour_count,
    ui::theme::{Theme, detect_color_capability, icon_color, temp_color, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    let capability = detect_color_capability(state.color_mode);

    let Some(bundle) = &state.weather else {
        let theme = theme_for(
            crate::domain::weather::WeatherCategory::Unknown,
            true,
            capability,
            state.settings.theme,
        );
        let panel_style = Style::default().fg(theme.text).bg(theme.surface);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Hourly")
            .style(panel_style)
            .border_style(Style::default().fg(theme.border).bg(theme.surface));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        render_loading_placeholder(
            frame,
            inner,
            state.frame_tick,
            panel_style,
            theme.accent,
            theme.muted_text,
        );
        return;
    };

    let theme = theme_for(
        weather_code_to_category(bundle.current.weather_code),
        bundle.current.is_day,
        capability,
        state.settings.theme,
    );
    let panel_style = Style::default().fg(theme.text).bg(theme.surface);
    let effective_mode = effective_hourly_mode(state.hourly_view_mode, area);

    let show = visible_hour_count(area.width);
    let offset = state
        .hourly_offset
        .min(bundle.hourly.len().saturating_sub(1));
    let slice = bundle
        .hourly
        .iter()
        .skip(offset)
        .take(show)
        .collect::<Vec<_>>();

    let mode_label = match effective_mode {
        HourlyViewMode::Table => "Table",
        HourlyViewMode::Hybrid => "Hybrid",
        HourlyViewMode::Chart => "Chart",
    };
    let title = if let (Some(first), Some(last)) = (slice.first(), slice.last()) {
        let first_date = first.time.format("%a %d %b");
        let last_date = last.time.format("%a %d %b");
        if first.time.date() == last.time.date() {
            format!("Hourly · {mode_label} · {first_date}")
        } else {
            format!("Hourly · {mode_label} · {first_date} → {last_date}")
        }
    } else {
        format!("Hourly · {mode_label}")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(panel_style)
        .border_style(Style::default().fg(theme.border).bg(theme.surface));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if slice.is_empty() {
        render_loading_placeholder(
            frame,
            inner,
            state.frame_tick,
            panel_style,
            theme.accent,
            theme.muted_text,
        );
        return;
    }

    match effective_mode {
        HourlyViewMode::Table => render_table_mode(frame, inner, state, bundle, &slice, theme),
        HourlyViewMode::Hybrid => {
            if !render_hybrid_mode(frame, inner, state, bundle, &slice, theme) {
                render_table_mode(frame, inner, state, bundle, &slice, theme);
            }
        }
        HourlyViewMode::Chart => {
            if !render_chart_mode(frame, inner, state, bundle, &slice, theme) {
                render_table_mode(frame, inner, state, bundle, &slice, theme);
            }
        }
    }
}

fn effective_hourly_mode(requested: HourlyViewMode, area: Rect) -> HourlyViewMode {
    let inner_width = area.width.saturating_sub(2);
    let inner_height = area.height.saturating_sub(2);
    if inner_width < 70 || inner_height < 5 {
        return HourlyViewMode::Table;
    }

    match requested {
        HourlyViewMode::Hybrid if inner_height >= 6 => HourlyViewMode::Hybrid,
        HourlyViewMode::Chart if inner_height >= 8 => HourlyViewMode::Chart,
        _ => HourlyViewMode::Table,
    }
}

fn render_table_mode(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    slice: &[&HourlyForecast],
    theme: Theme,
) {
    let panel_style = Style::default().fg(theme.text).bg(theme.surface);
    let label_width = if area.width >= 92 { 7 } else { 6 };
    let offset = state.hourly_offset;
    let cursor_in_view =
        if state.hourly_cursor >= offset && state.hourly_cursor < offset + slice.len() {
            Some(state.hourly_cursor - offset)
        } else {
            None
        };

    let mut rows = vec![
        build_time_row(slice, offset, cursor_in_view, theme),
        build_weather_row(slice, bundle, state, theme),
        build_temp_row(slice, state.units, cursor_in_view, theme),
    ];

    if let Some(date_row) = build_optional_date_row(slice, offset, theme) {
        rows.insert(1, date_row);
    }

    for (min_height, label, color, formatter) in metric_row_specs(theme) {
        if area.height >= min_height {
            rows.push(build_metric_row(label, slice, color, formatter, theme));
        }
    }

    let column_spacing = if area.width >= 140 {
        2
    } else {
        u16::from(area.width >= 104)
    };
    let mut widths = vec![Constraint::Length(label_width)];
    widths.extend(vec![
        Constraint::Ratio(1, slice.len().max(1) as u32);
        slice.len()
    ]);
    let table = Table::new(rows, widths)
        .column_spacing(column_spacing)
        .style(panel_style);
    frame.render_widget(table, area);
}

type HourlyMetricFormatter = fn(&HourlyForecast) -> String;

fn build_time_row(
    slice: &[&HourlyForecast],
    offset: usize,
    cursor_in_view: Option<usize>,
    theme: Theme,
) -> Row<'static> {
    let mut cells = vec![Cell::from("Time").style(Style::default().fg(theme.muted_text))];
    cells.extend(slice.iter().enumerate().map(|(idx, h)| {
        let is_now = idx + offset == 0;
        let is_cursor = cursor_in_view == Some(idx);
        let label = if is_now {
            "Now".to_string()
        } else {
            h.time.format("%H:%M").to_string()
        };
        let style = if is_cursor {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else if is_now {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted_text)
        };
        Cell::from(label).style(style)
    }));
    Row::new(cells)
}

fn build_weather_row(
    slice: &[&HourlyForecast],
    bundle: &ForecastBundle,
    state: &AppState,
    theme: Theme,
) -> Row<'static> {
    let mut cells = vec![Cell::from("Wx").style(Style::default().fg(theme.muted_text))];
    cells.extend(slice.iter().map(|h| {
        let code = h.weather_code.unwrap_or(bundle.current.weather_code);
        let is_day = h.is_day.unwrap_or(bundle.current.is_day);
        Cell::from(weather_icon(code, state.settings.icon_mode, is_day))
            .style(Style::default().fg(icon_color(&theme, weather_code_to_category(code))))
    }));
    Row::new(cells)
}

fn build_temp_row(
    slice: &[&HourlyForecast],
    units: Units,
    cursor_in_view: Option<usize>,
    theme: Theme,
) -> Row<'static> {
    let mut cells = vec![Cell::from("Temp").style(Style::default().fg(theme.muted_text))];
    cells.extend(slice.iter().enumerate().map(|(idx, h)| {
        let temp = h.temperature_2m_c.map(|t| convert_temp(t, units));
        let is_cursor = cursor_in_view == Some(idx);
        let mut style = Style::default()
            .fg(temp.map_or(theme.muted_text, |t| temp_color(&theme, t)))
            .add_modifier(Modifier::BOLD);
        if is_cursor {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        Cell::from(temp.map_or_else(|| "--".to_string(), |t| format!("{}°", round_temp(t))))
            .style(style)
    }));
    Row::new(cells)
}

fn build_optional_date_row(
    slice: &[&HourlyForecast],
    offset: usize,
    theme: Theme,
) -> Option<Row<'static>> {
    let has_date_change = slice
        .windows(2)
        .any(|w| w[0].time.date() != w[1].time.date());
    if !has_date_change && offset == 0 {
        return None;
    }

    let mut cells = vec![Cell::from("Date").style(Style::default().fg(theme.muted_text))];
    let mut last_shown_date: Option<chrono::NaiveDate> = None;
    cells.extend(slice.iter().map(|h| {
        let date = h.time.date();
        if last_shown_date == Some(date) {
            Cell::from("·").style(Style::default().fg(theme.muted_text))
        } else {
            last_shown_date = Some(date);
            Cell::from(date.format("%a %d").to_string()).style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
        }
    }));
    Some(Row::new(cells))
}

fn build_metric_row(
    label: &'static str,
    slice: &[&HourlyForecast],
    color: Color,
    formatter: HourlyMetricFormatter,
    theme: Theme,
) -> Row<'static> {
    let mut cells = vec![Cell::from(label).style(Style::default().fg(theme.muted_text))];
    cells.extend(
        slice
            .iter()
            .map(|h| Cell::from(formatter(h)).style(Style::default().fg(color))),
    );
    Row::new(cells)
}

fn metric_row_specs(theme: Theme) -> [(u16, &'static str, Color, HourlyMetricFormatter); 8] {
    [
        (5, "P mm", theme.info, format_precip_mm_metric),
        (6, "Gust", theme.warning, format_gust_metric),
        (7, "Vis", theme.success, format_visibility_metric),
        (8, "Cloud", theme.landmark_neutral, format_cloud_metric),
        (9, "Press", theme.info, format_pressure_metric),
        (10, "RH", theme.info, format_humidity_metric),
        (11, "P%", theme.warning, format_precip_probability_metric),
        (12, "Wind", theme.success, format_wind_metric),
    ]
}

fn format_precip_mm_metric(hour: &HourlyForecast) -> String {
    hour.precipitation_mm.map_or_else(
        || "--.-".to_string(),
        |p| format!("{:>4.1}", sanitize_precip_mm(p)),
    )
}

fn format_gust_metric(hour: &HourlyForecast) -> String {
    hour.wind_gusts_10m
        .map_or_else(|| "-- ".to_string(), |g| format!("{:>3}", g.round() as i32))
}

fn format_visibility_metric(hour: &HourlyForecast) -> String {
    hour.visibility_m.map_or_else(
        || "-- ".to_string(),
        |v| format!("{:>3}", (v / 1000.0).round() as i32),
    )
}

fn format_cloud_metric(hour: &HourlyForecast) -> String {
    hour.cloud_cover.map_or_else(
        || "-- ".to_string(),
        |c| format!("{:>3}%", c.round() as i32),
    )
}

fn format_pressure_metric(hour: &HourlyForecast) -> String {
    hour.pressure_msl_hpa
        .map_or_else(|| " -- ".to_string(), |p| format!("{p:>4.0}"))
}

fn format_humidity_metric(hour: &HourlyForecast) -> String {
    hour.relative_humidity_2m.map_or_else(
        || "-- ".to_string(),
        |rh| format!("{:>3}%", rh.round() as i32),
    )
}

fn format_precip_probability_metric(hour: &HourlyForecast) -> String {
    hour.precipitation_probability.map_or_else(
        || "-- ".to_string(),
        |p| format!("{:>3}%", p.round() as i32),
    )
}

fn format_wind_metric(hour: &HourlyForecast) -> String {
    hour.wind_speed_10m
        .map_or_else(|| "-- ".to_string(), |w| format!("{:>3}", w.round() as i32))
}

fn render_hybrid_mode(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    slice: &[&HourlyForecast],
    theme: Theme,
) -> bool {
    if area.height < 7 {
        return false;
    }

    let chunks = if area.height >= 12 {
        Layout::vertical([Constraint::Length(5), Constraint::Min(4)]).split(area)
    } else if area.height >= 10 {
        Layout::vertical([Constraint::Length(4), Constraint::Min(3)]).split(area)
    } else {
        Layout::vertical([Constraint::Length(3), Constraint::Min(2)]).split(area)
    };

    let _ = render_temp_precip_timeline(frame, chunks[0], slice, theme, state.units);

    let day_count = if area.width >= 100 { 2 } else { 1 };
    render_daypart_cards(frame, chunks[1], bundle, state, theme, day_count)
}

fn render_chart_mode(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    slice: &[&HourlyForecast],
    theme: Theme,
) -> bool {
    let _ = bundle;
    if area.height < 8 {
        return false;
    }

    let chunks = Layout::vertical([Constraint::Min(6), Constraint::Length(2)]).split(area);
    let timeline_stats = render_temp_precip_timeline(frame, chunks[0], slice, theme, state.units);
    render_chart_metrics(frame, chunks[1], timeline_stats, theme);
    true
}

fn render_chart_metrics(frame: &mut Frame, area: Rect, stats: TimelineStats, theme: Theme) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let wind = stats
        .wind_avg
        .map_or_else(|| "--".to_string(), |v| format!("{v:.0} km/h"));
    let precip = stats
        .precip_prob_max
        .map_or_else(|| "--".to_string(), |v| format!("{v:.0}%"));
    let cloud = stats
        .cloud_avg
        .map_or_else(|| "--".to_string(), |v| format!("{v:.0}%"));

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

fn render_date_chip(frame: &mut Frame, area: Rect, label: &str, theme: Theme) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let chip = Paragraph::new(format!("[ {label} ]"))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(chip, area);
}

fn render_daypart_cards(
    frame: &mut Frame,
    area: Rect,
    bundle: &ForecastBundle,
    state: &AppState,
    theme: Theme,
    day_count: usize,
) -> bool {
    if area.height < 3 || area.width < 24 {
        return false;
    }

    let summaries = summarize_dayparts(&bundle.hourly, bundle.current.weather_code, day_count);
    if summaries.is_empty() {
        return false;
    }

    let mut dates = summaries.iter().map(|s| s.date).collect::<Vec<_>>();
    dates.sort_unstable();
    dates.dedup();
    if dates.is_empty() {
        return false;
    }

    let sections =
        Layout::vertical(vec![Constraint::Ratio(1, dates.len() as u32); dates.len()]).split(area);

    for (idx, date) in dates.into_iter().enumerate() {
        let section = sections[idx];
        if section.height < 3 {
            return false;
        }

        let parts = collect_parts_for_date(&summaries, date);
        if parts.len() != Daypart::all().len() {
            continue;
        }

        let day_rows = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(section);
        render_date_chip(
            frame,
            day_rows[0],
            &date.format("%a %d %b").to_string(),
            theme,
        );

        let card_area = day_rows[1];
        let (show_secondary, show_wind, show_vis) = daypart_visibility(card_area.height);
        let rows = build_daypart_rows(&parts, state, theme, show_secondary, show_wind, show_vis);

        let table = Table::new(rows, vec![Constraint::Ratio(1, 4); 4]).column_spacing(1);
        frame.render_widget(table, card_area);
    }

    true
}

fn daypart_visibility(height: u16) -> (bool, bool, bool) {
    match height {
        0..=3 => (false, false, false),
        4 => (false, false, true),
        5 => (false, true, true),
        _ => (true, true, true),
    }
}

fn collect_parts_for_date(
    summaries: &[DaypartSummary],
    date: chrono::NaiveDate,
) -> Vec<DaypartSummary> {
    Daypart::all()
        .iter()
        .filter_map(|part| {
            summaries
                .iter()
                .find(|s| s.date == date && s.daypart == *part)
                .cloned()
        })
        .collect()
}

fn build_daypart_rows(
    parts: &[DaypartSummary],
    state: &AppState,
    theme: Theme,
    show_secondary: bool,
    show_wind: bool,
    show_vis: bool,
) -> Vec<Row<'static>> {
    let mut rows = vec![
        build_daypart_header_row(theme),
        build_daypart_primary_row(parts, state, theme),
        build_daypart_precip_row(parts, theme),
    ];
    if show_secondary {
        rows.push(build_daypart_secondary_row(parts, theme));
    }
    if show_wind {
        rows.push(build_daypart_wind_row(parts, theme));
    }
    if show_vis {
        rows.push(build_daypart_visibility_row(parts, theme));
    }
    rows
}

fn build_daypart_header_row(theme: Theme) -> Row<'static> {
    Row::new(
        Daypart::all()
            .iter()
            .map(|part| Cell::from(part.label()).style(Style::default().fg(theme.muted_text)))
            .collect::<Vec<_>>(),
    )
    .style(Style::default().add_modifier(Modifier::BOLD))
}

fn build_daypart_primary_row(
    parts: &[DaypartSummary],
    state: &AppState,
    theme: Theme,
) -> Row<'static> {
    Row::new(
        parts
            .iter()
            .map(|summary| {
                Cell::from(format!(
                    "{} {}",
                    weather_icon(
                        summary.weather_code,
                        state.settings.icon_mode,
                        !matches!(summary.daypart, Daypart::Night)
                    ),
                    temp_range(summary, state.units),
                ))
                .style(Style::default().fg(theme.text))
            })
            .collect::<Vec<_>>(),
    )
}

fn build_daypart_precip_row(parts: &[DaypartSummary], theme: Theme) -> Row<'static> {
    Row::new(
        parts
            .iter()
            .map(|summary| {
                let prob = summary
                    .precip_probability_max
                    .map_or_else(|| "--".to_string(), |v| format!("{v:.0}%"));
                Cell::from(format!(
                    "{:.1}mm {prob}",
                    sanitize_precip_mm(summary.precip_sum_mm)
                ))
                .style(Style::default().fg(theme.info))
            })
            .collect::<Vec<_>>(),
    )
}

fn build_daypart_secondary_row(parts: &[DaypartSummary], theme: Theme) -> Row<'static> {
    Row::new(
        parts
            .iter()
            .map(|summary| {
                let label = weather_label_for_time(
                    summary.weather_code,
                    !matches!(summary.daypart, Daypart::Night),
                );
                Cell::from(truncate(label, 14)).style(Style::default().fg(theme.muted_text))
            })
            .collect::<Vec<_>>(),
    )
}

fn build_daypart_wind_row(parts: &[DaypartSummary], theme: Theme) -> Row<'static> {
    Row::new(
        parts
            .iter()
            .map(|summary| {
                let min_wind = summary
                    .wind_min_kmh
                    .map_or_else(|| "--".to_string(), |v| format!("{v:.0}"));
                let max_wind = summary
                    .wind_max_kmh
                    .map_or_else(|| "--".to_string(), |v| format!("{v:.0}"));
                Cell::from(format!("{min_wind}-{max_wind}km/h"))
                    .style(Style::default().fg(theme.warning))
            })
            .collect::<Vec<_>>(),
    )
}

fn build_daypart_visibility_row(parts: &[DaypartSummary], theme: Theme) -> Row<'static> {
    Row::new(
        parts
            .iter()
            .map(|summary| {
                let vis = summary.visibility_median_m.map_or_else(
                    || "--".to_string(),
                    |v| format!("{:.0}km", (v / 1000.0).max(0.0)),
                );
                Cell::from(vis).style(Style::default().fg(theme.success))
            })
            .collect::<Vec<_>>(),
    )
}

fn render_temp_precip_timeline(
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
    let temps = slice
        .iter()
        .map(|h| h.temperature_2m_c.map(|t| convert_temp(t, units)))
        .collect::<Vec<_>>();
    let precips = slice
        .iter()
        .map(|h| h.precipitation_mm.unwrap_or(0.0).max(0.0))
        .collect::<Vec<_>>();
    let times = slice.iter().map(|h| h.time).collect::<Vec<_>>();

    let temp_line = sparkline_optional(&temps, cols);
    let rain_line = barline(&precips, cols);
    let tick_line = hour_tick_line(&times, cols);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Temp  ", Style::default().fg(theme.muted_text)),
            Span::styled(temp_line, Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("Tick  ", Style::default().fg(theme.muted_text)),
            Span::styled(tick_line, Style::default().fg(theme.popup_muted_text)),
        ]),
        Line::from(vec![
            Span::styled("Rain  ", Style::default().fg(theme.muted_text)),
            Span::styled(rain_line, Style::default().fg(theme.info)),
        ]),
    ];

    if area.height >= 4 {
        lines.push(Line::from(vec![
            Span::styled("Hour  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                hour_label_line(&times, cols),
                Style::default().fg(theme.text),
            ),
        ]));
    }

    let max_lines = area.height as usize;
    lines.truncate(max_lines);
    frame.render_widget(Paragraph::new(lines), area);

    TimelineStats {
        wind_avg: average(slice.iter().filter_map(|h| h.wind_speed_10m)),
        precip_prob_max: slice
            .iter()
            .filter_map(|h| h.precipitation_probability)
            .max_by(f32::total_cmp),
        cloud_avg: average(slice.iter().filter_map(|h| h.cloud_cover)),
    }
}

fn temp_range(summary: &DaypartSummary, units: Units) -> String {
    match (summary.temp_min_c, summary.temp_max_c) {
        (Some(min), Some(max)) => {
            let min = round_temp(convert_temp(min, units));
            let max = round_temp(convert_temp(max, units));
            format!("{min}-{max}°")
        }
        (Some(value), None) | (None, Some(value)) => {
            format!("{}°", round_temp(convert_temp(value, units)))
        }
        _ => "--".to_string(),
    }
}

fn sanitize_precip_mm(value: f32) -> f32 {
    let non_negative = value.max(0.0);
    if non_negative == 0.0 {
        0.0
    } else {
        non_negative
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    value
        .chars()
        .take(max_chars.saturating_sub(1))
        .chain(std::iter::once('…'))
        .collect()
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

fn render_loading_placeholder(
    frame: &mut Frame,
    area: Rect,
    frame_tick: u64,
    panel_style: Style,
    accent: Color,
    muted: Color,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let slots = (usize::from(area.width) / 6).max(4);
    let mut shimmer = vec!['·'; slots];
    let idx = (frame_tick as usize) % slots;
    shimmer[idx] = '◆';
    if idx > 0 {
        shimmer[idx - 1] = '◇';
    }
    let row1 = shimmer.iter().collect::<String>();
    let row2 = (0..slots)
        .map(|i| {
            if (i + idx).is_multiple_of(3) {
                '◦'
            } else {
                ' '
            }
        })
        .collect::<String>();

    let placeholder_rows = [
        Row::new(vec![
            Cell::from("Loading timeline").style(Style::default().fg(accent)),
        ]),
        Row::new(vec![Cell::from(row1).style(Style::default().fg(muted))]),
        Row::new(vec![Cell::from(row2).style(Style::default().fg(accent))]),
    ];
    let table = Table::new(placeholder_rows, [Constraint::Min(1)])
        .column_spacing(1)
        .style(panel_style);
    frame.render_widget(table, area);
}

#[derive(Debug, Clone, Copy, Default)]
struct TimelineStats {
    wind_avg: Option<f32>,
    precip_prob_max: Option<f32>,
    cloud_avg: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::{
        build_optional_date_row, daypart_visibility, effective_hourly_mode, metric_row_specs,
        sanitize_precip_mm,
    };
    use crate::{
        cli::ThemeArg,
        domain::weather::{HourlyForecast, HourlyViewMode, WeatherCategory},
        ui::theme::{ColorCapability, theme_for},
    };
    use chrono::{NaiveDate, NaiveDateTime};
    use ratatui::layout::Rect;

    #[test]
    fn width_below_70_forces_table() {
        let area = Rect::new(0, 0, 68, 12);
        assert_eq!(
            effective_hourly_mode(HourlyViewMode::Hybrid, area),
            HourlyViewMode::Table
        );
        assert_eq!(
            effective_hourly_mode(HourlyViewMode::Chart, area),
            HourlyViewMode::Table
        );
    }

    #[test]
    fn chart_mode_requires_more_height_than_hybrid() {
        let hybrid_ok = Rect::new(0, 0, 90, 10);
        let chart_too_short = Rect::new(0, 0, 90, 9);
        assert_eq!(
            effective_hourly_mode(HourlyViewMode::Hybrid, hybrid_ok),
            HourlyViewMode::Hybrid
        );
        assert_eq!(
            effective_hourly_mode(HourlyViewMode::Chart, chart_too_short),
            HourlyViewMode::Table
        );
    }

    #[test]
    fn sanitize_precip_normalizes_negative_zero() {
        let value = sanitize_precip_mm(-0.0);
        assert_eq!(value.to_bits(), 0.0f32.to_bits());
        assert_eq!(format!("{value:.1}"), "0.0");
    }

    #[test]
    fn optional_metric_rows_keep_height_thresholds() {
        let theme = theme_for(
            WeatherCategory::Cloudy,
            true,
            ColorCapability::Basic16,
            ThemeArg::Aurora,
        );
        let count_for = |height: u16| {
            metric_row_specs(theme)
                .iter()
                .filter(|(min_height, _, _, _)| *min_height <= height)
                .count()
        };

        assert_eq!(count_for(4), 0);
        assert_eq!(count_for(5), 1);
        assert_eq!(count_for(6), 2);
        assert_eq!(count_for(9), 5);
        assert_eq!(count_for(12), 8);
    }

    #[test]
    fn date_row_inserts_for_day_change_or_offset() {
        let theme = theme_for(
            WeatherCategory::Cloudy,
            true,
            ColorCapability::Basic16,
            ThemeArg::Aurora,
        );
        let hours = [
            sample_hour(dt(2026, 2, 20, 9)),
            sample_hour(dt(2026, 2, 20, 10)),
            sample_hour(dt(2026, 2, 21, 0)),
        ];
        let same_day = vec![&hours[0], &hours[1]];
        let crosses_day = vec![&hours[1], &hours[2]];

        assert!(build_optional_date_row(&same_day, 0, theme).is_none());
        assert!(build_optional_date_row(&crosses_day, 0, theme).is_some());
        assert!(build_optional_date_row(&same_day, 1, theme).is_some());
    }

    #[test]
    fn daypart_visibility_thresholds_match_layout_contract() {
        assert_eq!(daypart_visibility(3), (false, false, false));
        assert_eq!(daypart_visibility(4), (false, false, true));
        assert_eq!(daypart_visibility(5), (false, true, true));
        assert_eq!(daypart_visibility(6), (true, true, true));
    }

    fn dt(year: i32, month: u32, day: u32, hour: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("valid date")
            .and_hms_opt(hour, 0, 0)
            .expect("valid time")
    }

    fn sample_hour(time: NaiveDateTime) -> HourlyForecast {
        HourlyForecast {
            time,
            temperature_2m_c: Some(1.0),
            weather_code: Some(3),
            is_day: Some(true),
            relative_humidity_2m: Some(75.0),
            precipitation_probability: Some(10.0),
            precipitation_mm: Some(0.2),
            rain_mm: Some(0.2),
            snowfall_cm: Some(0.0),
            wind_speed_10m: Some(12.0),
            wind_gusts_10m: Some(18.0),
            pressure_msl_hpa: Some(1009.0),
            visibility_m: Some(8000.0),
            cloud_cover: Some(60.0),
            cloud_cover_low: Some(20.0),
            cloud_cover_mid: Some(25.0),
            cloud_cover_high: Some(15.0),
        }
    }
}
