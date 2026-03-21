use super::*;

pub(super) fn render_table_mode(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    slice: &[&HourlyForecast],
    theme: Theme,
) {
    let panel_style = Style::default().fg(theme.text).bg(theme.surface);
    let label_width = table_label_width(area.width);
    let offset = state.hourly_offset;
    let cursor_in_view = cursor_in_view(state.hourly_cursor, offset, slice.len());
    let rows = build_table_rows(
        area.height,
        slice,
        offset,
        cursor_in_view,
        bundle,
        state,
        theme,
    );
    let table = build_hourly_table(area.width, slice, rows, panel_style, label_width);

    let insight =
        crate::domain::weather::derive_nowcast_insight(bundle, state.units, &state.refresh_meta);
    let (table_area, detail_area, window_area) = split_table_and_detail_areas(area);
    frame.render_widget(table, table_area);
    let detail_text = cursor_detail_text(
        state,
        bundle,
        slice,
        offset,
        cursor_in_view,
        &insight.action_text,
    );
    render_cursor_detail_line(frame, detail_area, detail_text, theme);
    render_window_summary_line(frame, window_area, &insight.next_6h_summary, theme);
}

const fn table_label_width(width: u16) -> u16 {
    if width >= 92 { 7 } else { 6 }
}

const fn cursor_in_view(hourly_cursor: usize, offset: usize, visible_len: usize) -> Option<usize> {
    if hourly_cursor >= offset && hourly_cursor < offset + visible_len {
        Some(hourly_cursor - offset)
    } else {
        None
    }
}

fn build_table_rows(
    area_height: u16,
    slice: &[&HourlyForecast],
    offset: usize,
    cursor_in_view: Option<usize>,
    bundle: &ForecastBundle,
    state: &AppState,
    theme: Theme,
) -> Vec<Row<'static>> {
    let mut rows = vec![
        build_time_row(slice, offset, cursor_in_view, theme),
        build_weather_row(slice, bundle, state, theme),
        build_temp_row(slice, state.units, cursor_in_view, theme),
    ];
    if let Some(date_row) = build_optional_date_row(slice, offset, theme) {
        rows.insert(1, date_row);
    }
    for (min_height, label, color, formatter) in metric_row_specs(theme) {
        if area_height >= min_height {
            rows.push(build_metric_row(label, slice, color, formatter, theme));
        }
    }
    rows
}

fn build_hourly_table(
    area_width: u16,
    slice: &[&HourlyForecast],
    rows: Vec<Row<'static>>,
    panel_style: Style,
    label_width: u16,
) -> Table<'static> {
    let column_spacing = if area_width >= 140 {
        2
    } else {
        u16::from(area_width >= 104)
    };
    let mut widths = vec![Constraint::Length(label_width)];
    widths.extend(vec![
        Constraint::Ratio(1, slice.len().max(1) as u32);
        slice.len()
    ]);
    Table::new(rows, widths)
        .column_spacing(column_spacing)
        .style(panel_style)
}

fn split_table_and_detail_areas(area: Rect) -> (Rect, Option<Rect>, Option<Rect>) {
    if area.height < 8 {
        return (area, None, None);
    }
    if area.height >= 9 && area.width >= 96 {
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);
        return (chunks[0], Some(chunks[1]), Some(chunks[2]));
    }
    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);
    (chunks[0], Some(chunks[1]), None)
}

fn render_cursor_detail_line(
    frame: &mut Frame,
    area: Option<Rect>,
    detail_text: Option<String>,
    theme: Theme,
) {
    let Some(area) = area else {
        return;
    };
    let Some(detail) = detail_text else {
        return;
    };
    frame.render_widget(
        Paragraph::new(detail).style(Style::default().fg(theme.muted_text)),
        area,
    );
}

fn cursor_detail_text(
    state: &AppState,
    bundle: &ForecastBundle,
    slice: &[&HourlyForecast],
    offset: usize,
    cursor_in_view: Option<usize>,
    action_text: &str,
) -> Option<String> {
    let (cursor_idx, hour) = selected_cursor_hour(slice, cursor_in_view)?;
    let temp = cursor_temp_text(hour, state.units);
    let gust = cursor_gust_text(hour);
    let precip = cursor_precip_probability_text(hour);
    let mm = cursor_precip_mm_text(hour);
    let decision = strip_action_prefix(action_text);
    // OPTIMIZATION: Avoid chrono formatting overhead in rendering loop.
    let why = format!(
        "focus {} {} {:02}:{:02} is {}",
        offset + cursor_idx,
        crate::ui::widgets::daily::summary::utils::short_weekday(hour.time.date()),
        hour.time.hour(),
        hour.time.minute(),
        weather_label_for_time(
            hour.weather_code.unwrap_or(bundle.current.weather_code),
            hour.is_day.unwrap_or(bundle.current.is_day),
        ),
    );
    let details = format!("temp {temp} · P% {precip} · P {mm}mm · gust {gust}");
    Some(format_decision_line(decision, &why, &details))
}

fn selected_cursor_hour<'a>(
    slice: &'a [&'a HourlyForecast],
    cursor_in_view: Option<usize>,
) -> Option<(usize, &'a HourlyForecast)> {
    let cursor_idx = cursor_in_view.or_else(|| (!slice.is_empty()).then_some(0))?;
    slice
        .get(cursor_idx)
        .copied()
        .map(|hour| (cursor_idx, hour))
}

