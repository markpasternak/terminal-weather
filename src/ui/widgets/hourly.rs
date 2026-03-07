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
    app::state::{AppState, PanelFocus},
    cli::Cli,
    domain::weather::{
        Daypart, DaypartSummary, ForecastBundle, HourlyForecast, HourlyViewMode, Units,
        convert_temp, round_temp, summarize_dayparts, weather_code_to_category, weather_icon,
        weather_label_for_time,
    },
    ui::layout::visible_hour_count,
    ui::theme::{Theme, icon_color, resolved_theme, temp_color},
    ui::{motion_context, narrative::build_narrative},
};

mod daypart;
mod table;
mod timeline;

use daypart::render_daypart_cards;
use table::render_table_mode;
use timeline::{render_chart_metrics, render_temp_precip_timeline};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    if let Some(bundle) = &state.weather {
        render_hourly_with_bundle(frame, area, state, bundle);
    } else {
        render_hourly_loading(frame, area, state);
    }
}

fn render_hourly_loading(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = resolved_theme(state);
    let panel_style = Style::default().fg(theme.text).bg(theme.surface);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(
            if state.panel_focus == crate::app::state::PanelFocus::Hourly {
                "▶ Hourly"
            } else {
                "Hourly"
            },
        )
        .style(panel_style)
        .border_style(Style::default().fg(theme.border).bg(theme.surface));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    render_loading_placeholder(
        frame,
        inner,
        motion_context(state, "hourly-loading"),
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
) {
    let theme = resolved_theme(state);
    let panel_style = Style::default().fg(theme.text).bg(theme.surface);
    let effective_mode = effective_hourly_mode(state.hourly_view_mode, area);
    let slice = hourly_slice(bundle, state.hourly_offset, area.width);
    let title = hourly_panel_title(
        effective_mode,
        &slice,
        state.panel_focus == crate::app::state::PanelFocus::Hourly,
    );
    let inner = render_hourly_block(frame, area, title, panel_style, theme);
    if render_empty_hourly_slice(frame, inner, state, &slice, panel_style, theme) {
        return;
    }

    let content_area =
        render_hourly_context_strip(frame, inner, state, bundle, theme, effective_mode);
    render_hourly_mode(
        frame,
        content_area,
        state,
        bundle,
        &slice,
        theme,
        effective_mode,
    );
}

fn render_hourly_block(
    frame: &mut Frame,
    area: Rect,
    title: String,
    panel_style: Style,
    theme: Theme,
) -> Rect {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(panel_style)
        .border_style(Style::default().fg(theme.border).bg(theme.surface));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}

fn render_empty_hourly_slice(
    frame: &mut Frame,
    inner: Rect,
    state: &AppState,
    slice: &[&HourlyForecast],
    panel_style: Style,
    theme: Theme,
) -> bool {
    if !slice.is_empty() {
        return false;
    }
    render_loading_placeholder(
        frame,
        inner,
        motion_context(state, "hourly-empty"),
        panel_style,
        theme.accent,
        theme.muted_text,
    );
    true
}

fn render_hourly_context_strip(
    frame: &mut Frame,
    inner: Rect,
    state: &AppState,
    bundle: &ForecastBundle,
    theme: Theme,
    mode: HourlyViewMode,
) -> Rect {
    if !state.settings.inline_hints || state.panel_focus != PanelFocus::Hourly || inner.height < 8 {
        return inner;
    }

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
    let narrative = build_narrative(state, bundle);
    let motion = motion_context(state, "hourly-focus");
    let pulse =
        if motion.animate && motion.lane("focus").pulse(motion.elapsed_seconds, 0.9, 0) > 0.58 {
            "◦ "
        } else {
            "· "
        };
    let line = match mode {
        HourlyViewMode::Chart => compact_chart_context_line(state, &narrative, rows[0].width),
        _ => format!(
            "{pulse}{}",
            narrative.compact_triage_line(rows[0].width.saturating_sub(2))
        ),
    };
    let hint = Paragraph::new(line).style(Style::default().fg(theme.muted_text));
    frame.render_widget(hint, rows[0]);
    rows[1]
}

fn compact_chart_context_line(
    state: &AppState,
    narrative: &crate::ui::narrative::UiNarrativeState,
    width: u16,
) -> String {
    let action = narrative
        .now_action
        .strip_prefix("Now action: ")
        .unwrap_or(&narrative.now_action);
    let next_change = shorten_next_change(&narrative.next_change);
    let freshness = chart_freshness_summary(state);
    let raw = format!("{action}  |  {next_change}  |  {freshness}");
    truncate_with_ellipsis(&raw, width as usize)
}

fn shorten_next_change(next_change: &str) -> String {
    next_change
        .strip_prefix("Next change in ")
        .map_or_else(|| next_change.to_string(), |value| format!("+{value}"))
}

fn chart_freshness_summary(state: &AppState) -> String {
    freshness_summary_for_state(
        state,
        state
            .refresh_meta
            .age_minutes()
            .map(|minutes| format!("{minutes}m")),
    )
}

fn freshness_summary_for_state(state: &AppState, age: Option<String>) -> String {
    use crate::resilience::freshness::FreshnessState;

    match state.refresh_meta.state {
        FreshnessState::Fresh => freshness_age_label("Fresh", age),
        FreshnessState::Stale => freshness_age_label("Stale", age),
        FreshnessState::Offline => offline_freshness_label(state),
    }
}

fn freshness_age_label(label: &str, age: Option<String>) -> String {
    age.map_or_else(|| label.to_string(), |value| format!("{label} {value}"))
}

fn offline_freshness_label(state: &AppState) -> String {
    state.refresh_meta.retry_in_seconds().map_or_else(
        || "Offline".to_string(),
        |retry| format!("Offline retry {retry}s"),
    )
}

fn truncate_with_ellipsis(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = input.chars().count();
    if count <= max_chars {
        return input.to_string();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }

    input.char_indices().nth(max_chars - 3).map_or_else(
        || input.to_string(),
        |(byte_idx, _)| {
            let mut out = String::with_capacity(byte_idx + 3);
            out.push_str(&input[..byte_idx]);
            out.push_str("...");
            out
        },
    )
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

fn hourly_panel_title(mode: HourlyViewMode, slice: &[&HourlyForecast], focused: bool) -> String {
    let mode_label = match mode {
        HourlyViewMode::Table => "Table",
        HourlyViewMode::Hybrid => "Hybrid",
        HourlyViewMode::Chart => "Chart",
    };
    let focus_prefix = if focused { "▶ " } else { "" };
    if let (Some(first), Some(last)) = (slice.first(), slice.last()) {
        let first_date = first.time.format("%a %d %b");
        let last_date = last.time.format("%a %d %b");
        if first.time.date() == last.time.date() {
            format!("{focus_prefix}Hourly · {mode_label} · {first_date}")
        } else {
            format!("{focus_prefix}Hourly · {mode_label} · {first_date} → {last_date}")
        }
    } else {
        format!("{focus_prefix}Hourly · {mode_label}")
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

    let chunks = Layout::vertical([Constraint::Min(6), Constraint::Length(1)]).split(area);
    let timeline_stats = render_temp_precip_timeline(frame, chunks[0], slice, theme, state.units);
    render_chart_metrics(frame, chunks[1], timeline_stats, theme);
    true
}

fn render_loading_placeholder(
    frame: &mut Frame,
    area: Rect,
    motion: crate::ui::animation::UiMotionContext,
    panel_style: Style,
    accent: Color,
    muted: Color,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let slots = (usize::from(area.width) / 6).max(4);
    let row1 = (0..slots)
        .map(|idx| {
            let wave = motion
                .lane("hourly-loading")
                .pulse(motion.elapsed_seconds, 1.0, idx as u64);
            if wave > 0.82 {
                '◆'
            } else if wave > 0.68 {
                '◇'
            } else {
                '·'
            }
        })
        .collect::<String>();
    let row2 = (0..slots)
        .map(|i| {
            let wave =
                motion
                    .lane("hourly-loading-bottom")
                    .pulse(motion.elapsed_seconds, 0.7, i as u64);
            if wave > 0.62 { '◦' } else { ' ' }
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
