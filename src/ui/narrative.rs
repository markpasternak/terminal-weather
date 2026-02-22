use crate::{
    app::state::{AppState, PanelFocus},
    domain::weather::{ForecastBundle, InsightConfidence, derive_nowcast_insight},
    ui::symbols::{SemanticSymbol, symbol},
};

#[derive(Debug, Clone)]
pub struct UiNarrativeState {
    pub now_action: String,
    pub next_change: String,
    pub next_6h: String,
    pub reliability: String,
    pub confidence: InsightConfidence,
    pub confidence_symbol: String,
}

impl UiNarrativeState {
    #[must_use]
    pub fn compact_triage_line(&self, width: u16) -> String {
        if width < 48 {
            return self.now_action.clone();
        }
        let raw = format!(
            "{}  |  {}  |  {} {}",
            self.now_action, self.next_change, self.confidence_symbol, self.reliability
        );
        truncate_with_ellipsis(&raw, width as usize)
    }

    #[must_use]
    pub fn focus_hint(&self, panel: PanelFocus) -> String {
        match panel {
            PanelFocus::Hero => {
                format!("Decision now: {}  ·  {}", self.now_action, self.next_change)
            }
            PanelFocus::Hourly => {
                format!("Hourly focus: {}  ·  {}", self.next_6h, self.next_change)
            }
            PanelFocus::Daily => {
                format!("Week focus: {}  ·  {}", self.next_change, self.now_action)
            }
        }
    }
}

#[must_use]
pub fn build_narrative(state: &AppState, weather: &ForecastBundle) -> UiNarrativeState {
    let insight = derive_nowcast_insight(weather, state.units, &state.refresh_meta);
    let next_change = insight.next_change.map_or_else(
        || "Next change: none notable in next 24h".to_string(),
        |change| {
            format!(
                "Next change in {}h: {}",
                change.hours_from_now.max(1),
                change.message
            )
        },
    );

    let confidence_symbol = symbol(
        match insight.confidence {
            InsightConfidence::High => SemanticSymbol::ConfidenceHigh,
            InsightConfidence::Medium => SemanticSymbol::ConfidenceMedium,
            InsightConfidence::Low => SemanticSymbol::ConfidenceLow,
        },
        state.settings.icon_mode,
    )
    .to_string();

    UiNarrativeState {
        now_action: insight.action_text,
        next_change,
        next_6h: insight.next_6h_summary,
        reliability: insight.reliability.line(),
        confidence: insight.confidence,
        confidence_symbol,
    }
}

fn truncate_with_ellipsis(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = input.chars().count();
    if count <= max_chars {
        return input.to_string();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }
    let keep = max_chars - 3;
    let mut out = input.chars().take(keep).collect::<String>();
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::truncate_with_ellipsis;

    #[test]
    fn truncate_with_ellipsis_short_input_unchanged() {
        assert_eq!(truncate_with_ellipsis("abc", 5), "abc");
    }

    #[test]
    fn truncate_with_ellipsis_applies_marker() {
        assert_eq!(truncate_with_ellipsis("abcdefgh", 6), "abc...");
    }

    #[test]
    fn truncate_with_ellipsis_handles_tiny_width() {
        assert_eq!(truncate_with_ellipsis("abcdef", 2), "..");
    }
}