fn cursor_temp_text(hour: &HourlyForecast, units: Units) -> String {
    hour.temperature_2m_c.map_or_else(
        || "--".to_string(),
        |value| format!("{}°", round_temp(convert_temp(value, units))),
    )
}

fn cursor_gust_text(hour: &HourlyForecast) -> String {
    wind_reference(hour).map_or_else(
        || "--".to_string(),
        |value| format!("{}m/s", crate::domain::weather::round_wind_speed(value)),
    )
}

fn cursor_precip_probability_text(hour: &HourlyForecast) -> String {
    hour.precipitation_probability
        .map_or_else(|| "--".to_string(), |value| format!("{value:.0}%"))
}

fn cursor_precip_mm_text(hour: &HourlyForecast) -> String {
    hour.precipitation_mm.map_or_else(
        || "--.-".to_string(),
        |value| format!("{:.1}", value.max(0.0)),
    )
}

fn render_window_summary_line(
    frame: &mut Frame,
    area: Option<Rect>,
    next_6h_summary: &str,
    theme: Theme,
) {
    let Some(area) = area else {
        return;
    };
    let summary = format_window_details(next_6h_summary);
    frame.render_widget(
        Paragraph::new(summary).style(Style::default().fg(theme.info)),
        area,
    );
}

fn strip_action_prefix(action_text: &str) -> &str {
    action_text
        .strip_prefix("Now action: ")
        .unwrap_or(action_text)
}

fn format_decision_line(decision: &str, why: &str, details: &str) -> String {
    format!("Do: {decision} · Why: {why} · Details: {details}")
}

fn format_window_details(next_6h_summary: &str) -> String {
    let summary = next_6h_summary
        .strip_prefix("Next 6h: ")
        .unwrap_or(next_6h_summary);
    format!("Details: next 6h {summary}")
}

fn wind_reference(hour: &HourlyForecast) -> Option<f32> {
    hour.wind_gusts_10m.or(hour.wind_speed_10m)
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
        // OPTIMIZATION: Avoid chrono formatting overhead in rendering loop.
        let label = if is_now {
            "Now".to_string()
        } else {
            format!("{:02}:{:02}", h.time.hour(), h.time.minute())
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

pub(super) fn build_optional_date_row(
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
            Cell::from("│").style(Style::default().fg(theme.muted_text))
        } else {
            last_shown_date = Some(date);
            // OPTIMIZATION: Avoid chrono formatting overhead in rendering loop.
            Cell::from(format!(
                "▌{} {:02}",
                crate::ui::widgets::daily::summary::utils::short_weekday(date),
                date.day()
            ))
            .style(
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

pub(super) fn metric_row_specs(
    theme: Theme,
) -> [(u16, &'static str, Color, HourlyMetricFormatter); 8] {
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
    hour.wind_gusts_10m.map_or_else(
        || "-- ".to_string(),
        |g| format!("{:>3}", crate::domain::weather::round_wind_speed(g)),
    )
}

fn format_visibility_metric(hour: &HourlyForecast) -> String {
    hour.visibility_m.map_or_else(
        || "-- ".to_string(),
        |v| format!("{:>3}", (v / 1000.0).round() as i32),
    )
}

fn format_cloud_metric(hour: &HourlyForecast) -> String {
    format_percent_metric(hour.cloud_cover)
}

fn format_pressure_metric(hour: &HourlyForecast) -> String {
    hour.pressure_msl_hpa
        .map_or_else(|| " -- ".to_string(), |p| format!("{p:>4.0}"))
}

fn format_humidity_metric(hour: &HourlyForecast) -> String {
    format_percent_metric(hour.relative_humidity_2m)
}

fn format_precip_probability_metric(hour: &HourlyForecast) -> String {
    format_percent_metric(hour.precipitation_probability)
}

fn format_wind_metric(hour: &HourlyForecast) -> String {
    hour.wind_speed_10m.map_or_else(
        || "-- ".to_string(),
        |w| format!("{:>3}", crate::domain::weather::round_wind_speed(w)),
    )
}

fn format_percent_metric(value: Option<f32>) -> String {
    value.map_or_else(
        || "-- ".to_string(),
        |p| format!("{:>3}%", p.round() as i32),
    )
}

pub(super) fn sanitize_precip_mm(value: f32) -> f32 {
    let non_negative = value.max(0.0);
    if non_negative == 0.0 {
        0.0
    } else {
        non_negative
    }
}

#[cfg(test)]
mod tests {
    use super::{format_decision_line, format_window_details, strip_action_prefix};

    #[test]
    fn strip_action_prefix_removes_now_action_label() {
        assert_eq!(
            strip_action_prefix("Now action: layer up for cold (2°C)"),
            "layer up for cold (2°C)"
        );
    }

    #[test]
    fn format_decision_line_orders_sections_consistently() {
        let line = format_decision_line("layer up", "focus Sun 23:00 is overcast", "temp -2°");
        assert_eq!(
            line,
            "Do: layer up · Why: focus Sun 23:00 is overcast · Details: temp -2°"
        );
    }

    #[test]
    fn format_window_details_strips_existing_prefix() {
        let details = format_window_details("Next 6h: P 1.2mm · Pmax 70%");
        assert_eq!(details, "Details: next 6h P 1.2mm · Pmax 70%");
    }
}
