#![allow(clippy::must_use_candidate)]

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::state::AppState,
    domain::alerts::{AlertSeverity, WeatherAlert},
    ui::theme::resolved_theme,
};

pub fn render(frame: &mut Frame, area: Rect, alerts: &[WeatherAlert], state: &AppState) {
    if alerts.is_empty() || area.height == 0 || area.width < 10 {
        return;
    }
    let theme = resolved_theme(state);

    let available_width = area.width as usize;
    let mut spans = Vec::new();
    let mut current_width = 0usize;

    for (idx, alert) in alerts.iter().enumerate() {
        if !push_alert_span(
            &mut spans,
            &mut current_width,
            available_width,
            idx,
            alert,
            theme,
        ) {
            break;
        }
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.surface_alt));
    frame.render_widget(paragraph, area);
}

fn alert_color(theme: crate::ui::theme::Theme, severity: AlertSeverity) -> ratatui::style::Color {
    match severity {
        AlertSeverity::Danger => theme.danger,
        AlertSeverity::Warning => theme.warning,
        AlertSeverity::Info => theme.info,
    }
}

fn push_alert_span(
    spans: &mut Vec<Span<'static>>,
    current_width: &mut usize,
    available_width: usize,
    index: usize,
    alert: &WeatherAlert,
    theme: crate::ui::theme::Theme,
) -> bool {
    let separator = if index > 0 { "  │  " } else { " " };
    let entry = format!("{}{} {}", separator, alert.icon, alert.message);
    let entry_width = entry.chars().count();
    if *current_width + entry_width > available_width && index > 0 {
        return false;
    }
    if index > 0 {
        spans.push(Span::styled("  │  ", Style::default().fg(theme.muted_text)));
    } else {
        spans.push(Span::styled(" ", Style::default()));
    }
    spans.push(Span::styled(
        format!("{} {}", alert.icon, alert.message),
        Style::default()
            .fg(alert_color(theme, alert.severity))
            .add_modifier(Modifier::BOLD),
    ));
    *current_width += entry_width;
    true
}

#[must_use]
pub fn alert_row_height(alerts: &[WeatherAlert]) -> u16 {
    u16::from(!alerts.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{alert_color, alert_row_height, push_alert_span};
    use crate::domain::alerts::{AlertSeverity, WeatherAlert};

    fn dummy_alert() -> WeatherAlert {
        WeatherAlert {
            icon: "⚡",
            message: "Test alert".to_string(),
            severity: AlertSeverity::Info,
        }
    }

    fn make_theme() -> crate::ui::theme::Theme {
        crate::ui::theme::resolved_theme(&crate::app::state::AppState::new(
            &crate::test_support::state_test_cli(),
        ))
    }

    #[test]
    fn alert_row_height_zero_for_empty() {
        assert_eq!(alert_row_height(&[]), 0);
    }

    #[test]
    fn alert_row_height_one_for_non_empty() {
        assert_eq!(alert_row_height(&[dummy_alert()]), 1);
        assert_eq!(alert_row_height(&[dummy_alert(), dummy_alert()]), 1);
    }

    #[test]
    fn push_alert_span_returns_false_on_width_overflow() {
        let theme = make_theme();
        let mut spans = Vec::new();
        let mut current_width = 0usize;
        let available_width = 5;
        let alert = WeatherAlert {
            icon: "⚡",
            message: "This is a very long alert message".to_string(),
            severity: AlertSeverity::Warning,
        };

        let result = push_alert_span(
            &mut spans,
            &mut current_width,
            available_width,
            1,
            &alert,
            theme,
        );
        assert!(!result);
    }

    #[test]
    fn push_alert_span_returns_true_when_fits() {
        let theme = make_theme();
        let mut spans = Vec::new();
        let mut current_width = 0usize;
        let available_width = 100;
        let alert = WeatherAlert {
            icon: "⚡",
            message: "Test".to_string(),
            severity: AlertSeverity::Warning,
        };

        let result = push_alert_span(
            &mut spans,
            &mut current_width,
            available_width,
            0,
            &alert,
            theme,
        );
        assert!(result);
    }

    #[test]
    fn alert_color_danger() {
        let theme = make_theme();
        let color = alert_color(theme, AlertSeverity::Danger);
        assert_eq!(color, theme.danger);
    }

    #[test]
    fn alert_color_warning() {
        let theme = make_theme();
        let color = alert_color(theme, AlertSeverity::Warning);
        assert_eq!(color, theme.warning);
    }

    #[test]
    fn alert_color_info() {
        let theme = make_theme();
        let color = alert_color(theme, AlertSeverity::Info);
        assert_eq!(color, theme.info);
    }
}
