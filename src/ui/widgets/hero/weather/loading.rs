use super::*;

use crate::ui::animation::{MotionMode, UiMotionContext};

pub(super) fn render_loading_choreography(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: Theme,
    scale: HeroScale,
    motion: UiMotionContext,
) {
    let stage_idx = loading_stage_index(motion.elapsed_seconds);
    let spinner = loading_spinner(motion.elapsed_seconds, motion.motion_mode);
    let bar = indeterminate_bar(motion, loading_bar_width(scale));
    let mut lines = vec![
        Line::from(Span::styled(
            format!("{spinner} Building the weather scene"),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(loading_stage_spans(stage_idx, theme)),
        Line::from(Span::styled(bar, Style::default().fg(theme.info))),
        Line::from(Span::styled(
            state.loading_message.clone(),
            Style::default().fg(theme.text),
        )),
    ];
    append_loading_skeleton_lines(&mut lines, area, theme, motion);
    lines.push(Line::from(Span::styled(
        "Tip: press L for cities, S for settings, R to retry, Q to quit",
        Style::default().fg(theme.muted_text),
    )));
    frame.render_widget(Paragraph::new(lines), area);
}

fn loading_bar_width(scale: HeroScale) -> usize {
    match scale {
        HeroScale::Compact => 18,
        HeroScale::Standard => 24,
        HeroScale::Deluxe => 34,
    }
}

fn loading_stage_spans(stage_idx: usize, theme: Theme) -> Vec<Span<'static>> {
    let stage_labels = [
        "Shape the sky",
        "Pull weather layers",
        "Settle the dashboard",
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
    theme: Theme,
    motion: UiMotionContext,
) {
    if area.height < 9 {
        return;
    }
    let lane_width = usize::from(area.width).saturating_sub(4).clamp(18, 56);
    lines.push(Line::from(""));
    lines.push(loading_lane_line(
        "Sky    ",
        lane_width,
        theme.muted_text,
        theme.accent,
        motion,
        "sky",
    ));
    lines.push(loading_lane_line(
        "Front  ",
        lane_width,
        theme.muted_text,
        theme.info,
        motion,
        "front",
    ));
    lines.push(loading_lane_line(
        "Ground ",
        lane_width,
        theme.muted_text,
        theme.success,
        motion,
        "ground",
    ));
}

fn loading_lane_line(
    label: &'static str,
    width: usize,
    label_color: Color,
    value_color: Color,
    motion: UiMotionContext,
    lane: &str,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(label_color)),
        Span::styled(
            loading_lane(width, motion, lane),
            Style::default().fg(value_color),
        ),
    ])
}

fn loading_lane(width: usize, motion: UiMotionContext, lane: &str) -> String {
    if width == 0 {
        return String::new();
    }
    let seed = motion.lane(lane);
    let phase = motion.elapsed_seconds;
    (0..width)
        .map(|idx| {
            let idx_u64 = idx as u64;
            let pulse = seed.pulse(phase, lane_speed(lane), idx_u64);
            let crest = seed.pulse(phase, lane_speed(lane) * 0.45 + 0.08, idx_u64 + 100);
            if pulse > 0.78 {
                '█'
            } else if pulse > 0.64 {
                '▓'
            } else if crest > 0.62 {
                lane_glyph(lane, motion.motion_mode, seed, idx_u64)
            } else {
                '·'
            }
        })
        .collect()
}

fn lane_speed(lane: &str) -> f32 {
    match lane {
        "sky" => 0.55,
        "front" => 0.95,
        "ground" => 0.40,
        _ => 0.75,
    }
}

fn lane_glyph(
    lane: &str,
    mode: MotionMode,
    seed: crate::ui::animation::SeededMotion,
    idx: u64,
) -> char {
    match lane {
        "sky" => sky_lane_glyph(mode, seed, idx),
        "front" => front_lane_glyph(seed, idx),
        "ground" => ground_lane_glyph(seed, idx),
        _ => '·',
    }
}

fn sky_lane_glyph(mode: MotionMode, seed: crate::ui::animation::SeededMotion, idx: u64) -> char {
    if matches!(mode, MotionMode::Cinematic | MotionMode::Standard) && seed.unit(idx + 7) > 0.55 {
        '~'
    } else {
        '░'
    }
}

fn front_lane_glyph(seed: crate::ui::animation::SeededMotion, idx: u64) -> char {
    if seed.unit(idx + 11) > 0.55 {
        '╱'
    } else {
        '≈'
    }
}

fn ground_lane_glyph(seed: crate::ui::animation::SeededMotion, idx: u64) -> char {
    if seed.unit(idx + 19) > 0.68 {
        '▁'
    } else {
        '▒'
    }
}

fn loading_spinner(elapsed_seconds: f32, motion_mode: MotionMode) -> &'static str {
    const FRAMES: [&str; 8] = ["·", "◜", "◠", "◝", "◞", "◡", "◟", "◜"];
    if !motion_mode.allows_animation() {
        return "•";
    }
    FRAMES[((elapsed_seconds * 10.0) as usize) % FRAMES.len()]
}

fn loading_stage_index(elapsed_seconds: f32) -> usize {
    ((elapsed_seconds / 0.75) as usize) % 3
}

fn indeterminate_bar(motion: UiMotionContext, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let pulse = motion.lane("loading-bar");
    let chars = (0..width)
        .map(|idx| {
            let wave = pulse.pulse(motion.elapsed_seconds, 0.9, idx as u64);
            if wave > 0.84 {
                '█'
            } else if wave > 0.70 {
                '▓'
            } else if wave > 0.55 {
                '▒'
            } else {
                '·'
            }
        })
        .collect::<String>();
    format!("[{chars}]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::animation::{MotionMode, SeededMotion, UiMotionContext};

    fn test_motion(mode: MotionMode) -> UiMotionContext {
        UiMotionContext {
            elapsed_seconds: 1.2,
            dt_seconds: 0.04,
            frame_index: 4,
            motion_mode: mode,
            seed: SeededMotion::new(42),
            weather_profile: None,
            transition_progress: None,
            animate: mode.allows_animation(),
        }
    }

    #[test]
    fn loading_spinner_changes_with_motion_mode() {
        assert_eq!(loading_spinner(0.2, MotionMode::Off), "•");
        assert_ne!(
            loading_spinner(0.2, MotionMode::Cinematic),
            loading_spinner(0.6, MotionMode::Cinematic)
        );
    }

    #[test]
    fn loading_stage_index_cycles_over_time() {
        assert_eq!(loading_stage_index(0.0), 0);
        assert_eq!(loading_stage_index(0.8), 1);
        assert_eq!(loading_stage_index(1.6), 2);
    }

    #[test]
    fn indeterminate_bar_returns_empty_for_zero_width() {
        assert_eq!(
            indeterminate_bar(test_motion(MotionMode::Cinematic), 0),
            String::new()
        );
    }

    #[test]
    fn indeterminate_bar_contains_highlight_blocks() {
        let bar = indeterminate_bar(test_motion(MotionMode::Cinematic), 16);
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
        assert!(bar.contains('█') || bar.contains('▓'));
    }

    #[test]
    fn loading_lane_uses_lane_specific_texture() {
        let sky = loading_lane(24, test_motion(MotionMode::Cinematic), "sky");
        let front = loading_lane(24, test_motion(MotionMode::Cinematic), "front");
        assert_ne!(sky, front);
    }
}
