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
    ui::theme::{Theme, detect_color_capability, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, cli: &Cli) {
    frame.render_widget(Clear, area);

    let theme = help_theme(state);

    let panel_style = Style::default()
        .fg(theme.popup_text)
        .bg(theme.popup_surface);
    let block = help_block(theme, panel_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = help_lines(theme, cli.effective_color_mode());

    let text = Paragraph::new(lines)
        .style(panel_style)
        .wrap(Wrap { trim: true });
    frame.render_widget(text, inner);
}

fn help_theme(state: &AppState) -> Theme {
    let (category, is_day) =
        state
            .weather
            .as_ref()
            .map_or((WeatherCategory::Unknown, false), |w| {
                (
                    weather_code_to_category(w.current.weather_code),
                    w.current.is_day,
                )
            });
    theme_for(
        category,
        is_day,
        detect_color_capability(state.color_mode),
        state.settings.theme,
    )
}

fn help_block(theme: Theme, panel_style: Style) -> Block<'static> {
    Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(panel_style)
        .border_style(
            Style::default()
                .fg(theme.popup_border)
                .bg(theme.popup_surface),
        )
}

fn help_lines(theme: Theme, color_mode: ColorArg) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    append_global_help(&mut lines, theme);
    append_settings_help(&mut lines, theme);
    append_city_picker_help(&mut lines, theme);
    append_freshness_help(&mut lines, theme);
    append_color_policy_help(&mut lines, theme, color_mode);
    append_help_footer(&mut lines, theme);
    lines
}

fn append_global_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "Global",
        [
            "q / Esc quit  |  Ctrl+C immediate quit",
            "r refresh now  |  Ctrl+L force redraw",
            "s settings  |  l city picker  |  f/c units  |  v hourly view",
            "<-/-> scroll hourly  |  ? or F1 toggle help",
        ],
    );
}

fn append_settings_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "Settings Panel",
        [
            "Up/Down select  |  Left/Right or Enter change",
            "Enter on actions runs Refresh now / Close",
            "Esc or s closes settings",
        ],
    );
}

fn append_city_picker_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "City Picker",
        [
            "Type city + Enter search",
            "1..9 quick switch recent city",
            "Up/Down move  |  Del clear all  |  Esc close",
        ],
    );
}

fn append_freshness_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "Refresh & Status",
        [
            "Fresh: live data  |  Stale: retrying  |  Offline: fetch failed",
            "Use r to retry immediately while stale/offline",
        ],
    );
}

fn append_color_policy_help(lines: &mut Vec<Line<'static>>, theme: Theme, color_mode: ColorArg) {
    lines.push(section_title_line(theme, "Color Policy"));
    lines.push(Line::from(format!(
        "Current mode: {}",
        color_mode_label(color_mode)
    )));
    lines.push(Line::from("CLI: --color auto|always|never  |  --no-color"));
    lines.push(Line::from(
        "Auto mode honors NO_COLOR (non-empty) and TERM=dumb",
    ));
    lines.push(Line::from(""));
}

fn append_help_footer(lines: &mut Vec<Line<'static>>, theme: Theme) {
    lines.push(Line::from(Span::styled(
        "Esc / ? / F1 closes this help",
        Style::default()
            .fg(theme.popup_muted_text)
            .add_modifier(Modifier::BOLD),
    )));
}

fn push_section<const N: usize>(
    lines: &mut Vec<Line<'static>>,
    theme: Theme,
    title: &'static str,
    body: [&'static str; N],
) {
    lines.push(section_title_line(theme, title));
    lines.extend(body.into_iter().map(Line::from));
    lines.push(Line::from(""));
}

fn section_title_line(theme: Theme, title: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        title,
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))
}

fn color_mode_label(mode: ColorArg) -> &'static str {
    match mode {
        ColorArg::Auto => "auto",
        ColorArg::Always => "always",
        ColorArg::Never => "never",
    }
}
