use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    app::state::AppState,
    cli::{Cli, ColorArg},
    domain::weather::{WeatherCategory, weather_code_to_category},
    ui::theme::{detect_color_capability, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, cli: &Cli) {
    frame.render_widget(Clear, area);

    let (category, is_day) = state
        .weather
        .as_ref()
        .map(|w| {
            (
                weather_code_to_category(w.current.weather_code),
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
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(panel_style)
        .border_style(
            Style::default()
                .fg(theme.popup_border)
                .bg(theme.popup_surface),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled(
            "Global",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("q / Esc quit  |  Ctrl+C immediate quit"),
        Line::from("r refresh now  |  Ctrl+L force redraw"),
        Line::from("s settings  |  l city picker  |  f/c units  |  v hourly view"),
        Line::from("<-/-> scroll hourly  |  ? or F1 toggle help"),
        Line::from(""),
        Line::from(Span::styled(
            "Settings Panel",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("Up/Down select  |  Left/Right or Enter change"),
        Line::from("Enter on actions runs Refresh now / Close"),
        Line::from("Esc or s closes settings"),
        Line::from(""),
        Line::from(Span::styled(
            "City Picker",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("Type city + Enter search"),
        Line::from("1..9 quick switch recent city"),
        Line::from("Up/Down move  |  Del clear all  |  Esc close"),
        Line::from(""),
        Line::from(Span::styled(
            "Refresh & Status",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("Fresh: live data  |  Stale: retrying  |  Offline: fetch failed"),
        Line::from("Use r to retry immediately while stale/offline"),
        Line::from(""),
        Line::from(Span::styled(
            "Color Policy",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "Current mode: {}",
            color_mode_label(cli.effective_color_mode())
        )),
        Line::from("CLI: --color auto|always|never  |  --no-color"),
        Line::from("Auto mode honors NO_COLOR (non-empty) and TERM=dumb"),
        Line::from(""),
        Line::from(Span::styled(
            "Esc / ? / F1 closes this help",
            Style::default()
                .fg(theme.popup_muted_text)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    let text = Paragraph::new(lines)
        .style(panel_style)
        .wrap(Wrap { trim: true });
    frame.render_widget(text, inner);
}

fn color_mode_label(mode: ColorArg) -> &'static str {
    match mode {
        ColorArg::Auto => "auto",
        ColorArg::Always => "always",
        ColorArg::Never => "never",
    }
}
