use chrono::{Local, Timelike};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{
    app::state::AppState,
    cli::Cli,
    domain::weather::{convert_temp, round_temp, weather_icon},
    ui::layout::{HourlyDensity, hourly_density},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, cli: &Cli) {
    let block = Block::default().borders(Borders::ALL).title("Hourly");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(bundle) = &state.weather else {
        return;
    };

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

    let now_hour = Local::now().hour();

    let times = Row::new(
        slice
            .iter()
            .map(|h| {
                let label = if h.time.hour() == now_hour {
                    "Now".to_string()
                } else {
                    h.time.format("%H:%M").to_string()
                };
                Cell::from(label)
            })
            .collect::<Vec<_>>(),
    );

    let icons = Row::new(
        slice
            .iter()
            .map(|h| {
                Cell::from(weather_icon(
                    h.weather_code.unwrap_or(bundle.current.weather_code),
                    crate::icon_mode(cli),
                ))
            })
            .collect::<Vec<_>>(),
    );

    let temps = Row::new(
        slice
            .iter()
            .map(|h| {
                Cell::from(
                    h.temperature_2m_c
                        .map(|t| format!("{}°", round_temp(convert_temp(t, state.units))))
                        .unwrap_or_else(|| "--".to_string()),
                )
            })
            .collect::<Vec<_>>(),
    )
    .style(Style::default().add_modifier(Modifier::BOLD));

    let widths = vec![Constraint::Length(6); slice.len()];
    let table = Table::new([times, icons, temps], widths);
    frame.render_widget(table, inner);
}
