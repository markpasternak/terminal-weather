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

#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
pub(super) fn sparkline_blocks(values: &[f32], width: usize) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max - min).max(0.001);
    (0..width)
        .map(|idx| {
            let src = (idx * values.len() / width).min(values.len().saturating_sub(1));
            let norm = ((values[src] - min) / span).clamp(0.0, 1.0);
            BARS[(norm * (BARS.len() - 1) as f32).round() as usize]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sparkline_blocks;

    #[test]
    fn sparkline_blocks_empty_values_returns_empty() {
        assert_eq!(sparkline_blocks(&[], 8), String::new());
    }

    #[test]
    fn sparkline_blocks_zero_width_returns_empty() {
        assert_eq!(sparkline_blocks(&[1.0, 2.0], 0), String::new());
    }

    #[test]
    fn sparkline_blocks_uniform_values_returns_mid_bar() {
        // All same value → norm=0 everywhere → '▁'
        let out = sparkline_blocks(&[5.0, 5.0, 5.0], 3);
        assert_eq!(out.chars().count(), 3);
        assert!(out.chars().all(|c| c == '▁'), "got: {out}");
    }

    #[test]
    fn sparkline_blocks_rising_values_end_with_full_bar() {
        let values: Vec<f32> = (0u8..8).map(f32::from).collect();
        let out = sparkline_blocks(&values, 8);
        assert_eq!(out.chars().count(), 8);
        // Last position should be '█' (max)
        assert_eq!(out.chars().last().unwrap(), '█');
        // First position should be '▁' (min)
        assert_eq!(out.chars().next().unwrap(), '▁');
    }
}
