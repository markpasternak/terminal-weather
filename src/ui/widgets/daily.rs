use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::{
    app::state::AppState,
    cli::Cli,
    domain::weather::{
        DailyForecast, ForecastBundle, Units, WeatherCategory, convert_temp, round_temp,
        weather_code_to_category, weather_icon,
    },
    ui::theme::{detect_color_capability, icon_color, temp_color, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    let capability = detect_color_capability(state.color_mode);

    let Some(bundle) = &state.weather else {
        let theme = theme_for(
            WeatherCategory::Unknown,
            true,
            capability,
            state.settings.theme,
        );
        let panel_style = Style::default().fg(theme.text).bg(theme.surface_alt);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("7-Day")
            .style(panel_style)
            .border_style(Style::default().fg(theme.border).bg(theme.surface_alt));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        render_loading_daily(
            frame,
            inner,
            state.frame_tick,
            panel_style,
            theme.accent,
            theme.muted_text,
        );
        return;
    };

    let layout = DailyLayout::for_area(area);
    let title = "7-Day Forecast";

    let theme = theme_for(
        weather_code_to_category(bundle.current.weather_code),
        bundle.current.is_day,
        capability,
        state.settings.theme,
    );
    let panel_style = Style::default().fg(theme.text).bg(theme.surface_alt);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(panel_style)
        .border_style(Style::default().fg(theme.border).bg(theme.surface_alt));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let global_min = bundle
        .daily
        .iter()
        .filter_map(|d| d.temperature_min_c)
        .fold(f32::INFINITY, f32::min);
    let global_max = bundle
        .daily
        .iter()
        .filter_map(|d| d.temperature_max_c)
        .fold(f32::NEG_INFINITY, f32::max);

    let max_rows = layout.max_rows(inner.height);
    if max_rows == 0 {
        return;
    }

    let rows =
        bundle
            .daily
            .iter()
            .take(max_rows)
            .enumerate()
            .map(|(idx, day)| {
                let is_today = idx == 0;
                let min_c = day.temperature_min_c.unwrap_or(global_min);
                let max_c = day.temperature_max_c.unwrap_or(global_max);

                let min_label = format!("{}°", round_temp(convert_temp(min_c, state.units)));
                let max_label = format!("{}°", round_temp(convert_temp(max_c, state.units)));

                let mut row_cells = vec![
                    Cell::from(day.date.format("%a").to_string())
                        .style(Style::default().fg(theme.text)),
                ];
                if layout.show_icon {
                    let code = day.weather_code.unwrap_or(3);
                    row_cells.push(
                        Cell::from(weather_icon(code, state.settings.icon_mode, true)).style(
                            Style::default().fg(icon_color(&theme, weather_code_to_category(code))),
                        ),
                    );
                }
                row_cells.push(Cell::from(min_label).style(
                    Style::default().fg(temp_color(&theme, convert_temp(min_c, state.units))),
                ));

                if layout.show_bar {
                    let (start, end) =
                        bar_bounds(min_c, max_c, global_min, global_max, layout.bar_width);
                    let clamped_start = start.min(layout.bar_width);
                    let clamped_end = end.min(layout.bar_width.saturating_sub(1));
                    let before = "·".repeat(clamped_start);
                    let fill_len = clamped_end.saturating_sub(clamped_start).saturating_add(1);
                    let fill = "█".repeat(fill_len);
                    let after =
                        "·".repeat(layout.bar_width.saturating_sub(clamped_start + fill_len));
                    let bar = Line::from(vec![
                        Span::styled(before, Style::default().fg(theme.range_track)),
                        Span::styled(fill, Style::default().fg(theme.accent)),
                        Span::styled(after, Style::default().fg(theme.range_track)),
                    ]);
                    row_cells.push(Cell::from(bar));
                }

                row_cells.push(Cell::from(max_label).style(
                    Style::default().fg(temp_color(&theme, convert_temp(max_c, state.units))),
                ));
                if layout.show_precip_col {
                    let precip = day
                        .precipitation_sum_mm
                        .map_or_else(|| "--.-".to_string(), |v| format!("{v:>4.1}"));
                    row_cells.push(Cell::from(precip).style(Style::default().fg(theme.info)));
                }
                if layout.show_gust_col {
                    let gust = day
                        .wind_gusts_10m_max
                        .map_or_else(|| "-- ".to_string(), |v| format!("{:>3}", v.round() as i32));
                    row_cells.push(Cell::from(gust).style(Style::default().fg(theme.warning)));
                }

                let row = Row::new(row_cells);

                if is_today {
                    row.style(Style::default().add_modifier(Modifier::BOLD))
                } else {
                    row
                }
            })
            .collect::<Vec<_>>();

    let mut widths = vec![Constraint::Length(4)];
    if layout.show_icon {
        widths.push(Constraint::Length(3));
    }
    widths.push(Constraint::Length(5));
    if layout.show_bar {
        widths.push(Constraint::Length(layout.bar_width as u16));
    }
    widths.push(Constraint::Length(5));
    if layout.show_precip_col {
        widths.push(Constraint::Length(5));
    }
    if layout.show_gust_col {
        widths.push(Constraint::Length(4));
    }

    let row_count = rows.len() as u16;
    let header_rows = u16::from(layout.show_header);
    let table_height = row_count.saturating_add(header_rows);

    let mut table = Table::new(rows, widths)
        .column_spacing(layout.column_spacing)
        .style(panel_style);
    if layout.show_header {
        let mut header_cells = vec!["Day"];
        if layout.show_icon {
            header_cells.push("Wx");
        }
        header_cells.push("Low");
        if layout.show_bar {
            header_cells.push("Range");
        }
        header_cells.push("High");
        if layout.show_precip_col {
            header_cells.push("Pmm");
        }
        if layout.show_gust_col {
            header_cells.push("Gst");
        }
        table = table.header(Row::new(header_cells).style(Style::default().fg(theme.muted_text)));
    }

    let (table_area, summary_slot) = if inner.height > table_height.saturating_add(2) {
        let summary_y = inner.y.saturating_add(table_height);
        let summary_height = inner.bottom().saturating_sub(summary_y);
        (
            Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width,
                height: table_height,
            },
            if summary_height > 0 {
                Some(Rect {
                    x: inner.x,
                    y: summary_y,
                    width: inner.width,
                    height: summary_height,
                })
            } else {
                None
            },
        )
    } else {
        (inner, None)
    };
    frame.render_widget(table, table_area);

    if let Some(area) = summary_slot {
        render_week_summary(frame, area, bundle, state.units, theme);
    }
}

