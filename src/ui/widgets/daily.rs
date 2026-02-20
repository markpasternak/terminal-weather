#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::{
    app::state::AppState,
    cli::{Cli, IconMode},
    domain::weather::{
        DailyForecast, ForecastBundle, Units, WeatherCategory, convert_temp, round_temp,
        weather_code_to_category, weather_icon,
    },
    ui::theme::{detect_color_capability, icon_color, temp_color, theme_for},
};

mod layout;
mod summary;

use layout::DailyLayout;
use summary::render_week_summary;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    let capability = detect_color_capability(state.color_mode);
    match state.weather.as_ref() {
        Some(bundle) => render_daily_with_bundle(frame, area, state, bundle, capability),
        None => render_daily_loading(frame, area, state, capability),
    }
}

fn render_daily_loading(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    capability: crate::ui::theme::ColorCapability,
) {
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
}

fn render_daily_with_bundle(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    capability: crate::ui::theme::ColorCapability,
) {
    let layout = DailyLayout::for_area(area);
    let theme = theme_for(
        weather_code_to_category(bundle.current.weather_code),
        bundle.current.is_day,
        capability,
        state.settings.theme,
    );
    let panel_style = Style::default().fg(theme.text).bg(theme.surface_alt);
    let block = Block::default()
        .borders(Borders::ALL)
        .title("7-Day Forecast")
        .style(panel_style)
        .border_style(Style::default().fg(theme.border).bg(theme.surface_alt));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let max_rows = layout.max_rows(inner.height);
    if max_rows == 0 {
        return;
    }

    let (global_min, global_max) = global_temp_bounds(bundle);
    let ctx = DailyRenderContext {
        units: state.units,
        icon_mode: state.settings.icon_mode,
        layout,
        theme,
        global_min,
        global_max,
    };
    let rows = build_daily_rows(bundle, max_rows, ctx);
    let table = build_daily_table(rows, panel_style, layout, theme.muted_text);
    render_daily_table_and_summary(frame, inner, table, bundle, state.units, theme, layout);
}

fn build_daily_table(
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

fn render_daily_table_and_summary(
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

fn global_temp_bounds(bundle: &ForecastBundle) -> (f32, f32) {
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
    (global_min, global_max)
}

fn build_daily_rows(
    bundle: &ForecastBundle,
    max_rows: usize,
    ctx: DailyRenderContext,
) -> Vec<Row<'static>> {
    bundle
        .daily
        .iter()
        .take(max_rows)
        .enumerate()
        .map(|(idx, day)| build_daily_row(day, idx == 0, ctx))
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct DailyRenderContext {
    units: Units,
    icon_mode: IconMode,
    layout: DailyLayout,
    theme: crate::ui::theme::Theme,
    global_min: f32,
    global_max: f32,
}

fn build_daily_row(day: &DailyForecast, is_today: bool, ctx: DailyRenderContext) -> Row<'static> {
    let DailyRenderContext {
        units,
        icon_mode,
        layout,
        theme,
        global_min,
        global_max,
    } = ctx;

    let min_c = day.temperature_min_c.unwrap_or(global_min);
    let max_c = day.temperature_max_c.unwrap_or(global_max);
    let min_label = format!("{}°", round_temp(convert_temp(min_c, units)));
    let max_label = format!("{}°", round_temp(convert_temp(max_c, units)));

    let mut cells = daily_base_cells(day, theme);
    append_daily_optional_cells(
        &mut cells,
        day,
        DailyRowContext {
            units,
            icon_mode,
            layout,
            theme,
            min_c,
            max_c,
            global_min,
            global_max,
            min_label,
            max_label,
        },
    );

    let row = Row::new(cells);
    if is_today {
        row.style(Style::default().add_modifier(Modifier::BOLD))
    } else {
        row
    }
}

#[derive(Debug)]
struct DailyRowContext {
    units: Units,
    icon_mode: IconMode,
    layout: DailyLayout,
    theme: crate::ui::theme::Theme,
    min_c: f32,
    max_c: f32,
    global_min: f32,
    global_max: f32,
    min_label: String,
    max_label: String,
}

fn daily_base_cells(day: &DailyForecast, theme: crate::ui::theme::Theme) -> Vec<Cell<'static>> {
    vec![Cell::from(day.date.format("%a").to_string()).style(Style::default().fg(theme.text))]
}

