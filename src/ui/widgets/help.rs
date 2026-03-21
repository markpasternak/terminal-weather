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
    for section in CORE_HELP_SECTIONS {
        push_section(&mut lines, theme, section.title, section.body);
    }
    append_key_reference_help(&mut lines, theme);
    append_color_policy_help(&mut lines, theme, color_mode);
    append_help_footer(&mut lines, theme);
    lines
}

struct HelpSection {
    title: &'static str,
    body: &'static [&'static str],
}

const CORE_HELP_SECTIONS: [HelpSection; 5] = [
    HelpSection {
        title: "Start here",
        body: &[
            "1) Read top-left triad: now action, next change, confidence/freshness",
            "2) Press Tab to focus Hourly or 7-Day for deeper context",
            "3) Use :city <name> or L to switch location quickly",
        ],
    },
    HelpSection {
        title: "Switch city",
        body: &[
            "Press L, type city, Enter search",
            "Use 1..9 for recent locations",
            "When ambiguous results appear, choose 1..5",
        ],
    },
    HelpSection {
        title: "Read risk fast",
        body: &[
            "Hero shows: now action + next change + confidence",
            "Hourly table adds cursor detail and next 6h summary",
            "Alerts include severity and ETA context",
        ],
    },
    HelpSection {
        title: "Fix stale/offline",
        body: &[
            "Watch status badge: fresh / stale / offline",
            "Press R to retry immediately",
            "Reliability lines show data age and retry timer",
        ],
    },
    HelpSection {
        title: "Customize visuals",
        body: &[
            "Open settings with S for theme, icons, and hourly view",
            "Use V to cycle hourly views quickly",
            "Type :theme <name> or :view <table|hybrid|chart>",
        ],
    },
];

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

fn push_section(
    lines: &mut Vec<Line<'static>>,
    theme: Theme,
    title: &'static str,
    body: &[&'static str],
) {
    lines.push(section_title_line(theme, title));
    lines.extend(body.iter().copied().map(Line::from));
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