fn render_loading_daily(
    frame: &mut Frame,
    area: Rect,
    frame_tick: u64,
    panel_style: Style,
    accent: Color,
    muted: Color,
) {
    if area.height == 0 || area.width < 12 {
        return;
    }
    let rows = usize::from(area.height).min(6);
    let width = usize::from(area.width.saturating_sub(10)).clamp(8, 28);
    let phase = (frame_tick as usize) % width;

    let body = (0..rows)
        .map(|idx| {
            let mut bar = vec!['·'; width];
            let head = (phase + idx * 2) % width;
            bar[head] = '█';
            if head > 0 {
                bar[head - 1] = '▓';
            }

            Row::new(vec![
                Cell::from(format!("D{}", idx + 1)).style(Style::default().fg(muted)),
                Cell::from(bar.into_iter().collect::<String>()).style(Style::default().fg(accent)),
                Cell::from("--°").style(Style::default().fg(muted)),
            ])
        })
        .collect::<Vec<_>>();

    let table = Table::new(
        body,
        [
            Constraint::Length(4),
            Constraint::Length(width as u16),
            Constraint::Length(4),
        ],
    )
    .column_spacing(1)
    .style(panel_style);

    frame.render_widget(table, area);
}

fn render_week_summary(
    frame: &mut Frame,
    area: Rect,
    bundle: &ForecastBundle,
    units: Units,
    theme: crate::ui::theme::Theme,
) {
    if area.width < 20 || area.height == 0 || bundle.daily.is_empty() {
        return;
    }

    let summary = summarize_week(bundle, units);
    let mut lines = week_summary_header_lines(&summary, theme);
    if area.width >= 64 {
        append_week_meta_line(&mut lines, bundle, theme);
    }
    let mut remaining_rows = (area.height as usize).saturating_sub(lines.len());
    append_week_profiles(&mut lines, &mut remaining_rows, area, &summary, theme);
    append_day_cues(&mut lines, &mut remaining_rows, bundle, theme);
    append_compact_profiles(&mut lines, remaining_rows, area, &summary, theme);
    frame.render_widget(Paragraph::new(lines), area);
}

