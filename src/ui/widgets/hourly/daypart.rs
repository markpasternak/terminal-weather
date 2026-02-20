use super::table::sanitize_precip_mm;
use super::*;

pub(super) fn render_date_chip(frame: &mut Frame, area: Rect, label: &str, theme: Theme) {
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

pub(super) fn render_daypart_cards(
    frame: &mut Frame,
    area: Rect,
    bundle: &ForecastBundle,
    state: &AppState,
    theme: Theme,
    day_count: usize,
) -> bool {
    if !can_render_daypart_cards(area) {
        return false;
    }
    let summaries = summarize_dayparts(&bundle.hourly, bundle.current.weather_code, day_count);
    let dates = daypart_dates(&summaries);
    if dates.is_empty() {
        return false;
    }

    let sections =
        Layout::vertical(vec![Constraint::Ratio(1, dates.len() as u32); dates.len()]).split(area);
    for (section, date) in sections.iter().zip(dates.into_iter()) {
        if !render_daypart_section(frame, *section, date, &summaries, state, theme) {
            return false;
        }
    }

    true
}

fn can_render_daypart_cards(area: Rect) -> bool {
    area.height >= 3 && area.width >= 24
}

fn daypart_dates(summaries: &[DaypartSummary]) -> Vec<chrono::NaiveDate> {
    let mut dates = summaries.iter().map(|s| s.date).collect::<Vec<_>>();
    dates.sort_unstable();
    dates.dedup();
    dates
}

fn render_daypart_section(
    frame: &mut Frame,
    section: Rect,
    date: chrono::NaiveDate,
    summaries: &[DaypartSummary],
    state: &AppState,
    theme: Theme,
) -> bool {
    if section.height < 3 {
        return false;
    }
    let parts = collect_parts_for_date(summaries, date);
    if parts.len() != Daypart::all().len() {
        return true;
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
    true
}

pub(super) fn daypart_visibility(height: u16) -> (bool, bool, bool) {
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
                let min_wind = summary.wind_min_kmh.map_or_else(
                    || "--".to_string(),
                    |v| crate::domain::weather::round_wind_speed(v).to_string(),
                );
                let max_wind = summary.wind_max_kmh.map_or_else(
                    || "--".to_string(),
                    |v| crate::domain::weather::round_wind_speed(v).to_string(),
                );
                Cell::from(format!("{min_wind}-{max_wind}m/s"))
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
