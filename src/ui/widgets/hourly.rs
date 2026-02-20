#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use chrono::{Datelike, Timelike};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::{
    app::state::AppState,
    cli::Cli,
    domain::weather::{
        Daypart, DaypartSummary, ForecastBundle, HourlyForecast, HourlyViewMode, Units,
        convert_temp, round_temp, summarize_dayparts, weather_code_to_category, weather_icon,
        weather_label_for_time,
    },
    ui::layout::visible_hour_count,
    ui::theme::{Theme, detect_color_capability, icon_color, temp_color, theme_for},
};

mod daypart;
mod table;
mod timeline;

use daypart::render_daypart_cards;
use table::render_table_mode;
use timeline::{render_chart_metrics, render_temp_precip_timeline};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    let capability = detect_color_capability(state.color_mode);
    if let Some(bundle) = &state.weather {
        render_hourly_with_bundle(frame, area, state, bundle, capability);
    } else {
        render_hourly_loading(frame, area, state, capability);
    }
}

fn render_hourly_loading(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    capability: crate::ui::theme::ColorCapability,
) {
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
}

fn render_hourly_with_bundle(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    capability: crate::ui::theme::ColorCapability,
) {
    let theme = theme_for(
        weather_code_to_category(bundle.current.weather_code),
        bundle.current.is_day,
        capability,
        state.settings.theme,
    );
    let panel_style = Style::default().fg(theme.text).bg(theme.surface);
    let effective_mode = effective_hourly_mode(state.hourly_view_mode, area);
    let slice = hourly_slice(bundle, state.hourly_offset, area.width);
    let title = hourly_panel_title(effective_mode, &slice);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(panel_style)
        .border_style(Style::default().fg(theme.border).bg(theme.surface));
    let inner = block.inner(area);
    frame.render_widget(block, area);

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

    render_hourly_mode(frame, inner, state, bundle, &slice, theme, effective_mode);
}

fn hourly_slice(bundle: &ForecastBundle, offset: usize, area_width: u16) -> Vec<&HourlyForecast> {
    let show = visible_hour_count(area_width);
    let offset = offset.min(bundle.hourly.len().saturating_sub(1));
    bundle
        .hourly
        .iter()
        .skip(offset)
        .take(show)
        .collect::<Vec<_>>()
}

fn hourly_panel_title(mode: HourlyViewMode, slice: &[&HourlyForecast]) -> String {
    let mode_label = match mode {
        HourlyViewMode::Table => "Table",
        HourlyViewMode::Hybrid => "Hybrid",
        HourlyViewMode::Chart => "Chart",
    };
    if let (Some(first), Some(last)) = (slice.first(), slice.last()) {
        let first_date = first.time.format("%a %d %b");
        let last_date = last.time.format("%a %d %b");
        if first.time.date() == last.time.date() {
            format!("Hourly · {mode_label} · {first_date}")
        } else {
            format!("Hourly · {mode_label} · {first_date} → {last_date}")
        }
    } else {
        format!("Hourly · {mode_label}")
    }
}

fn render_hourly_mode(
    frame: &mut Frame,
    inner: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    slice: &[&HourlyForecast],
    theme: Theme,
    mode: HourlyViewMode,
) {
    match mode {
        HourlyViewMode::Table => render_table_mode(frame, inner, state, bundle, slice, theme),
        HourlyViewMode::Hybrid => {
            if !render_hybrid_mode(frame, inner, state, bundle, slice, theme) {
                render_table_mode(frame, inner, state, bundle, slice, theme);
            }
        }
        HourlyViewMode::Chart => {
            if !render_chart_mode(frame, inner, state, bundle, slice, theme) {
                render_table_mode(frame, inner, state, bundle, slice, theme);
            }
        }
    }
}

fn effective_hourly_mode(requested: HourlyViewMode, area: Rect) -> HourlyViewMode {
    let inner_width = area.width.saturating_sub(2);
    let inner_height = area.height.saturating_sub(2);
    if inner_width < 70 || inner_height < 5 {
        return HourlyViewMode::Table;
    }

    match requested {
        HourlyViewMode::Hybrid if inner_height >= 6 => HourlyViewMode::Hybrid,
        HourlyViewMode::Chart if inner_height >= 8 => HourlyViewMode::Chart,
        _ => HourlyViewMode::Table,
    }
}

fn render_hybrid_mode(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    slice: &[&HourlyForecast],
    theme: Theme,
) -> bool {
    if area.height < 7 {
        return false;
    }

    let chunks = if area.height >= 12 {
        Layout::vertical([Constraint::Length(5), Constraint::Min(4)]).split(area)
    } else if area.height >= 10 {
        Layout::vertical([Constraint::Length(4), Constraint::Min(3)]).split(area)
    } else {
        Layout::vertical([Constraint::Length(3), Constraint::Min(2)]).split(area)
    };

    let _ = render_temp_precip_timeline(frame, chunks[0], slice, theme, state.units);

    let day_count = if area.width >= 100 { 2 } else { 1 };
    render_daypart_cards(frame, chunks[1], bundle, state, theme, day_count)
}

fn render_chart_mode(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    slice: &[&HourlyForecast],
    theme: Theme,
) -> bool {
    let _ = bundle;
    if area.height < 8 {
        return false;
    }

    let chunks = Layout::vertical([Constraint::Min(6), Constraint::Length(2)]).split(area);
    let timeline_stats = render_temp_precip_timeline(frame, chunks[0], slice, theme, state.units);
    render_chart_metrics(frame, chunks[1], timeline_stats, theme);
    true
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

    let placeholder_rows = [
        Row::new(vec![
            Cell::from("Loading timeline").style(Style::default().fg(accent)),
        ]),
        Row::new(vec![Cell::from(row1).style(Style::default().fg(muted))]),
        Row::new(vec![Cell::from(row2).style(Style::default().fg(accent))]),
    ];
    let table = Table::new(placeholder_rows, [Constraint::Min(1)])
        .column_spacing(1)
        .style(panel_style);
    frame.render_widget(table, area);
}

#[cfg(test)]
mod tests;
