use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::{
    app::state::AppState,
    domain::weather::{Location, WeatherCategory},
    ui::theme::{detect_color_capability, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);
    let (category, is_day) =
        state
            .weather
            .as_ref()
            .map_or((WeatherCategory::Unknown, false), |w| {
                (
                    crate::domain::weather::weather_code_to_category(w.current.weather_code),
                    w.current.is_day,
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

    let items = state
        .pending_locations
        .iter()
        .enumerate()
        .map(|(idx, loc)| ListItem::new(format_selector_location(idx, loc)))
        .collect::<Vec<_>>();

    let list = List::new(items)
        .style(panel_style)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select location 1-5")
                .style(panel_style)
                .border_style(
                    Style::default()
                        .fg(theme.popup_border)
                        .bg(theme.popup_surface),
                ),
        );

    frame.render_widget(list, area);
}

fn format_selector_location(index: usize, location: &Location) -> String {
    let timezone = location.timezone.as_deref().unwrap_or("--");
    format!(
        "{}. {} · {:.2}, {:.2} · TZ {}",
        index + 1,
        location.display_name(),
        location.latitude,
        location.longitude,
        timezone
    )
}

#[cfg(test)]
mod tests {
    use super::format_selector_location;
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
}
