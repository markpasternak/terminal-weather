use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::{
    app::{settings::RecentLocation, state::AppState},
    ui::theme::{Theme, resolved_theme},
};

use super::shared::{popup_block, popup_panel_style};

const SEARCH_LABEL: &str = "Search: ";
const CITY_QUERY_MAX: usize = 50;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);

    let theme = resolved_theme(state);
    let panel_style = popup_panel_style(theme);

    let block = popup_block("Locations", theme, panel_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(2),
    ])
    .split(inner);

    render_query_line(frame, chunks[0], state, theme);
    let items = recent_city_items(state, theme);
    let mut list_state =
        ListState::default().with_selected(Some(selected_recent_index(state, &items)));
    let list = recent_city_list(items, panel_style, theme);
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    render_status_line(frame, chunks[2], state, theme);
}

fn render_query_line(frame: &mut Frame, area: Rect, state: &AppState, theme: Theme) {
    let inner_area = render_query_block(frame, area, theme);
    let [input_area, count_area] = query_line_areas(inner_area);

    frame.render_widget(Paragraph::new(query_line(state, theme)), input_area);
    frame.render_widget(query_counter(state, theme), count_area);

    frame.set_cursor_position((query_cursor_x(state, input_area), input_area.y));
}

fn render_query_block(frame: &mut Frame, area: Rect, theme: Theme) -> Rect {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.popup_border));
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    inner_area
}

fn query_line_areas(inner_area: Rect) -> [Rect; 2] {
    Layout::horizontal([
        Constraint::Min(10),
        Constraint::Length(10), // Space for (50/50)
    ])
    .areas(inner_area)
}

fn query_line(state: &AppState, theme: Theme) -> Line<'static> {
    let mut spans = vec![Span::styled(
        SEARCH_LABEL,
        Style::default().fg(theme.popup_muted_text),
    )];
    spans.extend(query_spans(state, theme));
    Line::from(spans)
}

fn query_spans(state: &AppState, theme: Theme) -> Vec<Span<'static>> {
    if state.city_query.is_empty() {
        empty_query_spans(theme)
    } else {
        active_query_spans(state, theme)
    }
}

fn empty_query_spans(theme: Theme) -> Vec<Span<'static>> {
    vec![
        Span::styled(
            "Type a city and press ",
            Style::default().fg(theme.popup_muted_text),
        ),
        Span::styled(
            "Enter",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " (or use history)",
            Style::default().fg(theme.popup_muted_text),
        ),
    ]
}

fn active_query_spans(state: &AppState, theme: Theme) -> Vec<Span<'static>> {
    vec![Span::styled(
        state.city_query.clone(),
        Style::default()
            .fg(theme.popup_text)
            .add_modifier(Modifier::BOLD),
    )]
}

fn query_counter(state: &AppState, theme: Theme) -> Paragraph<'static> {
    let count = state.city_query.chars().count();
    let count_style = if count >= CITY_QUERY_MAX {
        Style::default().fg(theme.warning)
    } else {
        Style::default().fg(theme.popup_muted_text)
    };

    Paragraph::new(Line::from(format!("({count}/{CITY_QUERY_MAX})")))
        .style(count_style)
        .alignment(Alignment::Right)
}

fn query_cursor_x(state: &AppState, input_area: Rect) -> u16 {
    let query_width = if state.city_query.is_empty() {
        0
    } else {
        Line::from(state.city_query.as_str()).width() as u16
    };

    // Keep the cursor pinned to the typed query even if the surrounding layout shifts.
    input_area.x + SEARCH_LABEL.len() as u16 + query_width
}

fn recent_city_items(state: &AppState, theme: Theme) -> Vec<ListItem<'static>> {
    let mut items = state
        .settings
        .recent_locations
        .iter()
        .take(9)
        .enumerate()
        .map(|(idx, saved)| ListItem::new(format_recent_location(idx, saved, state)))
        .collect::<Vec<_>>();
    if !items.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "Clear all recent locations",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )])));
    }
    items
}

