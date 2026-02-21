use super::*;

#[test]
fn summarize_week_empty_daily_returns_defaults() {
    let bundle = sample_bundle_with_daily(vec![]);
    let summary = summarize_week(&bundle, Units::Celsius);
    assert_eq!(summary.avg_daylight, "--:--");
    assert_eq!(summary.avg_sun, "--:--");
    assert_eq!(summary.wettest_txt, "--");
    assert_eq!(summary.breeziest_txt, "--");
    assert_eq!(summary.uv_peak, "--");
    assert_eq!(summary.week_thermal, "--");
    assert!(summary.highs.is_empty());
}

#[test]
fn summarize_week_aggregates_single_day() {
    let bundle = sample_bundle_with_daily(vec![sample_day(3.5, 20.0, 4.5)]);
    let summary = summarize_week(&bundle, Units::Celsius);
    assert!((summary.precip_total - 3.5).abs() < f32::EPSILON);
    assert_eq!(summary.highs.len(), 1);
}

#[test]
fn summarize_week_null_precipitation_skipped() {
    let mut day = sample_day(0.0, 20.0, 3.0);
    day.precipitation_sum_mm = None;
    let bundle = sample_bundle_with_daily(vec![day]);
    let summary = summarize_week(&bundle, Units::Celsius);
    assert_eq!(summary.wettest_txt, "--");
}

#[test]
fn append_week_profiles_covers_guard_and_render_paths() {
    let bundle = sample_bundle_with_daily(vec![sample_day(1.0, 10.0, 2.0)]);
    let summary = summarize_week(&bundle, Units::Celsius);
    let theme = test_theme();

    let mut lines = Vec::new();
    let mut rows = 4;
    append_week_profiles(
        &mut lines,
        &mut rows,
        Rect::new(0, 0, 70, 10),
        &summary,
        theme,
    );
    assert!(lines.is_empty());

    lines.clear();
    rows = 3;
    append_week_profiles(
        &mut lines,
        &mut rows,
        Rect::new(0, 0, 80, 10),
        &summary,
        theme,
    );
    assert!(lines.is_empty());

    lines.clear();
    rows = 4;
    append_week_profiles(
        &mut lines,
        &mut rows,
        Rect::new(0, 0, 80, 10),
        &summary,
        theme,
    );
    assert_eq!(lines.len(), 4);
    assert_eq!(rows, 0);
}

fn append_profile_line_result(
    values: &[f32],
    mut rows: usize,
    theme: crate::ui::theme::Theme,
) -> (usize, usize) {
    let mut lines = Vec::new();
    append_profile_line(
        &mut lines,
        &mut rows,
        ProfileLineSpec {
            label: "L",
            values,
            suffix: "ok".to_string(),
            color: theme.text,
        },
        10,
        theme,
    );
    (lines.len(), rows)
}

#[test]
fn append_profile_line_returns_when_no_rows_left() {
    let theme = test_theme();
    let values = [1.0_f32];
    assert_eq!(append_profile_line_result(&values, 0, theme), (0, 0));
}

#[test]
fn append_profile_line_returns_when_values_empty() {
    let theme = test_theme();
    let empty: [f32; 0] = [];
    assert_eq!(append_profile_line_result(&empty, 1, theme), (0, 1));
}

#[test]
fn append_profile_line_renders_and_consumes_row_when_values_present() {
    let theme = test_theme();
    let values = [1.0_f32];
    assert_eq!(append_profile_line_result(&values, 1, theme), (1, 0));
}

#[test]
fn append_compact_profiles_covers_return_and_skip_paths() {
    let theme = test_theme();
    let summary = WeekSummaryData {
        precip_total: 2.0,
        breeziest_txt: "Mon 5m/s".to_string(),
        week_thermal: "1°..8°".to_string(),
        highs: vec![8.0],
        precip: vec![2.0],
        gusts: vec![5.0],
        ..WeekSummaryData::default()
    };

    let mut lines = Vec::new();
    append_compact_profiles(&mut lines, 0, Rect::new(0, 0, 40, 10), &summary, theme);
    assert!(lines.is_empty());

    append_compact_profiles(&mut lines, 2, Rect::new(0, 0, 20, 10), &summary, theme);
    assert!(lines.is_empty());

    append_compact_profiles(&mut lines, 1, Rect::new(0, 0, 40, 10), &summary, theme);
    assert_eq!(lines.len(), 1);

    let sparse = WeekSummaryData {
        gusts: vec![5.0],
        ..WeekSummaryData::default()
    };
    let mut sparse_lines = Vec::new();
    append_compact_profiles(
        &mut sparse_lines,
        2,
        Rect::new(0, 0, 40, 10),
        &sparse,
        theme,
    );
    assert_eq!(sparse_lines.len(), 1);
}
