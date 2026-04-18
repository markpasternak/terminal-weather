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
    append_core_help_sections(&mut lines, theme);
    append_key_reference_help(&mut lines, theme);
    append_color_policy_help(&mut lines, theme, color_mode);
    append_help_footer(&mut lines, theme);
    lines
}

fn append_core_help_sections(lines: &mut Vec<Line<'static>>, theme: Theme) {
    let key = Style::default().fg(theme.text).add_modifier(Modifier::BOLD);

    lines.push(section_title_line(theme, "Start here"));
    lines.push(Line::from(
        "1) Read top-left triad: now action, next change, confidence/freshness",
    ));
    lines.push(Line::from(vec![
        Span::raw("2) Press "),
        Span::styled("Tab", key),
        Span::raw(" to focus Hourly or 7-Day for deeper context"),
    ]));
    lines.push(Line::from(vec![
        Span::raw("3) Use "),
        Span::styled(":city <name>", key),
        Span::raw(" or "),
        Span::styled("L", key),
        Span::raw(" to switch location quickly"),
    ]));
    lines.push(Line::from(""));

    lines.push(section_title_line(theme, "Switch city"));
    lines.push(Line::from(vec![
        Span::raw("Press "),
        Span::styled("L", key),
        Span::raw(", type city, "),
        Span::styled("Enter", key),
        Span::raw(" search"),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Use "),
        Span::styled("1..9", key),
        Span::raw(" for recent locations"),
    ]));
    lines.push(Line::from(vec![
        Span::raw("When ambiguous results appear, choose "),
        Span::styled("1..5", key),
    ]));
    lines.push(Line::from(""));

    lines.push(section_title_line(theme, "Read risk fast"));
    lines.push(Line::from(
        "Hero shows: now action + next change + confidence",
    ));
    lines.push(Line::from(
        "Hourly table adds cursor detail and next 6h summary",
    ));
    lines.push(Line::from("Alerts include severity and ETA context"));
    lines.push(Line::from(""));

    lines.push(section_title_line(theme, "Fix stale/offline"));
    lines.push(Line::from("Watch status badge: fresh / stale / offline"));
    lines.push(Line::from(vec![
        Span::raw("Press "),
        Span::styled("R", key),
        Span::raw(" to retry immediately"),
    ]));
    lines.push(Line::from(
        "Reliability lines show data age and retry timer",
    ));
    lines.push(Line::from(""));

    lines.push(section_title_line(theme, "Customize visuals"));
    lines.push(Line::from(vec![
        Span::raw("Open settings with "),
        Span::styled("S", key),
        Span::raw(" for theme, icons, and hourly view"),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Use "),
        Span::styled("V", key),
        Span::raw(" to cycle hourly views quickly"),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Type "),
        Span::styled(":theme <name>", key),
        Span::raw(" or "),
        Span::styled(":view <table|hybrid|chart>", key),
    ]));
    lines.push(Line::from(""));
}

fn append_key_reference_help(lines: &mut Vec<Line<'static>>, theme: Theme) {
    lines.push(section_title_line(theme, "Key reference"));

    let key = Style::default().fg(theme.text).add_modifier(Modifier::BOLD);
    let muted = Style::default().fg(theme.popup_muted_text);

    lines.push(Line::from(vec![
        Span::styled("Q", key),
        Span::styled(" / ", muted),
        Span::styled("Esc", key),
        Span::styled(" quit  |  ", muted),
        Span::styled("Ctrl+C", key),
        Span::styled(" immediate quit", muted),
    ]));

    lines.push(Line::from(vec![
        Span::styled("R", key),
        Span::styled(" refresh now  |  ", muted),
        Span::styled("Ctrl+L", key),
        Span::styled(" force redraw", muted),
    ]));

    lines.push(Line::from(vec![
        Span::styled("S", key),
        Span::styled(" settings  |  ", muted),
        Span::styled("L", key),
        Span::styled(" city picker  |  ", muted),
        Span::styled("F/C", key),
        Span::styled(" units  |  ", muted),
        Span::styled("V", key),
        Span::styled(" hourly view", muted),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Tab", key),
        Span::styled(" / ", muted),
        Span::styled("Shift+Tab", key),
        Span::styled(" panel focus  |  ", muted),
        Span::styled(":", key),
        Span::styled(" command bar", muted),
    ]));

    lines.push(Line::from(""));
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
    let key = Style::default().fg(theme.text).add_modifier(Modifier::BOLD);
    let muted = Style::default().fg(theme.popup_muted_text);

    lines.push(Line::from(vec![
        Span::styled("Esc", key),
        Span::styled(" / ", muted),
        Span::styled("?", key),
        Span::styled(" / ", muted),
        Span::styled("F1", key),
        Span::styled(" closes this help", muted),
    ]));
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
