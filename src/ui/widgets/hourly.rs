use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{
    app::state::AppState,
    cli::Cli,
    domain::weather::{convert_temp, round_temp, weather_code_to_category, weather_icon},
    ui::layout::{HourlyDensity, hourly_density},
    ui::theme::{detect_color_capability, icon_color, temp_color, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    let capability = detect_color_capability();

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

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Hourly")
        .style(panel_style)
        .border_style(Style::default().fg(theme.border).bg(theme.surface));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let show = match hourly_density(area.width) {
        HourlyDensity::Full16 => 16,
        HourlyDensity::Full12 => 12,
        HourlyDensity::Compact8 => 8,
        HourlyDensity::Compact6 => 6,
    };

    let offset = state
        .hourly_offset
        .min(bundle.hourly.len().saturating_sub(1));
    let slice = bundle
        .hourly
        .iter()
        .skip(offset)
        .take(show)
        .collect::<Vec<_>>();
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

    let times = Row::new(
        slice
            .iter()
            .enumerate()
            .map(|(idx, h)| {
                let is_now = offset == 0 && idx == 0;
                let label = if is_now {
                    "Now".to_string()
                } else {
                    h.time.format("%H:%M").to_string()
                };
                let mut cell = Cell::from(label);
                if is_now {
                    cell = cell.style(
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    );
                } else {
                    cell = cell.style(Style::default().fg(theme.muted_text));
                }
                cell
            })
            .collect::<Vec<_>>(),
    );

    let icons = Row::new(
        slice
            .iter()
            .map(|h| {
                let code = h.weather_code.unwrap_or(bundle.current.weather_code);
                Cell::from(weather_icon(code, state.settings.icon_mode))
                    .style(Style::default().fg(icon_color(&theme, weather_code_to_category(code))))
            })
            .collect::<Vec<_>>(),
    );

    let temps = Row::new(
        slice
            .iter()
            .map(|h| {
                let temp = h.temperature_2m_c.map(|t| convert_temp(t, state.units));
                Cell::from(
                    temp.map(|t| format!("{}°", round_temp(t)))
                        .unwrap_or_else(|| "--".to_string()),
                )
                .style(
                    Style::default().fg(temp
                        .map(|t| temp_color(&theme, t))
                        .unwrap_or(theme.muted_text)),
                )
            })
            .collect::<Vec<_>>(),
    )
    .style(Style::default().add_modifier(Modifier::BOLD));

    let precip = Row::new(
        slice
            .iter()
            .map(|h| {
                let text = h
                    .precipitation_probability
                    .map(|p| format!("P{:>2}", p.round() as i32))
                    .unwrap_or_else(|| "P--".to_string());
                Cell::from(text).style(Style::default().fg(theme.info))
            })
            .collect::<Vec<_>>(),
    );

    let humidity = Row::new(
        slice
            .iter()
            .map(|h| {
                let text = h
                    .relative_humidity_2m
                    .map(|rh| format!("H{:>2}", rh.round() as i32))
                    .unwrap_or_else(|| "H--".to_string());
                Cell::from(text).style(Style::default().fg(theme.muted_text))
            })
            .collect::<Vec<_>>(),
    );

    let trend = Row::new(
        slice
            .iter()
            .map(|h| {
                let glyph = h
                    .temperature_2m_c
                    .map(|t| convert_temp(t, state.units))
                    .map(temp_level_glyph)
                    .unwrap_or('·');
                Cell::from(glyph.to_string()).style(Style::default().fg(theme.accent))
            })
            .collect::<Vec<_>>(),
    );

    let mut rows = vec![times, icons, temps];
    if inner.height >= 5 {
        rows.push(precip);
    }
    if inner.height >= 6 {
        rows.push(humidity);
    }
    if inner.height >= 7 {
        rows.push(trend);
    }

    let needed = rows.len() as u16;
    let table_area = if inner.height > needed {
        Rect {
            x: inner.x,
            y: inner.y + (inner.height - needed) / 2,
            width: inner.width,
            height: needed,
        }
    } else {
        inner
    };

    let column_spacing = if inner.width >= 140 {
        2
    } else if inner.width >= 104 {
        1
    } else {
        0
    };
    let widths = vec![Constraint::Ratio(1, slice.len().max(1) as u32); slice.len()];
    let table = Table::new(rows, widths)
        .column_spacing(column_spacing)
        .style(panel_style);
    frame.render_widget(table, table_area);
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

    let rows = [
        Row::new(vec![
            Cell::from("Loading timeline").style(Style::default().fg(accent)),
        ]),
        Row::new(vec![Cell::from(row1).style(Style::default().fg(muted))]),
        Row::new(vec![Cell::from(row2).style(Style::default().fg(accent))]),
    ];
    let table = Table::new(rows, [Constraint::Min(1)])
        .column_spacing(1)
        .style(panel_style);
    frame.render_widget(table, area);
}

fn temp_level_glyph(temp: f32) -> char {
    if temp <= -8.0 {
        '▁'
    } else if temp <= 0.0 {
        '▂'
    } else if temp <= 8.0 {
        '▄'
    } else if temp <= 16.0 {
        '▅'
    } else if temp <= 24.0 {
        '▆'
    } else if temp <= 30.0 {
        '▇'
    } else {
        '█'
    }
}
