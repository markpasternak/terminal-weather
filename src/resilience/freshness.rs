use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreshnessState {
    Fresh,
    Stale,
    Offline,
}

#[must_use]
pub fn evaluate_freshness(
    last_success: Option<DateTime<Utc>>,
    consecutive_failures: u32,
) -> FreshnessState {
    let Some(last_success) = last_success else {
        return if consecutive_failures >= 3 {
            FreshnessState::Offline
        } else {
            FreshnessState::Stale
        };
    };

    let age = Utc::now() - last_success;

    if age > Duration::minutes(30) || consecutive_failures >= 3 {
        FreshnessState::Offline
    } else if age > Duration::minutes(10) || consecutive_failures >= 1 {
        FreshnessState::Stale
    } else {
        FreshnessState::Fresh
    }
}