fn append_daily_optional_cells(
    cells: &mut Vec<Cell<'static>>,
    day: &DailyForecast,
    ctx: DailyRowContext,
) {
    append_daily_icon_cell(cells, day, ctx.layout.show_icon, ctx.icon_mode, ctx.theme);
    append_daily_temp_cells(
        cells,
        ctx.units,
        ctx.min_c,
        ctx.max_c,
        &ctx.min_label,
        &ctx.max_label,
        ctx.theme,
    );
    append_daily_range_cell(cells, &ctx);
    append_daily_precip_cell(cells, day, ctx.layout.show_precip_col, ctx.theme);
    append_daily_gust_cell(cells, day, ctx.layout.show_gust_col, ctx.theme);
}

fn append_daily_icon_cell(
    cells: &mut Vec<Cell<'static>>,
    day: &DailyForecast,
    show_icon: bool,
    icon_mode: IconMode,
    theme: crate::ui::theme::Theme,
) {
    if !show_icon {
        return;
    }
    let code = day.weather_code.unwrap_or(3);
    cells.push(
        Cell::from(weather_icon(code, icon_mode, true))
            .style(Style::default().fg(icon_color(&theme, weather_code_to_category(code)))),
    );
}

fn append_daily_temp_cells(
    cells: &mut Vec<Cell<'static>>,
    units: Units,
    min_c: f32,
    max_c: f32,
    min_label: &str,
    max_label: &str,
    theme: crate::ui::theme::Theme,
) {
    cells.push(
        Cell::from(min_label.to_string())
            .style(Style::default().fg(temp_color(&theme, convert_temp(min_c, units)))),
    );
    cells.push(
        Cell::from(max_label.to_string())
            .style(Style::default().fg(temp_color(&theme, convert_temp(max_c, units)))),
    );
}

fn append_daily_range_cell(cells: &mut Vec<Cell<'static>>, ctx: &DailyRowContext) {
    if !ctx.layout.show_bar {
        return;
    }
    cells.insert(
        cells.len().saturating_sub(1),
        Cell::from(build_range_bar(
            ctx.min_c,
            ctx.max_c,
            ctx.global_min,
            ctx.global_max,
            ctx.layout,
            ctx.theme,
        )),
    );
}

fn append_daily_precip_cell(
    cells: &mut Vec<Cell<'static>>,
    day: &DailyForecast,
    show_precip_col: bool,
    theme: crate::ui::theme::Theme,
) {
    if !show_precip_col {
        return;
    }
    let precip = day
        .precipitation_sum_mm
        .map_or_else(|| "--.-".to_string(), |v| format!("{v:>4.1}"));
    cells.push(Cell::from(precip).style(Style::default().fg(theme.info)));
}

fn append_daily_gust_cell(
    cells: &mut Vec<Cell<'static>>,
    day: &DailyForecast,
    show_gust_col: bool,
    theme: crate::ui::theme::Theme,
) {
    if !show_gust_col {
        return;
    }
    let gust = day.wind_gusts_10m_max.map_or_else(
        || "-- ".to_string(),
        |v| format!("{:>3}", crate::domain::weather::round_wind_speed(v)),
    );
    cells.push(Cell::from(gust).style(Style::default().fg(theme.warning)));
}

fn build_range_bar(
    min_c: f32,
    max_c: f32,
    global_min: f32,
    global_max: f32,
    layout: DailyLayout,
    theme: crate::ui::theme::Theme,
) -> Line<'static> {
    let (start, end) = bar_bounds(min_c, max_c, global_min, global_max, layout.bar_width);
    let clamped_start = start.min(layout.bar_width);
    let clamped_end = end.min(layout.bar_width.saturating_sub(1));
    let before = "·".repeat(clamped_start);
    let fill_len = clamped_end.saturating_sub(clamped_start).saturating_add(1);
    let fill = "█".repeat(fill_len);
    let after = "·".repeat(layout.bar_width.saturating_sub(clamped_start + fill_len));
    Line::from(vec![
        Span::styled(before, Style::default().fg(theme.range_track)),
        Span::styled(fill, Style::default().fg(theme.accent)),
        Span::styled(after, Style::default().fg(theme.range_track)),
    ])
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

fn daily_header_cells(layout: DailyLayout) -> Vec<&'static str> {
    let mut cells = vec!["Day"];
    if layout.show_icon {
        cells.push("Wx");
    }
    cells.push("Low");
    if layout.show_bar {
        cells.push("Range");
    }
    cells.push("High");
    if layout.show_precip_col {
        cells.push("Pmm");
    }
    if layout.show_gust_col {
        cells.push("Gst");
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
mod tests;
