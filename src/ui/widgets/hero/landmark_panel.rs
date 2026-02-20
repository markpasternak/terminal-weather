use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::{
    app::state::AppState,
    cli::HeroVisualArg,
    ui::theme::Theme,
    ui::widgets::landmark::{
        LandmarkTint, scene_for_gauge_cluster, scene_for_sky_observatory, scene_for_weather,
    },
};

pub fn render_landmark(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    is_day: bool,
    theme: Theme,
) {
    if area.width < 10 || area.height < 4 {
        return;
    }

    let scene = match state.settings.hero_visual {
        HeroVisualArg::AtmosCanvas => state.weather.as_ref().map_or_else(
            || loading_scene("Atmos Canvas", area.width, area.height, is_day),
            |bundle| {
                scene_for_weather(
                    bundle,
                    state.frame_tick,
                    state.animate_ui,
                    area.width.saturating_sub(2),
                    area.height.saturating_sub(2),
                )
            },
        ),
        HeroVisualArg::GaugeCluster => state.weather.as_ref().map_or_else(
            || loading_scene("Gauge Cluster", area.width, area.height, is_day),
            |bundle| {
                scene_for_gauge_cluster(
                    bundle,
                    area.width.saturating_sub(2),
                    area.height.saturating_sub(2),
                )
            },
        ),
        HeroVisualArg::SkyObservatory => state.weather.as_ref().map_or_else(
            || loading_scene("Sky Observatory", area.width, area.height, is_day),
            |bundle| {
                scene_for_sky_observatory(
                    bundle,
                    state.frame_tick,
                    state.animate_ui,
                    area.width.saturating_sub(2),
                    area.height.saturating_sub(2),
                )
            },
        ),
    };

    let tint = match scene.tint {
        LandmarkTint::Warm => theme.landmark_warm,
        LandmarkTint::Cool => theme.landmark_cool,
        LandmarkTint::Neutral => theme.landmark_neutral,
    };
    let scene_lines = scene.lines;

    let mut lines = Vec::new();
    for line in scene_lines {
        lines.push(colorize_scene_line(
            &line,
            theme,
            tint,
            state.settings.hero_visual,
        ));
    }

    let text = Text::from(lines).patch_style(Style::default().fg(tint));
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

fn colorize_scene_line(
    line: &str,
    theme: Theme,
    base_tint: Color,
    visual: HeroVisualArg,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_color: Option<Color> = None;
    let mut current = String::new();

    for ch in line.chars() {
        let color = scene_char_color(ch, theme, base_tint, visual);
        if Some(color) != current_color {
            if !current.is_empty() {
                spans.push(Span::styled(
                    std::mem::take(&mut current),
                    Style::default().fg(current_color.unwrap_or(base_tint)),
                ));
            }
            current_color = Some(color);
        }
        current.push(ch);
    }

    if !current.is_empty() {
        spans.push(Span::styled(
            current,
            Style::default().fg(current_color.unwrap_or(base_tint)),
        ));
    }

    Line::from(spans)
}

fn scene_char_color(ch: char, theme: Theme, base_tint: Color, visual: HeroVisualArg) -> Color {
    match visual {
        HeroVisualArg::AtmosCanvas => scene_char_color_atmos(ch, theme, base_tint),
        HeroVisualArg::GaugeCluster => scene_char_color_gauge(ch, theme, base_tint),
        HeroVisualArg::SkyObservatory => scene_char_color_sky(ch, theme, base_tint),
    }
}

fn scene_char_color_atmos(ch: char, theme: Theme, base_tint: Color) -> Color {
    match ch {
        '█' | '▅' | '▃' | '▁' => theme.accent,
        '◉' | '◐' | 'o' | '•' => theme.warning,
        'v' | 'V' | '>' | '=' | '-' => theme.info,
        '░' | '▒' | '▓' => theme.landmark_neutral,
        '/' | '╱' | '╲' | '.' | ',' => theme.info,
        '❆' | '*' | '✦' | '✧' => theme.landmark_cool,
        '·' | '∴' | '─' => theme.muted_text,
        _ => base_tint,
    }
}

fn scene_char_color_gauge(ch: char, theme: Theme, base_tint: Color) -> Color {
    match ch {
        '█' | '▓' | '▒' => theme.accent,
        '[' | ']' | '|' | '+' | '◉' => theme.info,
        '↑' | '→' | '↓' | '←' | '↗' | '↘' | '↙' | '↖' => theme.warning,
        _ if ch.is_ascii_digit() || matches!(ch, '%' | '.') => theme.text,
        _ if ch.is_ascii_alphabetic() => theme.muted_text,
        _ => base_tint,
    }
}

fn scene_char_color_sky(ch: char, theme: Theme, base_tint: Color) -> Color {
    match ch {
        '◉' | '☀' | '○' | '◑' | '◐' | '◔' | '◕' | '◖' | '◗' | '●' | '✶' => {
            theme.warning
        }
        '█' | '▓' | '▒' | '░' => theme.info,
        '*' | '·' | 'E' | 'W' | '─' | '~' | '/' => theme.landmark_cool,
        _ if ch.is_ascii_digit() || ch == ':' => theme.text,
        _ => base_tint,
    }
}

fn loading_scene(
    name: &str,
    width: u16,
    height: u16,
    _is_day: bool,
) -> crate::ui::widgets::landmark::LandmarkScene {
    let mut lines = Vec::new();
    let h = height as usize;
    let w = width as usize;
    for _ in 0..h {
        lines.push("-".repeat(w));
    }
    if h >= 2 {
        lines[h / 2] = format!("{:^width$}", format!("Loading {name}..."), width = w);
    }
    crate::ui::widgets::landmark::LandmarkScene {
        label: format!("Loading {name}"),
        lines,
        tint: crate::ui::widgets::landmark::LandmarkTint::Neutral,
    }
}
