use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::app::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);

    let items = state
        .pending_locations
        .iter()
        .enumerate()
        .map(|(idx, loc)| ListItem::new(format!("{}. {}", idx + 1, loc.display_name())))
        .collect::<Vec<_>>();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Select location 1-5"),
    );

    frame.render_widget(list, area);
}
