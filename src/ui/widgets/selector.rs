use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::{
    app::state::AppState,
    domain::weather::{Location, WeatherCategory},
    ui::theme::{Theme, detect_color_capability, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);
    let (theme, panel_style) = selector_theme(state);

    let chunks = Layout::vertical([Constraint::Min(4), Constraint::Length(2)]).split(area);
    let list = selector_list(selector_items(state), panel_style, theme);
    frame.render_widget(list, chunks[0]);
    let state_line = selector_state_line(state, theme);
    frame.render_widget(Paragraph::new(state_line), chunks[1]);
}

fn selector_theme(state: &AppState) -> (crate::ui::theme::Theme, Style) {
    let (category, is_day) =
        state
            .weather
            .as_ref()
            .map_or((WeatherCategory::Unknown, false), |weather| {
                (
                    crate::domain::weather::weather_code_to_category(weather.current.weather_code),
                    weather.current.is_day,
                )
            });
    let theme = theme_for(
        category,
        is_day,
        detect_color_capability(state.color_mode),
        state.settings.theme,
    );
    let panel_style = Style::default()
        .fg(theme.popup_text)
        .bg(theme.popup_surface);
    (theme, panel_style)
}

fn selector_items(state: &AppState) -> Vec<ListItem<'static>> {
    let items = state
        .pending_locations
        .iter()
        .enumerate()
        .map(|(idx, location)| ListItem::new(format_selector_location(idx, location)))
        .collect::<Vec<_>>();
    if items.is_empty() {
        vec![ListItem::new("No candidate locations")]
    } else {
        items
    }
}

fn selector_list(
    items: Vec<ListItem<'static>>,
    panel_style: Style,
    theme: crate::ui::theme::Theme,
) -> List<'static> {
    List::new(items)
        .style(panel_style)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select location")
                .style(panel_style)
                .border_style(
                    Style::default()
                        .fg(theme.popup_border)
                        .bg(theme.popup_surface),
                ),
        )
}

fn format_selector_location(index: usize, location: &Location) -> String {
    let timezone = location.timezone.as_deref().unwrap_or("--");
    let population = location
        .population
        .map_or_else(|| "pop --".to_string(), |value| format!("pop {value}"));
    let region = match (&location.country, &location.admin1) {
        (Some(country), Some(admin)) => format!("{country} · {admin}"),
        (Some(country), None) => country.clone(),
        (None, Some(admin)) => admin.clone(),
        (None, None) => "--".to_string(),
    };
    format!(
        "{}. {} · {} · {} · TZ {} · {:.2}, {:.2}",
        index + 1,
        location.display_name(),
        region,
        population,
        timezone,
        location.latitude,
        location.longitude
    )
}

fn selector_state_line(state: &AppState, theme: Theme) -> Line<'static> {
    let muted = Style::default().fg(theme.popup_muted_text);
    let key = Style::default().fg(theme.text).add_modifier(Modifier::BOLD);

    if !state.pending_locations.is_empty() {
        return Line::from(vec![
            Span::styled("State: Ambiguous · choose ", muted),
            Span::styled("1-5", key),
            Span::styled(", or ", muted),
            Span::styled("Esc", key),
            Span::styled(" to refine search", muted),
        ]);
    }
    if state
        .city_status
        .as_deref()
        .is_some_and(|status| status.contains("No results"))
    {
        return Line::from(vec![
            Span::styled("State: No results · ", muted),
            Span::styled("Esc", key),
            Span::styled(" and try a broader city query", muted),
        ]);
    }
    if state.last_error.is_some() {
        return Line::from(vec![
            Span::styled("State: Failed · ", muted),
            Span::styled("Esc", key),
            Span::styled(" and retry search", muted),
        ]);
    }
    Line::from(vec![
        Span::styled("State: Ready · ", muted),
        Span::styled("Enter", key),
        Span::styled(" number to continue", muted),
    ])
}

#[cfg(test)]
mod tests {
    use super::{format_selector_location, selector_state_line};
    use crate::app::state::AppState;
    use crate::domain::weather::Location;

    #[test]
    fn selector_entry_includes_timezone_and_coordinates() {
        let location = Location {
            name: "Springfield".to_string(),
            latitude: 39.7990,
            longitude: -89.6440,
            country: Some("United States".to_string()),
            admin1: Some("Illinois".to_string()),
            timezone: Some("America/Chicago".to_string()),
            population: None,
        };

        let text = format_selector_location(0, &location);
        assert!(text.contains("39.80, -89.64"));
        assert!(text.contains("TZ America/Chicago"));
    }

    #[test]
    fn selector_state_line_ambiguous_when_candidates_present() {
        let mut state = AppState::new(&crate::test_support::state_test_cli());
        state
            .pending_locations
            .push(Location::from_coords(1.0, 2.0));
        let theme = crate::ui::theme::resolved_theme(&state);
        let line = selector_state_line(&state, theme);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("Ambiguous"));
    }
}
