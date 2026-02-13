use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::{
    app::state::AppState,
    domain::weather::WeatherCategory,
    ui::theme::{detect_color_capability, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);
    let (category, is_day) = state
        .weather
        .as_ref()
        .map(|w| {
            (
                crate::domain::weather::weather_code_to_category(w.current.weather_code),
                w.current.is_day,
            )
        })
        .unwrap_or((WeatherCategory::Unknown, false));
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
        .map(|(idx, loc)| ListItem::new(format!("{}. {}", idx + 1, loc.display_name())))
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
