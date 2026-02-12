use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
};

use crate::{
    app::state::AppState,
    cli::Cli,
    domain::weather::{convert_temp, round_temp, weather_icon},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, cli: &Cli) {
    let block = Block::default().borders(Borders::ALL).title("7-Day");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(bundle) = &state.weather else {
        return;
    };

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

    let range = (global_max - global_min).max(1.0);

    let rows = bundle
        .daily
        .iter()
        .take(7)
        .enumerate()
        .map(|(idx, day)| {
            let is_today = idx == 0;
            let min_c = day.temperature_min_c.unwrap_or(global_min);
            let max_c = day.temperature_max_c.unwrap_or(global_max);

            let min_label = format!("{}°", round_temp(convert_temp(min_c, state.units)));
            let max_label = format!("{}°", round_temp(convert_temp(max_c, state.units)));

            let start = (((min_c - global_min) / range) * 12.0)
                .round()
                .clamp(0.0, 12.0) as usize;
            let end = (((max_c - global_min) / range) * 12.0)
                .round()
                .clamp(0.0, 12.0) as usize;

            let mut bar = String::with_capacity(12);
            for i in 0..12 {
                bar.push(if i >= start && i <= end { '█' } else { '·' });
            }

            let row = Row::new(vec![
                day.date.format("%a").to_string(),
                weather_icon(day.weather_code.unwrap_or(3), crate::icon_mode(cli)).to_string(),
                min_label,
                bar,
                max_label,
            ]);

            if is_today {
                row.style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row
            }
        })
        .collect::<Vec<_>>();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(6),
        Constraint::Length(14),
        Constraint::Length(6),
    ];

    frame.render_widget(Table::new(rows, widths), inner);
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
}