#[derive(Debug, Default)]
struct WeekSummaryData {
    precip_total: f32,
    rain_total: f32,
    snow_total: f32,
    avg_daylight: String,
    avg_sun: String,
    breeziest_txt: String,
    wettest_txt: String,
    precip_hours_avg: String,
    uv_peak: String,
    week_thermal: String,
    highs: Vec<f32>,
    precip: Vec<f32>,
    gusts: Vec<f32>,
}

#[derive(Debug, Default)]
struct WeekAccumulator {
    precip_total: f32,
    rain_total: f32,
    snow_total: f32,
    daylight_total: f32,
    sunshine_total: f32,
    daylight_count: usize,
    sunshine_count: usize,
    precipitation_hours_total: f32,
    precipitation_hours_count: usize,
    breeziest: Option<(String, f32)>,
    wettest: Option<(String, f32)>,
    strongest_uv: Option<(String, f32)>,
    week_min_temp_c: Option<f32>,
    week_max_temp_c: Option<f32>,
}

impl WeekAccumulator {
    fn ingest(&mut self, day: &DailyForecast) {
        if let Some(v) = day.precipitation_sum_mm {
            self.precip_total += v.max(0.0);
            let tag = day.date.format("%a").to_string();
            if self.wettest.as_ref().is_none_or(|(_, best)| v > *best) {
                self.wettest = Some((tag, v));
            }
        }
        if let Some(v) = day.rain_sum_mm {
            self.rain_total += v.max(0.0);
        }
        if let Some(v) = day.snowfall_sum_cm {
            self.snow_total += v.max(0.0);
        }
        if let Some(v) = day.daylight_duration_s {
            self.daylight_total += v.max(0.0);
            self.daylight_count += 1;
        }
        if let Some(v) = day.sunshine_duration_s {
            self.sunshine_total += v.max(0.0);
            self.sunshine_count += 1;
        }
        if let Some(v) = day.precipitation_hours {
            self.precipitation_hours_total += v.max(0.0);
            self.precipitation_hours_count += 1;
        }
        if let Some(v) = day.wind_gusts_10m_max {
            let tag = day.date.format("%a").to_string();
            if self.breeziest.as_ref().is_none_or(|(_, best)| v > *best) {
                self.breeziest = Some((tag, v));
            }
        }
        if let Some(v) = day.uv_index_max {
            let tag = day.date.format("%a").to_string();
            if self.strongest_uv.as_ref().is_none_or(|(_, best)| v > *best) {
                self.strongest_uv = Some((tag, v));
            }
        }
        if let Some(v) = day.temperature_min_c {
            self.week_min_temp_c = Some(self.week_min_temp_c.map_or(v, |current| current.min(v)));
        }
        if let Some(v) = day.temperature_max_c {
            self.week_max_temp_c = Some(self.week_max_temp_c.map_or(v, |current| current.max(v)));
        }
    }

