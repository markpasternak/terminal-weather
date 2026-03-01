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

    let start = ((state.frame_tick / 45) as usize) % alerts.len();
    for local_idx in 0..alerts.len() {
        let idx = (start + local_idx) % alerts.len();
        let alert = &alerts[idx];
        if !push_alert_span(
            &mut spans,
            &mut current_width,
            available_width,
            local_idx,
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
    let separator_width = separator.chars().count();
    if *current_width + separator_width >= available_width {
        return false;
    }
    let mut entry = format_alert_entry(alert);
    let remaining_after_separator =
        available_width.saturating_sub(*current_width + separator_width);
    if entry.chars().count() > remaining_after_separator {
        if index > 0 {
            return false;
        }
        truncate_alert_entry(&mut entry, remaining_after_separator);
    }
    let entry_width = entry.chars().count();
    spans.push(Span::styled(
        separator.to_string(),
        Style::default().fg(theme.muted_text),
    ));
    spans.push(Span::styled(
        entry,
        Style::default()
            .fg(alert_color(theme, alert.severity))
            .add_modifier(Modifier::BOLD),
    ));
    *current_width += separator_width + entry_width;
    true
}

fn format_alert_entry(alert: &WeatherAlert) -> String {
    let horizon = alert_horizon_label(alert.eta_hours);
    format!(
        "{} Do: {} · Why: {} · Details: timing {horizon}",
        alert.icon,
        alert_decision(alert),
        alert.message
    )
}

fn alert_horizon_label(eta_hours: Option<usize>) -> String {
    eta_hours.map_or_else(
        || "today".to_string(),
        |hours| {
            if hours == 0 {
                "now".to_string()
            } else {
                format!("in {hours}h")
            }
        },
    )
}

fn alert_decision(alert: &WeatherAlert) -> &'static str {
    let message = alert.message.to_ascii_lowercase();
    let keyword_table: &[(&[&str], &str)] = &[
        (&["freezing", "snow", "cold", "ice"], "Use winter gear"),
        (&["gust", "wind"], "Secure loose items"),
        (&["uv", "heat"], "Limit sun and heat load"),
        (&["visibility", "fog"], "Allow extra travel margin"),
        (&["thunder", "precip", "rain"], "Carry precipitation layer"),
    ];
    for (keywords, advice) in keyword_table {
        if keywords.iter().any(|kw| message.contains(kw)) {
            return advice;
        }
    }
    match alert.severity {
        AlertSeverity::Danger => "Act now",
        AlertSeverity::Warning => "Plan ahead",
        AlertSeverity::Info => "Stay aware",
    }
}

fn truncate_alert_entry(value: &mut String, max_chars: usize) {
    if value.chars().count() <= max_chars {
        return;
    }
    if max_chars <= 1 {
        value.clear();
        value.push('…');
        return;
    }
    if let Some((idx, _)) = value.char_indices().nth(max_chars - 1) {
        value.truncate(idx);
        value.push('…');
    }
}

#[must_use]
pub fn alert_row_height(alerts: &[WeatherAlert]) -> u16 {
    u16::from(!alerts.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{alert_color, alert_row_height, format_alert_entry, push_alert_span};
    use crate::domain::alerts::{AlertSeverity, WeatherAlert};

    fn dummy_alert() -> WeatherAlert {
        WeatherAlert {
            icon: "⚡",
            message: "Test alert".to_string(),
            eta_hours: Some(1),
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
            eta_hours: Some(2),
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
            eta_hours: Some(0),
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

    #[test]
    fn format_alert_entry_uses_decision_why_details_order() {
        let alert = WeatherAlert {
            icon: "≡",
            message: "Low visibility: 0.8km".to_string(),
            eta_hours: Some(2),
            severity: AlertSeverity::Warning,
        };

        let entry = format_alert_entry(&alert);
        assert!(entry.contains("Do: Allow extra travel margin"));
        assert!(entry.contains(" · Why: Low visibility: 0.8km"));
        assert!(entry.contains(" · Details: timing in 2h"));
    }

    #[test]
    fn format_alert_entry_uses_now_for_zero_eta() {
        let alert = WeatherAlert {
            icon: "⚡",
            message: "Thunderstorms expected".to_string(),
            eta_hours: Some(0),
            severity: AlertSeverity::Warning,
        };

        let entry = format_alert_entry(&alert);
        assert!(entry.ends_with("Details: timing now"));
    }

    #[test]
    fn test_truncate_alert_entry() {
        use super::truncate_alert_entry;

        let mut s = "Hello, world!".to_string();
        truncate_alert_entry(&mut s, 5);
        assert_eq!(s, "Hell…");

        let mut s = "A".to_string();
        truncate_alert_entry(&mut s, 5);
        assert_eq!(s, "A");

        let mut s = "Hello".to_string();
        truncate_alert_entry(&mut s, 1);
        assert_eq!(s, "…");

        let mut s = "Hello".to_string();
        truncate_alert_entry(&mut s, 0);
        assert_eq!(s, "…");

        let mut s = "This is a very long alert message".to_string();
        truncate_alert_entry(&mut s, 10);
        assert_eq!(s, "This is a…");

        let mut s = "⚡This is a very long alert message".to_string();
        truncate_alert_entry(&mut s, 10);
        assert_eq!(s, "⚡This is …");
    }
}
