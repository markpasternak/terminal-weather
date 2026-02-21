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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_never_succeeded() {
        assert_eq!(evaluate_freshness(None, 0), FreshnessState::Stale);
        assert_eq!(evaluate_freshness(None, 1), FreshnessState::Stale);
        assert_eq!(evaluate_freshness(None, 2), FreshnessState::Stale);
        assert_eq!(evaluate_freshness(None, 3), FreshnessState::Offline);
        assert_eq!(evaluate_freshness(None, 100), FreshnessState::Offline);
    }

    #[test]
    fn test_fresh_state() {
        let now = Utc::now();
        // 5 minutes ago, 0 failures -> Fresh
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(5)), 0),
            FreshnessState::Fresh
        );
    }

    #[test]
    fn test_stale_state() {
        let now = Utc::now();
        // 15 minutes ago, 0 failures -> Stale
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(15)), 0),
            FreshnessState::Stale
        );
        // 5 minutes ago, 1 failure -> Stale
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(5)), 1),
            FreshnessState::Stale
        );
        // 5 minutes ago, 2 failures -> Stale
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(5)), 2),
            FreshnessState::Stale
        );
    }

    #[test]
    fn test_offline_state() {
        let now = Utc::now();
        // 35 minutes ago, 0 failures -> Offline
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(35)), 0),
            FreshnessState::Offline
        );
        // 5 minutes ago, 3 failures -> Offline
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(5)), 3),
            FreshnessState::Offline
        );
        // 35 minutes ago, 3 failures -> Offline
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(35)), 3),
            FreshnessState::Offline
        );
    }

    #[test]
    fn test_boundaries() {
        let now = Utc::now();

        // Just under 10 mins (e.g. 9 mins) -> Fresh
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(9)), 0),
            FreshnessState::Fresh
        );

        // Just over 10 mins (e.g. 11 mins) -> Stale
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(11)), 0),
            FreshnessState::Stale
        );

        // Just under 30 mins (e.g. 29 mins) -> Stale
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(29)), 0),
            FreshnessState::Stale
        );

        // Just over 30 mins (e.g. 31 mins) -> Offline
        assert_eq!(
            evaluate_freshness(Some(now - Duration::minutes(31)), 0),
            FreshnessState::Offline
        );
    }
}
