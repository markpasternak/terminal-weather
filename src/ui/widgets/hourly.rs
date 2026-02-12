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

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, cli: &Cli) {
    let capability = detect_color_capability();

    let Some(bundle) = &state.weather else {
        let theme = theme_for(
            crate::domain::weather::WeatherCategory::Unknown,
            true,
            capability,
            cli.theme,
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Hourly")
            .border_style(Style::default().fg(theme.border));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        render_loading_placeholder(
            frame,
            inner,
            state.frame_tick,
            theme.accent,
            theme.muted_text,
        );
        return;
    };

    let theme = theme_for(
        weather_code_to_category(bundle.current.weather_code),
        bundle.current.is_day,
        capability,
        cli.theme,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Hourly")
        .border_style(Style::default().fg(theme.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let show = match hourly_density(area.width) {
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
                Cell::from(weather_icon(code, crate::icon_mode(cli)))
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

    let widths = vec![Constraint::Length(6); slice.len()];
    let table = Table::new([times, icons, temps], widths);
    frame.render_widget(table, inner);
}

fn render_loading_placeholder(
    frame: &mut Frame,
    area: Rect,
    frame_tick: u64,
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
    let table = Table::new(rows, [Constraint::Min(1)]).column_spacing(1);
    frame.render_widget(table, area);
}
