#![allow(clippy::match_same_arms)]

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

    let scene = select_scene(state, area, is_day);
    let tint = tint_color(scene.tint, theme);
    let has_title = area.height >= 6;
    let has_context = scene.context_line.is_some() && area.height >= 8;
    render_title_bar(frame, area, has_title, &scene.label, theme);
    render_scene_content(
        frame,
        area,
        &scene.lines,
        SceneRenderContext {
            has_title,
            has_context,
            theme,
            tint,
            visual: state.settings.hero_visual,
        },
    );
    render_context_bar(
        frame,
        area,
        has_context,
        scene.context_line.as_deref(),
        theme,
    );
}

#[derive(Clone, Copy)]
struct SceneRenderContext {
    has_title: bool,
    has_context: bool,
    theme: Theme,
    tint: Color,
    visual: HeroVisualArg,
}

fn render_title_bar(frame: &mut Frame, area: Rect, has_title: bool, label: &str, theme: Theme) {
    if !has_title {
        return;
    }
    let title_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    let hint = "‹V›";
    let label_max = area.width as usize - hint.len() - 2;
    let truncated_label: String = label.chars().take(label_max).collect();
    let pad = area
        .width
        .saturating_sub((truncated_label.chars().count() + hint.len()) as u16);
    let title_line = Line::from(vec![
        Span::styled(truncated_label, Style::default().fg(theme.muted_text)),
        Span::raw(" ".repeat(pad.saturating_sub(1) as usize)),
        Span::styled(hint.to_string(), Style::default().fg(theme.muted_text)),
    ]);
    frame.render_widget(Paragraph::new(Text::from(title_line)), title_area);
}

fn render_scene_content(
    frame: &mut Frame,
    area: Rect,
    scene_lines: &[String],
    ctx: SceneRenderContext,
) {
    let title_rows = u16::from(ctx.has_title);
    let context_rows = u16::from(ctx.has_context);
    let scene_area = Rect {
        x: area.x,
        y: area.y + title_rows,
        width: area.width,
        height: area.height.saturating_sub(title_rows + context_rows),
    };
    let lines = scene_lines
        .iter()
        .map(|line| colorize_scene_line(line, ctx.theme, ctx.tint, ctx.visual))
        .collect::<Vec<_>>();
    let text = Text::from(lines).patch_style(Style::default().fg(ctx.tint));
    frame.render_widget(Paragraph::new(text), scene_area);
}

fn render_context_bar(
    frame: &mut Frame,
    area: Rect,
    has_context: bool,
    context: Option<&str>,
    theme: Theme,
) {
    let Some(ctx_text) = context.filter(|_| has_context) else {
        return;
    };
    let context_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    let ctx_truncated: String = ctx_text.chars().take(area.width as usize).collect();
    let context_line = Line::from(Span::styled(
        ctx_truncated,
        Style::default().fg(theme.muted_text),
    ));
    frame.render_widget(Paragraph::new(Text::from(context_line)), context_area);
}

fn select_scene(
    state: &AppState,
    area: Rect,
    is_day: bool,
) -> crate::ui::widgets::landmark::LandmarkScene {
    let scene_area = (area.width.saturating_sub(2), area.height.saturating_sub(2));
    match state.settings.hero_visual {
        HeroVisualArg::AtmosCanvas => select_atmos_scene(state, area, is_day, scene_area),
        HeroVisualArg::GaugeCluster => select_gauge_scene(state, area, is_day, scene_area),
        HeroVisualArg::SkyObservatory => select_sky_scene(state, area, is_day, scene_area),
    }
}

fn select_atmos_scene(
    state: &AppState,
    area: Rect,
    is_day: bool,
    scene_area: (u16, u16),
) -> crate::ui::widgets::landmark::LandmarkScene {
    state.weather.as_ref().map_or_else(
        || loading_scene("Atmos Canvas", area.width, area.height, is_day),
        |bundle| {
            scene_for_weather(
                bundle,
                state.frame_tick,
                state.animate_ui,
                scene_area.0,
                scene_area.1,
            )
        },
    )
}

fn select_gauge_scene(
    state: &AppState,
    area: Rect,
    is_day: bool,
    scene_area: (u16, u16),
) -> crate::ui::widgets::landmark::LandmarkScene {
    state.weather.as_ref().map_or_else(
        || loading_scene("Gauge Cluster", area.width, area.height, is_day),
        |bundle| scene_for_gauge_cluster(bundle, scene_area.0, scene_area.1),
    )
}

fn select_sky_scene(
    state: &AppState,
    area: Rect,
    is_day: bool,
    scene_area: (u16, u16),
) -> crate::ui::widgets::landmark::LandmarkScene {
    state.weather.as_ref().map_or_else(
        || loading_scene("Sky Observatory", area.width, area.height, is_day),
        |bundle| {
            scene_for_sky_observatory(
                bundle,
                state.frame_tick,
                state.animate_ui,
                scene_area.0,
                scene_area.1,
            )
        },
    )
}

fn tint_color(tint: LandmarkTint, theme: Theme) -> Color {
    match tint {
        LandmarkTint::Warm => theme.landmark_warm,
        LandmarkTint::Cool => theme.landmark_cool,
        LandmarkTint::Neutral => theme.landmark_neutral,
    }
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
    if char_in(ch, &['█', '▅', '▃', '▁']) {
        theme.accent
    } else if char_in(ch, &['◉', '◐', 'o', '•']) {
        theme.warning
    } else if char_in(ch, &['v', 'V', '>', '=', '-', '/', '╱', '╲', '.', ',']) {
        theme.info
    } else if char_in(ch, &['░', '▒', '▓']) {
        theme.landmark_neutral
    } else if char_in(ch, &['❆', '*', '✦', '✧']) {
        theme.landmark_cool
    } else if char_in(ch, &['·', '∴', '─']) {
        theme.muted_text
    } else {
        base_tint
    }
}

fn scene_char_color_gauge(ch: char, theme: Theme, base_tint: Color) -> Color {
    if char_in(ch, &['█', '▓', '▒']) {
        theme.accent
    } else if char_in(ch, &['[', ']', '|', '+', '◉']) {
        theme.info
    } else if char_in(ch, &['↑', '→', '↓', '←', '↗', '↘', '↙', '↖']) {
        theme.warning
    } else if ch.is_ascii_digit() || matches!(ch, '%' | '.') {
        theme.text
    } else if ch.is_ascii_alphabetic() {
        theme.muted_text
    } else {
        base_tint
    }
}

fn char_in(ch: char, set: &[char]) -> bool {
    set.contains(&ch)
}

fn scene_char_color_sky(ch: char, theme: Theme, base_tint: Color) -> Color {
    if char_in(
        ch,
        &[
            '◉', '☀', '○', '◑', '◐', '◔', '◕', '◖', '◗', '●', '✶', '↑', '↓',
        ],
    ) {
        theme.warning
    } else if char_in(ch, &['█', '▓', '▒', '░']) {
        theme.info
    } else if char_in(ch, &['*', '·', '─', '~', '/', '╭', '╮']) {
        theme.landmark_cool
    } else if ch.is_ascii_digit() || ch == ':' {
        theme.text
    } else {
        base_tint
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
        context_line: None,
    }
}
