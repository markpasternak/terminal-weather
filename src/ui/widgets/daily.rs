use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{
    app::state::AppState,
    cli::Cli,
    domain::weather::{
        WeatherCategory, convert_temp, round_temp, weather_code_to_category, weather_icon,
    },
    ui::theme::{detect_color_capability, icon_color, temp_color, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    let capability = detect_color_capability();

    let Some(bundle) = &state.weather else {
        let theme = theme_for(
            WeatherCategory::Unknown,
            true,
            capability,
            state.settings.theme,
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .title("7-Day")
            .border_style(Style::default().fg(theme.border));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        render_loading_daily(
            frame,
            inner,
            state.frame_tick,
            theme.accent,
            theme.muted_text,
        );
        return;
    };

    let layout = DailyLayout::for_area(area);
    let title = if layout.show_bar && inner_title_width(area) >= 34 {
        "7-Day (Low .. Day Range .. High)"
    } else if layout.show_bar {
        "7-Day (Low..High)"
    } else {
        "7-Day (Low/High)"
    };

    let theme = theme_for(
        weather_code_to_category(bundle.current.weather_code),
        bundle.current.is_day,
        capability,
        state.settings.theme,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(theme.border));
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
                        Cell::from(weather_icon(code, state.settings.icon_mode)).style(
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
                    let mut bar = String::with_capacity(layout.bar_width);
                    for i in 0..layout.bar_width {
                        bar.push(if i >= start && i <= end { '█' } else { '·' });
                    }
                    row_cells.push(Cell::from(bar).style(Style::default().fg(theme.accent)));
                }

                row_cells.push(Cell::from(max_label).style(
                    Style::default().fg(temp_color(&theme, convert_temp(max_c, state.units))),
                ));

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

    let mut table = Table::new(rows, widths).column_spacing(1);
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
        table = table.header(Row::new(header_cells).style(Style::default().fg(theme.muted_text)));
    }

    frame.render_widget(table, inner);
}

fn render_loading_daily(
    frame: &mut Frame,
    area: Rect,
    frame_tick: u64,
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
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn inner_title_width(area: Rect) -> u16 {
    area.width.saturating_sub(2)
}

#[derive(Debug, Clone, Copy)]
struct DailyLayout {
    show_icon: bool,
    show_bar: bool,
    show_header: bool,
    bar_width: usize,
}

impl DailyLayout {
    fn for_area(area: Rect) -> Self {
        let inner_width = area.width.saturating_sub(2) as usize;

        // Width tiers:
        // - wide: icon + bar + header
        // - medium: no icon, still show range bar + header
        // - narrow: low/high only
        if inner_width >= 50 {
            let bar_width = inner_width.saturating_sub(4 + 3 + 5 + 5 + 4).clamp(8, 28);
            Self {
                show_icon: true,
                show_bar: true,
                show_header: true,
                bar_width,
            }
        } else if inner_width >= 36 {
            let bar_width = inner_width.saturating_sub(4 + 5 + 5 + 3).clamp(6, 18);
            Self {
                show_icon: false,
                show_bar: true,
                show_header: true,
                bar_width,
            }
        } else {
            Self {
                show_icon: false,
                show_bar: false,
                show_header: false,
                bar_width: 0,
            }
        }
    }

    fn max_rows(self, inner_height: u16) -> usize {
        let reserved = if self.show_header { 1 } else { 0 };
        usize::from(inner_height.saturating_sub(reserved)).min(7)
    }
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
}
