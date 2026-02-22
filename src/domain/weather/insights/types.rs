use crate::resilience::freshness::FreshnessState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsightConfidence {
    High,
    Medium,
    Low,
}

impl InsightConfidence {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::High => "●",
            Self::Medium => "◐",
            Self::Low => "○",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCue {
    CarryUmbrella,
    WinterTraction,
    SecureLooseItems,
    SunProtection,
    Hydrate,
    LayerUp,
    LowVisibility,
    Comfortable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    PrecipStart,
    WindIncrease,
    TempShift,
    ConditionShift,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeEvent {
    pub hours_from_now: usize,
    pub kind: ChangeKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReliabilitySummary {
    pub state: FreshnessState,
    pub age_minutes: Option<i64>,
    pub retry_in_seconds: Option<i64>,
    pub consecutive_failures: u32,
}

impl ReliabilitySummary {
    #[must_use]
    pub fn line(self) -> String {
        let state_label = match self.state {
            FreshnessState::Fresh => "fresh",
            FreshnessState::Stale => "stale",
            FreshnessState::Offline => "offline",
        };
        let age = self
            .age_minutes
            .map_or_else(|| "--".to_string(), |mins| format!("{}m", mins.max(0)));
        let retry = self
            .retry_in_seconds
            .map_or_else(|| "--".to_string(), |secs| format!("{secs}s"));

        format!("Data {state_label} · age {age} · retry {retry}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowcastInsight {
    pub action: ActionCue,
    pub action_text: String,
    pub next_change: Option<ChangeEvent>,
    pub reliability: ReliabilitySummary,
    pub confidence: InsightConfidence,
    pub next_6h_summary: String,
}
