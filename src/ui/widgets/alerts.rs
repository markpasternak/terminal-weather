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
    domain::weather::{WeatherCategory, weather_code_to_category},
    ui::theme::{detect_color_capability, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, alerts: &[WeatherAlert], state: &AppState) {
    if alerts.is_empty() || area.height == 0 || area.width < 10 {
        return;
    }

    let capability = detect_color_capability(state.color_mode);
    let (category, is_day) =
        state
            .weather
            .as_ref()
            .map_or((WeatherCategory::Unknown, false), |w| {
                (
                    weather_code_to_category(w.current.weather_code),
                    w.current.is_day,
                )
            });
    let theme = theme_for(category, is_day, capability, state.settings.theme);

    let available_width = area.width as usize;
    let mut spans = Vec::new();
    let mut current_width = 0usize;

    for (idx, alert) in alerts.iter().enumerate() {
        let color = match alert.severity {
            AlertSeverity::Danger => theme.danger,
            AlertSeverity::Warning => theme.warning,
            AlertSeverity::Info => theme.info,
        };

        let separator = if idx > 0 { "  │  " } else { " " };
        let entry = format!("{}{} {}", separator, alert.icon, alert.message);
        let entry_width = entry.chars().count();

        if current_width + entry_width > available_width && idx > 0 {
            break;
        }

        if idx > 0 {
            spans.push(Span::styled("  │  ", Style::default().fg(theme.muted_text)));
        } else {
            spans.push(Span::styled(" ", Style::default()));
        }
        spans.push(Span::styled(
            format!("{} {}", alert.icon, alert.message),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
        current_width += entry_width;
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.surface_alt));
    frame.render_widget(paragraph, area);
}

#[must_use]
pub fn alert_row_height(alerts: &[WeatherAlert]) -> u16 {
    u16::from(!alerts.is_empty())
}
