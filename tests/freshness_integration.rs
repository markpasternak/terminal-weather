use chrono::{Duration, Utc};

use atmos_tui::resilience::{
    backoff::Backoff,
    freshness::{FreshnessState, evaluate_freshness},
};

#[test]
fn backoff_progression_and_cap() {
    let mut backoff = Backoff::new(10, 300);
    assert_eq!(backoff.next_delay(), 10);
    assert_eq!(backoff.next_delay(), 20);
    assert_eq!(backoff.next_delay(), 40);
    assert_eq!(backoff.next_delay(), 80);
    assert_eq!(backoff.next_delay(), 160);
    assert_eq!(backoff.next_delay(), 300);
    assert_eq!(backoff.next_delay(), 300);

    backoff.reset();
    assert_eq!(backoff.next_delay(), 10);
}

#[test]
fn freshness_transitions_and_recovery() {
    let now = Utc::now();

    let fresh = evaluate_freshness(Some(now - Duration::minutes(5)), 0);
    assert_eq!(fresh, FreshnessState::Fresh);

    let stale_from_age = evaluate_freshness(Some(now - Duration::minutes(12)), 0);
    assert_eq!(stale_from_age, FreshnessState::Stale);

    let stale_from_failure = evaluate_freshness(Some(now - Duration::minutes(2)), 1);
    assert_eq!(stale_from_failure, FreshnessState::Stale);

    let offline_from_age = evaluate_freshness(Some(now - Duration::minutes(31)), 0);
    assert_eq!(offline_from_age, FreshnessState::Offline);

    let offline_from_failures = evaluate_freshness(Some(now - Duration::minutes(2)), 3);
    assert_eq!(offline_from_failures, FreshnessState::Offline);

    let recovered = evaluate_freshness(Some(now - Duration::minutes(1)), 0);
    assert_eq!(recovered, FreshnessState::Fresh);
}