    fn finish(self, units: Units, daily: &[DailyForecast]) -> WeekSummaryData {
        let wettest_txt = self
            .wettest
            .map_or_else(|| "--".to_string(), |(day, mm)| format!("{day} {mm:.1}mm"));
        let breeziest_txt = self.breeziest.map_or_else(
            || "--".to_string(),
            |(day, gust)| format!("{day} {}km/h", gust.round() as i32),
        );
        let avg_daylight = if self.daylight_count > 0 {
            format_duration_hm(self.daylight_total / self.daylight_count as f32)
        } else {
            "--:--".to_string()
        };
        let avg_sun = if self.sunshine_count > 0 {
            format_duration_hm(self.sunshine_total / self.sunshine_count as f32)
        } else {
            "--:--".to_string()
        };
        let precip_hours_avg = if self.precipitation_hours_count > 0 {
            format!(
                "{:.1}h/day",
                self.precipitation_hours_total / self.precipitation_hours_count as f32
            )
        } else {
            "--".to_string()
        };
        let uv_peak = self
            .strongest_uv
            .map_or_else(|| "--".to_string(), |(day, uv)| format!("{day} {uv:.1}"));
        let week_thermal = match (self.week_min_temp_c, self.week_max_temp_c) {
            (Some(low), Some(high)) => {
                let low = round_temp(convert_temp(low, units));
                let high = round_temp(convert_temp(high, units));
                format!("{low}°..{high}°")
            }
            _ => "--".to_string(),
        };
        let highs = daily
            .iter()
            .filter_map(|d| d.temperature_max_c)
            .map(|t| convert_temp(t, units))
            .collect::<Vec<_>>();
        let precip = daily
            .iter()
            .map(|d| d.precipitation_sum_mm.unwrap_or(0.0))
            .collect::<Vec<_>>();
        let gusts = daily
            .iter()
            .map(|d| d.wind_gusts_10m_max.unwrap_or(0.0))
            .collect::<Vec<_>>();

        WeekSummaryData {
            precip_total: self.precip_total,
            rain_total: self.rain_total,
            snow_total: self.snow_total,
            avg_daylight,
            avg_sun,
            breeziest_txt,
            wettest_txt,
            precip_hours_avg,
            uv_peak,
            week_thermal,
            highs,
            precip,
            gusts,
        }
    }
}

fn summarize_week(bundle: &ForecastBundle, units: Units) -> WeekSummaryData {
    let mut accumulator = WeekAccumulator::default();
    for day in &bundle.daily {
        accumulator.ingest(day);
    }
    accumulator.finish(units, &bundle.daily)
}

fn week_summary_header_lines(
    summary: &WeekSummaryData,
    theme: crate::ui::theme::Theme,
) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Totals ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("P {:.1}mm", summary.precip_total),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Rain ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{:.1}mm", summary.rain_total),
                Style::default().fg(theme.info),
            ),
            Span::raw("  "),
            Span::styled("Snow ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{:.1}cm", summary.snow_total),
                Style::default().fg(theme.temp_cold),
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg daylight ", Style::default().fg(theme.muted_text)),
            Span::styled(
                summary.avg_daylight.clone(),
                Style::default().fg(theme.warning),
            ),
            Span::raw("  "),
            Span::styled("Avg sun ", Style::default().fg(theme.muted_text)),
            Span::styled(summary.avg_sun.clone(), Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("Breeziest ", Style::default().fg(theme.muted_text)),
            Span::styled(
                summary.breeziest_txt.clone(),
                Style::default().fg(theme.warning),
            ),
            Span::raw("  "),
            Span::styled("Wettest ", Style::default().fg(theme.muted_text)),
            Span::styled(summary.wettest_txt.clone(), Style::default().fg(theme.info)),
        ]),
        Line::from(vec![
            Span::styled("Avg precip hrs ", Style::default().fg(theme.muted_text)),
            Span::styled(
                summary.precip_hours_avg.clone(),
                Style::default().fg(theme.info),
            ),
            Span::raw("  "),
            Span::styled("Peak UV ", Style::default().fg(theme.muted_text)),
            Span::styled(summary.uv_peak.clone(), Style::default().fg(theme.warning)),
            Span::raw("  "),
            Span::styled("Week span ", Style::default().fg(theme.muted_text)),
            Span::styled(
                summary.week_thermal.clone(),
                Style::default().fg(theme.accent),
            ),
        ]),
    ]
}

