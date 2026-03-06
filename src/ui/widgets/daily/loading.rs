use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Cell, Row, Table},
};

use crate::ui::animation::UiMotionContext;

pub(super) fn render_loading_daily(
    frame: &mut Frame,
    area: Rect,
    motion: UiMotionContext,
    panel_style: Style,
    accent: Color,
    muted: Color,
) {
    if area.height == 0 || area.width < 12 {
        return;
    }
    let rows = usize::from(area.height).min(6);
    let width = usize::from(area.width.saturating_sub(10)).clamp(8, 28);
    frame.render_widget(
        build_loading_table(
            loading_rows(rows, width, motion, accent, muted),
            width,
            panel_style,
        ),
        area,
    );
}

fn loading_rows(
    rows: usize,
    width: usize,
    motion: UiMotionContext,
    accent: Color,
    muted: Color,
) -> Vec<Row<'static>> {
    (0..rows)
        .map(|idx| {
            Row::new(vec![
                Cell::from(format!("D{}", idx + 1)).style(Style::default().fg(muted)),
                Cell::from(loading_bar(width, motion, idx)).style(Style::default().fg(accent)),
                Cell::from("--°").style(Style::default().fg(muted)),
            ])
        })
        .collect()
}

fn loading_bar(width: usize, motion: UiMotionContext, row_idx: usize) -> String {
    let lane = motion.lane("daily-loading");
    (0..width)
        .map(|col| {
            loading_bar_glyph(lane.pulse(
                motion.elapsed_seconds,
                0.55 + row_idx as f32 * 0.10,
                (col + row_idx) as u64,
            ))
        })
        .collect()
}

fn loading_bar_glyph(wave: f32) -> char {
    if wave > 0.82 {
        '█'
    } else if wave > 0.68 {
        '▓'
    } else if wave > 0.55 {
        '▒'
    } else {
        '·'
    }
}

fn build_loading_table(
    rows: Vec<Row<'static>>,
    width: usize,
    panel_style: Style,
) -> Table<'static> {
    Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(width as u16),
            Constraint::Length(4),
        ],
    )
    .column_spacing(1)
    .style(panel_style)
}
