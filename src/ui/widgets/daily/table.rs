use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Row, Table},
};

use crate::{
    domain::weather::{ForecastBundle, Units},
    ui::widgets::daily::{layout::DailyLayout, summary::render_week_summary},
};

pub(super) fn build_daily_table(
    rows: Vec<Row<'static>>,
    panel_style: Style,
    layout: DailyLayout,
    muted_text: Color,
) -> Table<'static> {
    let mut table = Table::new(rows, daily_table_widths(layout))
        .column_spacing(layout.column_spacing)
        .style(panel_style);
    if layout.show_header {
        table = table
            .header(Row::new(daily_header_cells(layout)).style(Style::default().fg(muted_text)));
    }
    table
}

pub(super) fn render_daily_table_and_summary(
    frame: &mut Frame,
    inner: Rect,
    table: Table<'static>,
    bundle: &ForecastBundle,
    units: Units,
    theme: crate::ui::theme::Theme,
    layout: DailyLayout,
) {
    let row_count = bundle.daily.len().min(layout.max_rows(inner.height)) as u16;
    let table_height = row_count.saturating_add(u16::from(layout.show_header));
    let (table_area, summary_slot) = split_table_and_summary(inner, table_height);
    frame.render_widget(table, table_area);
    if let Some(summary_area) = summary_slot {
        render_week_summary(frame, summary_area, bundle, units, theme);
    }
}

fn daily_table_widths(layout: DailyLayout) -> Vec<Constraint> {
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
    widths
}

fn daily_header_cells(layout: DailyLayout) -> Vec<Line<'static>> {
    let mut cells = vec![Line::from("Day")];
    if layout.show_icon {
        cells.push(Line::from("Wx"));
    }
    cells.push(Line::from("Low"));
    if layout.show_bar {
        cells.push(Line::from("Range"));
    }
    cells.push(Line::from("High"));
    if layout.show_precip_col {
        cells.push(Line::from("Pmm"));
    }
    if layout.show_gust_col {
        cells.push(Line::from("Gst"));
    }
    cells
}

fn split_table_and_summary(inner: Rect, table_height: u16) -> (Rect, Option<Rect>) {
    if inner.height <= table_height.saturating_add(2) {
        return (inner, None);
    }

    let summary_y = inner.y.saturating_add(table_height);
    let summary_height = inner.bottom().saturating_sub(summary_y);
    let table_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: table_height,
    };
    let summary_slot = (summary_height > 0).then_some(Rect {
        x: inner.x,
        y: summary_y,
        width: inner.width,
        height: summary_height,
    });
    (table_area, summary_slot)
}