fn append_week_meta_line(
    lines: &mut Vec<Line<'static>>,
    bundle: &ForecastBundle,
    theme: crate::ui::theme::Theme,
) {
    let tz = bundle
        .location
        .timezone
        .as_deref()
        .unwrap_or("--")
        .to_string();
    let sunrise = bundle
        .daily
        .first()
        .and_then(|d| d.sunrise)
        .map_or_else(|| "--:--".to_string(), |v| v.format("%H:%M").to_string());
    let sunset = bundle
        .daily
        .first()
        .and_then(|d| d.sunset)
        .map_or_else(|| "--:--".to_string(), |v| v.format("%H:%M").to_string());
    let dawn = bundle.daily.first().and_then(|d| d.sunrise).map_or_else(
        || "--:--".to_string(),
        |v| {
            (v - chrono::Duration::minutes(30))
                .format("%H:%M")
                .to_string()
        },
    );
    let dusk = bundle.daily.first().and_then(|d| d.sunset).map_or_else(
        || "--:--".to_string(),
        |v| {
            (v + chrono::Duration::minutes(30))
                .format("%H:%M")
                .to_string()
        },
    );

    lines.push(Line::from(vec![
        Span::styled("Meta ", Style::default().fg(theme.muted_text)),
        Span::styled(format!("TZ {tz}"), Style::default().fg(theme.text)),
        Span::raw("  "),
        Span::styled(
            format!("Dawn {dawn} Sun {sunrise} Set {sunset} Dusk {dusk}"),
            Style::default().fg(theme.info),
        ),
        Span::raw("  "),
        Span::styled(
            bundle.location.name.clone(),
            Style::default().fg(theme.accent),
        ),
    ]));
}

fn append_week_profiles(
    lines: &mut Vec<Line<'static>>,
    remaining_rows: &mut usize,
    area: Rect,
    summary: &WeekSummaryData,
    theme: crate::ui::theme::Theme,
) {
    if area.width < 72 || *remaining_rows < 4 {
        return;
    }

    lines.push(Line::from(Span::styled(
        "Week profiles",
        Style::default()
            .fg(theme.muted_text)
            .add_modifier(Modifier::BOLD),
    )));
    *remaining_rows = remaining_rows.saturating_sub(1);

    let profile_width = area.width.saturating_sub(28).clamp(10, 56) as usize;
    if *remaining_rows > 0 && !summary.highs.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Temp arc    ", Style::default().fg(theme.muted_text)),
            Span::styled(
                profile_bar(&summary.highs, profile_width),
                Style::default().fg(theme.accent),
            ),
            Span::raw(" "),
            Span::styled(
                summary.week_thermal.clone(),
                Style::default().fg(theme.accent),
            ),
        ]));
        *remaining_rows = remaining_rows.saturating_sub(1);
    }
    if *remaining_rows > 0 && !summary.precip.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Precip lane ", Style::default().fg(theme.muted_text)),
            Span::styled(
                profile_bar(&summary.precip, profile_width),
                Style::default().fg(theme.info),
            ),
            Span::raw(" "),
            Span::styled(
                format!("{:.1}mm", summary.precip_total),
                Style::default().fg(theme.info),
            ),
        ]));
        *remaining_rows = remaining_rows.saturating_sub(1);
    }
    if *remaining_rows > 0 && !summary.gusts.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Wind lane   ", Style::default().fg(theme.muted_text)),
            Span::styled(
                profile_bar(&summary.gusts, profile_width),
                Style::default().fg(theme.warning),
            ),
            Span::raw(" "),
            Span::styled(
                summary.breeziest_txt.clone(),
                Style::default().fg(theme.warning),
            ),
        ]));
        *remaining_rows = remaining_rows.saturating_sub(1);
    }
}

fn append_day_cues(
    lines: &mut Vec<Line<'static>>,
    remaining_rows: &mut usize,
    bundle: &ForecastBundle,
    theme: crate::ui::theme::Theme,
) {
    if *remaining_rows < 2 {
        return;
    }

    lines.push(Line::from(Span::styled(
        "Day cues",
        Style::default()
            .fg(theme.muted_text)
            .add_modifier(Modifier::BOLD),
    )));
    *remaining_rows = remaining_rows.saturating_sub(1);

    let cue_rows = (*remaining_rows).min(bundle.daily.len());
    for day in bundle.daily.iter().take(cue_rows) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>3} ", day.date.format("%a")),
                Style::default().fg(theme.muted_text),
            ),
            Span::styled(day_cue(day), Style::default().fg(theme.text)),
        ]));
    }
    *remaining_rows = remaining_rows.saturating_sub(cue_rows);
}