fn selected_recent_index(state: &AppState, items: &[ListItem<'_>]) -> usize {
    state
        .city_history_selected
        .min(items.len().saturating_sub(1))
}

fn recent_city_list(
    items: Vec<ListItem<'static>>,
    panel_style: Style,
    theme: Theme,
) -> List<'static> {
    let body = if items.is_empty() {
        let text = Line::from(vec![
            Span::styled(
                "No recent cities. Type to search · ",
                Style::default().fg(theme.popup_muted_text),
            ),
            Span::styled(
                "Esc",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" close", Style::default().fg(theme.popup_muted_text)),
        ]);
        List::new(vec![ListItem::new(text)])
    } else {
        List::new(items)
    };
    body.style(panel_style)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ")
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled("Recent (", Style::default().fg(theme.popup_muted_text)),
                    Span::styled(
                        "1-9",
                        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " quick switch · ",
                        Style::default().fg(theme.popup_muted_text),
                    ),
                    Span::styled(
                        "Del",
                        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" clear all)", Style::default().fg(theme.popup_muted_text)),
                ]))
                .borders(Borders::NONE),
        )
}

fn render_status_line(frame: &mut Frame, area: Rect, state: &AppState, theme: Theme) {
    let status = Paragraph::new(city_picker_state_line(state, theme));
    frame.render_widget(status, area);
}

fn format_recent_location(index: usize, saved: &RecentLocation, state: &AppState) -> String {
    let timezone = saved.timezone.as_deref().unwrap_or("--");
    let region = match (&saved.country, &saved.admin1) {
        (Some(country), Some(admin)) => format!("{country} · {admin}"),
        (Some(country), None) => country.clone(),
        (None, Some(admin)) => admin.clone(),
        (None, None) => "--".to_string(),
    };
    let marker = if is_selected_location(saved, state) {
        "* "
    } else {
        ""
    };
    format!(
        "{}. {}{} · {} · TZ {} · {:.2}, {:.2}",
        index + 1,
        marker,
        saved.display_name(),
        region,
        timezone,
        saved.latitude,
        saved.longitude
    )
}

fn city_picker_state_line(state: &AppState, theme: Theme) -> Line<'static> {
    let muted = Style::default().fg(theme.popup_muted_text);
    let key = Style::default().fg(theme.text).add_modifier(Modifier::BOLD);

    let detail = state.city_status.as_deref().unwrap_or(
        "Enter search/switch · ↑/↓ history · Del clear all · Backspace edit · Esc close",
    );
    let kind = city_status_kind(detail);

    let mut spans = vec![Span::styled(format!("State: {kind} · "), muted)];

    if detail.contains("Enter search/switch") {
        spans.extend(vec![
            Span::styled("Enter", key),
            Span::styled(" search/switch · ", muted),
            Span::styled("↑/↓", key),
            Span::styled(" history · ", muted),
            Span::styled("Del", key),
            Span::styled(" clear all · ", muted),
            Span::styled("Backspace", key),
            Span::styled(" edit · ", muted),
            Span::styled("Esc", key),
            Span::styled(" close", muted),
        ]);
    } else {
        spans.push(Span::styled(format!("{detail} · "), muted));
        spans.push(Span::styled("Esc", key));
        spans.push(Span::styled(" close", muted));
    }

    Line::from(spans)
}

fn city_status_kind(status: &str) -> &'static str {
    if status.contains("Searching") {
        "Searching"
    } else if status.contains("No results") {
        "No results"
    } else if status.contains("Ambiguous") {
        "Ambiguous"
    } else if status.contains("Failed") {
        "Failed"
    } else {
        "Ready"
    }
}

fn is_selected_location(saved: &RecentLocation, state: &AppState) -> bool {
    state.selected_location.as_ref().is_some_and(|selected| {
        let selected_recent = RecentLocation::from_location(selected);
        saved.same_place(&selected_recent)
    })
}

#[cfg(test)]
mod tests;
