use chrono::{Local, Timelike};
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
    ui::theme::{detect_color_capability, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, cli: &Cli) {
    let Some(bundle) = &state.weather else {
        let block = Block::default().borders(Borders::ALL).title("Hourly");
        frame.render_widget(block, area);
        return;
    };

    let capability = detect_color_capability();
    let theme = theme_for(
        weather_code_to_category(bundle.current.weather_code),
        bundle.current.is_day,
        capability,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Hourly")
        .border_style(Style::default().fg(theme.muted_text));
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
                let mut cell = Cell::from(label);
                if h.time.hour() == now_hour {
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
                    .style(Style::default().fg(icon_color_for(code)))
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
                .style(Style::default().fg(temp.map(temp_color_for).unwrap_or(Color::Gray)))
            })
            .collect::<Vec<_>>(),
    )
    .style(Style::default().add_modifier(Modifier::BOLD));

    let widths = vec![Constraint::Length(6); slice.len()];
    let table = Table::new([times, icons, temps], widths);
    frame.render_widget(table, inner);
}

fn icon_color_for(code: u8) -> Color {
    match weather_code_to_category(code) {
        crate::domain::weather::WeatherCategory::Clear => Color::Yellow,
        crate::domain::weather::WeatherCategory::Cloudy => Color::Gray,
        crate::domain::weather::WeatherCategory::Rain => Color::Cyan,
        crate::domain::weather::WeatherCategory::Snow => Color::White,
        crate::domain::weather::WeatherCategory::Fog => Color::DarkGray,
        crate::domain::weather::WeatherCategory::Thunder => Color::Magenta,
        crate::domain::weather::WeatherCategory::Unknown => Color::LightBlue,
    }
}

fn temp_color_for(temp: f32) -> Color {
    if temp <= -5.0 {
        Color::LightBlue
    } else if temp <= 5.0 {
        Color::Cyan
    } else if temp <= 18.0 {
        Color::Green
    } else if temp <= 28.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}