fn append_compact_profiles(
    lines: &mut Vec<Line<'static>>,
    remaining_rows: usize,
    area: Rect,
    summary: &WeekSummaryData,
    theme: crate::ui::theme::Theme,
) {
    if remaining_rows == 0 || area.width < 32 {
        return;
    }

    let mut slots_left = remaining_rows;
    let profile_width = area.width.saturating_sub(18).clamp(8, 40) as usize;
    if slots_left > 0 && !summary.highs.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Temp profile ", Style::default().fg(theme.muted_text)),
            Span::styled(
                profile_bar(&summary.highs, profile_width),
                Style::default().fg(theme.accent),
            ),
        ]));
        slots_left = slots_left.saturating_sub(1);
    }
    if slots_left > 0 && !summary.precip.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Precip lane  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                profile_bar(&summary.precip, profile_width),
                Style::default().fg(theme.info),
            ),
        ]));
        slots_left = slots_left.saturating_sub(1);
    }
    if slots_left > 0 && !summary.gusts.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Wind lane    ", Style::default().fg(theme.muted_text)),
            Span::styled(
                profile_bar(&summary.gusts, profile_width),
                Style::default().fg(theme.warning),
            ),
        ]));
    }
}

fn day_cue(day: &DailyForecast) -> String {
    let precip = day.precipitation_sum_mm.unwrap_or(0.0);
    let snow = day.snowfall_sum_cm.unwrap_or(0.0);
    let gust = day.wind_gusts_10m_max.unwrap_or(0.0);
    let sun_ratio = match (day.sunshine_duration_s, day.daylight_duration_s) {
        (Some(sun), Some(daylight)) if daylight > 0.0 => Some((sun / daylight).clamp(0.0, 1.0)),
        _ => None,
    };

    let mut parts = Vec::new();
    if snow >= 1.0 {
        parts.push(format!("snow {snow:.1}cm"));
    } else if precip >= 6.0 {
        parts.push(format!("wet {precip:.1}mm"));
    } else if precip >= 1.0 {
        parts.push(format!("light rain {precip:.1}mm"));
    } else {
        parts.push("mostly dry".to_string());
    }

    if gust >= 45.0 {
        parts.push(format!("gusty {}km/h", gust.round() as i32));
    } else if gust >= 30.0 {
        parts.push(format!("breezy {}km/h", gust.round() as i32));
    }

    if let Some(ratio) = sun_ratio {
        if ratio >= 0.65 {
            parts.push("bright".to_string());
        } else if ratio <= 0.25 {
            parts.push("dim".to_string());
        }
    }

    parts.join(", ")
}

#[derive(Debug, Clone, Copy)]
struct DailyLayout {
    show_icon: bool,
    show_bar: bool,
    show_header: bool,
    show_precip_col: bool,
    show_gust_col: bool,
    bar_width: usize,
    column_spacing: u16,
}

