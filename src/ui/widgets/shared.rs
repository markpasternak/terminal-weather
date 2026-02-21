use ratatui::{
    style::Style,
    widgets::{Block, Borders},
};

use crate::ui::theme::Theme;

pub(super) fn popup_panel_style(theme: Theme) -> Style {
    Style::default()
        .fg(theme.popup_text)
        .bg(theme.popup_surface)
}

pub(super) fn popup_block(title: &'static str, theme: Theme, panel_style: Style) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(panel_style)
        .border_style(
            Style::default()
                .fg(theme.popup_border)
                .bg(theme.popup_surface),
        )
}
