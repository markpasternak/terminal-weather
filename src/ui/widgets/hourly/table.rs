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
    hour.wind_speed_10m.map_or_else(
        || "-- ".to_string(),
        |w| format!("{:>3}", crate::domain::weather::round_wind_speed(w)),
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
