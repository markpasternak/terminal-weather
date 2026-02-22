use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Cell, Row, Table},
};

pub(super) fn render_loading_daily(
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
