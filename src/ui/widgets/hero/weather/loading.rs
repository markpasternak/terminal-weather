use super::*;

pub(super) fn render_loading_choreography(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: Theme,
    scale: HeroScale,
) {
    let stage_idx = loading_stage_index(state.frame_tick);
    let spinner = loading_spinner(state.frame_tick);
    let bar = indeterminate_bar(
        state.frame_tick,
        match scale {
            HeroScale::Compact => 18,
            HeroScale::Standard => 24,
            HeroScale::Deluxe => 34,
        },
    );
    let mut lines = vec![
        Line::from(Span::styled(
            format!("{spinner} Preparing atmosphere"),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(loading_stage_spans(stage_idx, theme)),
        Line::from(Span::styled(bar, Style::default().fg(theme.info))),
        Line::from(Span::styled(
            state.loading_message.clone(),
            Style::default().fg(theme.text),
        )),
    ];
    append_loading_skeleton_lines(&mut lines, area, state.frame_tick, theme);
    lines.push(Line::from(Span::styled(
        "Tip: press l for cities, s for settings, r to retry, q to quit",
        Style::default().fg(theme.muted_text),
    )));
    frame.render_widget(Paragraph::new(lines), area);
}

fn loading_stage_spans(stage_idx: usize, theme: Theme) -> Vec<Span<'static>> {
    let stage_labels = [
        "Locate city context",
        "Fetch weather layers",
        "Compose ambient scene",
    ];
    let mut spans = Vec::new();
    for (idx, label) in stage_labels.into_iter().enumerate() {
        let (marker, color) = loading_stage_marker_color(idx, stage_idx, theme);
        spans.push(Span::styled(marker, Style::default().fg(color)));
        spans.push(Span::styled(label, Style::default().fg(color)));
        if idx + 1 < stage_labels.len() {
            spans.push(Span::raw("   "));
        }
    }
    spans
}

fn loading_stage_marker_color(idx: usize, stage_idx: usize, theme: Theme) -> (&'static str, Color) {
    if idx < stage_idx {
        ("● ", theme.success)
    } else if idx == stage_idx {
        ("◉ ", theme.accent)
    } else {
        ("○ ", theme.muted_text)
    }
}

fn append_loading_skeleton_lines(
    lines: &mut Vec<Line<'static>>,
    area: Rect,
    frame_tick: u64,
    theme: Theme,
) {
    if area.height < 9 {
        return;
    }
    let skeleton_width = usize::from(area.width).saturating_sub(4).clamp(16, 56);
    lines.push(Line::from(""));
    lines.push(loading_skeleton_line(
        "Hero   ",
        frame_tick,
        skeleton_width,
        0,
        theme.muted_text,
        theme.accent,
    ));
    lines.push(loading_skeleton_line(
        "Hourly ",
        frame_tick,
        skeleton_width,
        1,
        theme.muted_text,
        theme.info,
    ));
    lines.push(loading_skeleton_line(
        "Daily  ",
        frame_tick,
        skeleton_width,
        2,
        theme.muted_text,
        theme.success,
    ));
}

fn loading_skeleton_line(
    label: &'static str,
    frame_tick: u64,
    skeleton_width: usize,
    row: usize,
    label_color: Color,
    value_color: Color,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(label_color)),
        Span::styled(
            loading_skeleton_row(frame_tick, skeleton_width, row),
            Style::default().fg(value_color),
        ),
    ])
}

fn loading_skeleton_row(frame_tick: u64, width: usize, lane: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut chars = vec!['·'; width];
    let head = ((frame_tick as usize) + lane * 5) % width;
    chars[head] = '█';
    if head > 0 {
        chars[head - 1] = '▓';
    }
    if head + 1 < width {
        chars[head + 1] = '▓';
    }
    if head + 2 < width {
        chars[head + 2] = '▒';
    }
    chars.into_iter().collect()
}

fn loading_spinner(frame_tick: u64) -> &'static str {
    const FRAMES: [&str; 8] = ["-", "\\", "|", "/", "-", "\\", "|", "/"];
    FRAMES[(frame_tick as usize) % FRAMES.len()]
}

fn loading_stage_index(frame_tick: u64) -> usize {
    ((frame_tick / 14) as usize) % 3
}

fn indeterminate_bar(frame_tick: u64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut chars = vec!['·'; width];
    let head = (frame_tick as usize) % width;
    chars[head] = '█';
    if head > 0 {
        chars[head - 1] = '▓';
    }
    if head + 1 < width {
        chars[head + 1] = '▓';
    }
    format!("[{}]", chars.into_iter().collect::<String>())
}
