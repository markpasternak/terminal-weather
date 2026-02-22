use super::*;
use chrono::{Duration, Utc};

#[test]
fn retry_countdown_is_clamped_at_zero() {
    let now = Utc::now();
    let metadata = RefreshMetadata {
        next_retry_at: Some(now - Duration::seconds(5)),
        ..RefreshMetadata::default()
    };

    assert_eq!(metadata.retry_in_seconds_at(now), Some(0));
}

#[test]
fn refresh_metadata_mark_success_clears_failure_state() {
    let mut meta = RefreshMetadata::default();
    meta.mark_failure();
    assert_eq!(meta.consecutive_failures, 1);
    meta.mark_success();
    assert_eq!(meta.consecutive_failures, 0);
    assert!(meta.last_success.is_some());
}

#[test]
fn refresh_metadata_schedule_retry_sets_next_retry() {
    let mut meta = RefreshMetadata::default();
    meta.schedule_retry_in(30);
    assert!(meta.next_retry_at.is_some());
}
