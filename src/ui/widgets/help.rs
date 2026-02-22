use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

use crate::{
    app::state::AppState,
    cli::{Cli, ColorArg},
    ui::theme::{Theme, resolved_theme},
};

use super::shared::{popup_block, popup_panel_style};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, cli: &Cli) {
    frame.render_widget(Clear, area);

    let theme = resolved_theme(state);

    let panel_style = popup_panel_style(theme);
    let block = popup_block("Help", theme, panel_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = help_lines(theme, cli.effective_color_mode());

    let text = Paragraph::new(lines)
        .style(panel_style)
        .wrap(Wrap { trim: true });
    frame.render_widget(text, inner);
}

fn help_lines(theme: Theme, color_mode: ColorArg) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    append_start_here_help(&mut lines, theme);
    append_switch_city_help(&mut lines, theme);
    append_read_risk_help(&mut lines, theme);
    append_recover_data_help(&mut lines, theme);
    append_customize_visuals_help(&mut lines, theme);
    append_key_reference_help(&mut lines, theme);
    append_color_policy_help(&mut lines, theme, color_mode);
    append_help_footer(&mut lines, theme);
    lines
}

fn append_start_here_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "Start here",
        [
            "1) Read top-left triad: now action, next change, confidence/freshness",
            "2) Press Tab to focus Hourly or 7-Day for deeper context",
            "3) Use :city <name> or l to switch location quickly",
        ],
    );
}

fn append_switch_city_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "Switch city",
        [
            "Press l, type city, Enter search",
            "Use 1..9 for recent locations",
            "When ambiguous results appear, choose 1..5",
        ],
    );
}

fn append_read_risk_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "Read risk fast",
        [
            "Hero shows: now action + next change + confidence",
            "Hourly table adds cursor detail and next 6h summary",
            "Alerts include severity and ETA context",
        ],
    );
}

fn append_recover_data_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "Fix stale/offline",
        [
            "Watch status badge: fresh / stale / offline",
            "Press r to retry immediately",
            "Reliability lines show data age and retry timer",
        ],
    );
}

fn append_customize_visuals_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "Customize visuals",
        [
            "Open settings with s for theme, icons, and hourly view",
            "Use v to cycle hourly views quickly",
            "Type :theme <name> or :view <table|hybrid|chart>",
        ],
    );
}

fn append_key_reference_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    push_section(
        lines,
        theme,
        "Key reference",
        [
            "q / Esc quit  |  Ctrl+C immediate quit",
            "r refresh now  |  Ctrl+L force redraw",
            "s settings  |  l city picker  |  f/c units  |  v hourly view",
            "Tab / Shift+Tab panel focus  |  : command bar",
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