impl DailyLayout {
    fn for_area(area: Rect) -> Self {
        let inner_width = area.width.saturating_sub(2) as usize;

        // Width tiers:
        // - wide: icon + bar + header
        // - medium: no icon, still show range bar + header
        // - narrow: low/high only
        if inner_width >= 112 {
            let bar_width = inner_width
                .saturating_sub(4 + 3 + 5 + 5 + 5 + 4 + 10)
                .clamp(18, 48);
            Self {
                show_icon: true,
                show_bar: true,
                show_header: true,
                show_precip_col: true,
                show_gust_col: true,
                bar_width,
                column_spacing: 2,
            }
        } else if inner_width >= 86 {
            let bar_width = inner_width
                .saturating_sub(4 + 3 + 5 + 5 + 5 + 8)
                .clamp(14, 34);
            Self {
                show_icon: true,
                show_bar: true,
                show_header: true,
                show_precip_col: true,
                show_gust_col: false,
                bar_width,
                column_spacing: 1,
            }
        } else if inner_width >= 56 {
            let bar_width = inner_width.saturating_sub(4 + 3 + 5 + 5 + 6).clamp(10, 24);
            Self {
                show_icon: true,
                show_bar: true,
                show_header: true,
                show_precip_col: false,
                show_gust_col: false,
                bar_width,
                column_spacing: 1,
            }
        } else if inner_width >= 36 {
            let bar_width = inner_width.saturating_sub(4 + 5 + 5 + 3).clamp(6, 18);
            Self {
                show_icon: false,
                show_bar: true,
                show_header: true,
                show_precip_col: false,
                show_gust_col: false,
                bar_width,
                column_spacing: 1,
            }
        } else {
            Self {
                show_icon: false,
                show_bar: false,
                show_header: false,
                show_precip_col: false,
                show_gust_col: false,
                bar_width: 0,
                column_spacing: 1,
            }
        }
    }

    fn max_rows(self, inner_height: u16) -> usize {
        let reserved = if self.show_header { 1 } else { 0 };
        usize::from(inner_height.saturating_sub(reserved)).min(7)
    }
}

fn format_duration_hm(seconds: f32) -> String {
    let total_minutes = (seconds.max(0.0) / 60.0).round() as i64;
    let h = total_minutes / 60;
    let m = total_minutes % 60;
    format!("{h:02}:{m:02}")
}

fn profile_bar(values: &[f32], width: usize) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max - min).max(0.001);
    (0..width)
        .map(|idx| {
            let src = (idx * values.len() / width).min(values.len().saturating_sub(1));
            let norm = ((values[src] - min) / span).clamp(0.0, 1.0);
            BARS[(norm * (BARS.len() - 1) as f32).round() as usize]
        })
        .collect()
}

