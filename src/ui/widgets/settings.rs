use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, ListState, Paragraph},
};

use crate::{
    app::state::{AppState, SETTINGS_ORDER, SettingsSelection},
    ui::theme::{Theme, resolved_theme},
};

use super::shared::{popup_block, popup_panel_style};

const SAVE_SETTINGS_ERROR_FRAGMENT: &str = "save settings";

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);

    let theme = resolved_theme(state);
    let panel_style = popup_panel_style(theme);

    let block = popup_block("Settings", theme, panel_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Min(6),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(inner);

    let rows = settings_rows(state, theme);
    let selected_index = rows
        .iter()
        .position(|row| row.selection == Some(state.settings_selected))
        .unwrap_or(0);
    let items = rows
        .into_iter()
        .map(|row| row.item)
        .collect::<Vec<ListItem<'static>>>();

    let mut list_state = ListState::default().with_selected(Some(selected_index));
    let list = settings_list(items, panel_style, theme);
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    render_controls(frame, chunks[1], theme);
    render_hint(frame, chunks[2], state, theme);
}

struct SettingsRow {
    selection: Option<SettingsSelection>,
    item: ListItem<'static>,
}

fn settings_rows(state: &AppState, theme: Theme) -> Vec<SettingsRow> {
    let entries = state.settings_entries();
    let mut rows = Vec::new();
    let label_width = label_width(&entries);
    push_settings_section(
        &mut rows,
        &entries,
        "Visual",
        &[
            SettingsSelection::Theme,
            SettingsSelection::Motion,
            SettingsSelection::Flash,
            SettingsSelection::Icons,
            SettingsSelection::HeroVisual,
        ],
        label_width,
        theme,
    );
    push_settings_section(
        &mut rows,
        &entries,
        "Interaction",
        &[
            SettingsSelection::InlineHints,
            SettingsSelection::CommandBar,
            SettingsSelection::HourlyView,
        ],
        label_width,
        theme,
    );
    push_settings_section(
        &mut rows,
        &entries,
        "System",
        &[
            SettingsSelection::Units,
            SettingsSelection::RefreshInterval,
            SettingsSelection::RefreshNow,
            SettingsSelection::Close,
        ],
        label_width,
        theme,
    );
    rows
}

fn label_width(entries: &[crate::app::state::SettingsEntry]) -> usize {
    entries
        .iter()
        .map(|entry| entry.label.chars().count())
        .max()
        .unwrap_or(0)
}

fn push_settings_section(
    rows: &mut Vec<SettingsRow>,
    entries: &[crate::app::state::SettingsEntry],
    title: &str,
    selections: &[SettingsSelection],
    label_width: usize,
    theme: Theme,
) {
    push_section_header(rows, title, theme);
    for selection in selections {
        push_setting_row(rows, entries, *selection, label_width);
    }
}

#[cfg(test)]
fn settings_items(state: &AppState) -> Vec<ListItem<'static>> {
    settings_rows(state, resolved_theme(state))
        .into_iter()
        .map(|row| row.item)
        .collect::<Vec<_>>()
}

fn push_section_header(rows: &mut Vec<SettingsRow>, title: &str, theme: Theme) {
    rows.push(SettingsRow {
        selection: None,
        item: ListItem::new(Line::from(format!("-- {title} --"))).style(
            Style::default()
                .fg(theme.popup_muted_text)
                .add_modifier(Modifier::BOLD),
        ),
    });
}

fn push_setting_row(
    rows: &mut Vec<SettingsRow>,
    entries: &[crate::app::state::SettingsEntry],
    selection: SettingsSelection,
    label_width: usize,
) {
    let idx = selection_entry_index(selection);
    let Some(entry) = entries.get(idx) else {
        return;
    };
    let padded_label = format!("{:<width$}", entry.label, width = label_width);
    let line = if entry.editable {
        Line::from(vec![
            Span::raw(padded_label),
            Span::raw("  "),
            Span::raw(entry.value.clone()),
        ])
    } else {
        Line::from(vec![
            Span::raw(padded_label),
            Span::raw("  ["),
            Span::raw(entry.value.clone()),
            Span::raw("]"),
        ])
    };
    rows.push(SettingsRow {
        selection: Some(selection),
        item: ListItem::new(line),
    });
}

fn selection_entry_index(selection: SettingsSelection) -> usize {
    SETTINGS_ORDER
        .iter()
        .position(|candidate| *candidate == selection)
        .unwrap_or(0)
}

fn settings_list(items: Vec<ListItem<'static>>, panel_style: Style, theme: Theme) -> List<'static> {
    List::new(items)
        .style(panel_style)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ")
}

fn render_controls(frame: &mut Frame, area: Rect, theme: Theme) {
    let key_style = Style::default().fg(theme.text).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(theme.popup_muted_text);

    let controls = Paragraph::new(Line::from(vec![
        Span::styled("↑/↓", key_style),
        Span::styled(" select  ", desc_style),
        Span::styled("←/→", key_style),
        Span::styled(" or ", desc_style),
        Span::styled("Enter", key_style),
        Span::styled(" change  ", desc_style),
        Span::styled("Enter", key_style),
        Span::styled(" on actions  ", desc_style),
        Span::styled("S", key_style),
        Span::styled(" or ", desc_style),
        Span::styled("Esc", key_style),
        Span::styled(" close", desc_style),
    ]));
    frame.render_widget(controls, area);
}

fn render_hint(frame: &mut Frame, area: Rect, state: &AppState, theme: Theme) {
    let hint_text = settings_hint_text(state);
    let hint_style = settings_hint_style(&hint_text, theme);
    frame.render_widget(
        Paragraph::new(format!("Preview: {hint_text}")).style(hint_style),
        area,
    );
}

fn settings_hint_text(state: &AppState) -> String {
    state
        .last_error
        .as_deref()
        .filter(|error| has_save_settings_error(error))
        .map_or_else(|| state.settings_hint(), str::to_owned)
}

fn settings_hint_style(hint_text: &str, theme: Theme) -> Style {
    let color = if has_save_settings_error(hint_text) {
        theme.warning
    } else {
        theme.popup_muted_text
    };
    Style::default().fg(color)
}

fn has_save_settings_error(message: &str) -> bool {
    message.contains(SAVE_SETTINGS_ERROR_FRAGMENT)
}

#[cfg(test)]
mod tests;
