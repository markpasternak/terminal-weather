#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row},
};

use crate::{
    app::state::{AppState, PanelFocus},
    cli::{Cli, IconMode},
    domain::weather::{
        DailyForecast, ForecastBundle, Units, WeatherCategory, convert_temp, round_temp,
        weather_code_to_category, weather_icon,
    },
    ui::{
        motion_context,
        narrative::build_narrative,
        theme::{detect_color_capability, icon_color, temp_color, theme_for},
    },
};

mod layout;
mod loading;
mod summary;
mod table;

use layout::DailyLayout;
use loading::render_loading_daily;
use table::{build_daily_table, render_daily_table_and_summary};

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
        .title(
            if state.panel_focus == crate::app::state::PanelFocus::Daily {
                "▶ 7-Day"
            } else {
                "7-Day"
            },
        )
        .style(panel_style)
        .border_style(Style::default().fg(theme.border).bg(theme.surface_alt));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    render_loading_daily(
        frame,
        inner,
        motion_context(state, "daily-loading"),
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
    let (layout, theme, panel_style, inner) =
        prepare_daily_bundle_panel(frame, area, state, bundle, capability);
    let content_area = render_daily_context_strip(frame, inner, state, bundle, theme);
    let max_rows = layout.max_rows(content_area.height);
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
    let reveal_rows = visible_reveal_rows(max_rows, state.transition_progress());
    let rows = build_daily_rows(bundle, reveal_rows, ctx);
    let table = build_daily_table(rows, panel_style, layout, theme.muted_text);
    render_daily_table_and_summary(
        frame,
        content_area,
        table,
        bundle,
        state.units,
        theme,
        layout,
    );
}

fn visible_reveal_rows(max_rows: usize, transition_progress: Option<f32>) -> usize {
    let progress = transition_progress.unwrap_or(1.0);
    if progress >= 0.999 {
        return max_rows;
    }
    (((max_rows as f32) * progress.max(0.28)).ceil() as usize).max(usize::from(max_rows > 0))
}

fn prepare_daily_bundle_panel(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    capability: crate::ui::theme::ColorCapability,
) -> (DailyLayout, crate::ui::theme::Theme, Style, Rect) {
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
        .title(if state.panel_focus == PanelFocus::Daily {
            "▶ 7-Day Forecast"
        } else {
            "7-Day Forecast"
        })
        .style(panel_style)
        .border_style(Style::default().fg(theme.border).bg(theme.surface_alt));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    (layout, theme, panel_style, inner)
}

fn render_daily_context_strip(
    frame: &mut Frame,
    inner: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    theme: crate::ui::theme::Theme,
) -> Rect {
    if !state.settings.inline_hints || state.panel_focus != PanelFocus::Daily || inner.height < 8 {
        return inner;
    }

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
    let narrative = build_narrative(state, bundle);
    let line = narrative.focus_hint(PanelFocus::Daily);
    let hint = Paragraph::new(line).style(Style::default().fg(theme.muted_text));
    frame.render_widget(hint, rows[0]);
    rows[1]
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
    let ctx = DailyRowContext {
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
    };

    append_daily_optional_cells(&mut cells, day, &ctx);

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
    ctx: &DailyRowContext,
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
    append_daily_range_cell(cells, ctx);
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

#[must_use]
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