pub fn bar_bounds(
    min: f32,
    max: f32,
    global_min: f32,
    global_max: f32,
    width: usize,
) -> (usize, usize) {
    let span = (global_max - global_min).max(1.0);
    let start = (((min - global_min) / span) * width as f32)
        .round()
        .clamp(0.0, width as f32) as usize;
    let end = (((max - global_min) / span) * width as f32)
        .round()
        .clamp(0.0, width as f32) as usize;
    (start, end.max(start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::weather::{CurrentConditions, ForecastBundle, Location};
    use chrono::{NaiveDate, Utc};

    #[test]
    fn range_bounds_are_clamped() {
        let (start, end) = bar_bounds(-50.0, 80.0, -10.0, 40.0, 12);
        assert!(start <= 12);
        assert!(end <= 12);
        assert!(start <= end);
    }

    #[test]
    fn daily_layout_changes_by_width() {
        let wide = DailyLayout::for_area(Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 10,
        });
        assert!(wide.show_icon);
        assert!(wide.show_bar);
        assert!(wide.show_header);

        let medium = DailyLayout::for_area(Rect {
            x: 0,
            y: 0,
            width: 44,
            height: 10,
        });
        assert!(!medium.show_icon);
        assert!(medium.show_bar);
        assert!(medium.show_header);

        let narrow = DailyLayout::for_area(Rect {
            x: 0,
            y: 0,
            width: 32,
            height: 10,
        });
        assert!(!narrow.show_icon);
        assert!(!narrow.show_bar);
        assert!(!narrow.show_header);
    }

    #[test]
    fn summarize_week_aggregates_three_day_dataset() {
        let daily = vec![
            sample_day(DayInput {
                date: (2026, 2, 20),
                precip_mm: 3.0,
                rain_mm: 2.0,
                snow_cm: 0.0,
                gust_kmh: 30.0,
                uv: 4.5,
                min_c: -2.0,
                max_c: 6.0,
                daylight_s: 36000.0,
                sunshine_s: 18000.0,
                precip_hours: 2.0,
            }),
            sample_day(DayInput {
                date: (2026, 2, 21),
                precip_mm: 5.0,
                rain_mm: 1.5,
                snow_cm: 1.2,
                gust_kmh: 52.0,
                uv: 7.0,
                min_c: -4.0,
                max_c: 9.0,
                daylight_s: 43200.0,
                sunshine_s: 21600.0,
                precip_hours: 4.0,
            }),
            sample_day(DayInput {
                date: (2026, 2, 22),
                precip_mm: 2.0,
                rain_mm: 1.0,
                snow_cm: 0.0,
                gust_kmh: 22.0,
                uv: 5.2,
                min_c: 0.0,
                max_c: 4.0,
                daylight_s: 32400.0,
                sunshine_s: 10800.0,
                precip_hours: 1.0,
            }),
        ];
        let bundle = sample_bundle(daily.clone());

        let summary = summarize_week(&bundle, Units::Celsius);

        assert!((summary.precip_total - 10.0).abs() < f32::EPSILON);
        assert!((summary.rain_total - 4.5).abs() < f32::EPSILON);
        assert!((summary.snow_total - 1.2).abs() < f32::EPSILON);
        assert_eq!(summary.avg_daylight, "10:20");
        assert_eq!(summary.avg_sun, "04:40");
        assert_eq!(summary.precip_hours_avg, "2.3h/day");
        assert_eq!(
            summary.wettest_txt,
            format!("{} 5.0mm", daily[1].date.format("%a"))
        );
        assert_eq!(
            summary.breeziest_txt,
            format!("{} 52km/h", daily[1].date.format("%a"))
        );
        assert_eq!(
            summary.uv_peak,
            format!("{} 7.0", daily[1].date.format("%a"))
        );
        assert_eq!(summary.week_thermal, "-4°..9°");
        assert_eq!(summary.highs, vec![6.0, 9.0, 4.0]);
        assert_eq!(summary.precip, vec![3.0, 5.0, 2.0]);
        assert_eq!(summary.gusts, vec![30.0, 52.0, 22.0]);
    }

    #[derive(Clone, Copy)]
    struct DayInput {
        date: (i32, u32, u32),
        precip_mm: f32,
        rain_mm: f32,
        snow_cm: f32,
        gust_kmh: f32,
        uv: f32,
        min_c: f32,
        max_c: f32,
        daylight_s: f32,
        sunshine_s: f32,
        precip_hours: f32,
    }

    fn sample_day(input: DayInput) -> DailyForecast {
        DailyForecast {
            date: NaiveDate::from_ymd_opt(input.date.0, input.date.1, input.date.2)
                .expect("valid date"),
            weather_code: Some(3),
            temperature_max_c: Some(input.max_c),
            temperature_min_c: Some(input.min_c),
            sunrise: None,
            sunset: None,
            uv_index_max: Some(input.uv),
            precipitation_probability_max: Some(70.0),
            precipitation_sum_mm: Some(input.precip_mm),
            rain_sum_mm: Some(input.rain_mm),
            snowfall_sum_cm: Some(input.snow_cm),
            precipitation_hours: Some(input.precip_hours),
            wind_gusts_10m_max: Some(input.gust_kmh),
            daylight_duration_s: Some(input.daylight_s),
            sunshine_duration_s: Some(input.sunshine_s),
        }
    }

    fn sample_bundle(daily: Vec<DailyForecast>) -> ForecastBundle {
        ForecastBundle {
            location: Location::from_coords(59.3293, 18.0686),
            current: CurrentConditions {
                temperature_2m_c: 2.0,
                relative_humidity_2m: 75.0,
                apparent_temperature_c: 0.0,
                dew_point_2m_c: -1.0,
                weather_code: 3,
                precipitation_mm: 0.0,
                cloud_cover: 60.0,
                pressure_msl_hpa: 1010.0,
                visibility_m: 9000.0,
                wind_speed_10m: 12.0,
                wind_gusts_10m: 18.0,
                wind_direction_10m: 180.0,
                is_day: true,
                high_today_c: Some(6.0),
                low_today_c: Some(-2.0),
            },
            hourly: Vec::new(),
            daily,
            fetched_at: Utc::now(),
        }
    }
}
