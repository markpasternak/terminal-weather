use super::*;
use ratatui::{Terminal, backend::TestBackend};

fn draw_week_summary(area: Rect, daily: Vec<DailyForecast>) {
    let bundle = sample_bundle_with_daily(daily);
    let theme = test_theme();
    let backend = TestBackend::new(area.width, area.height);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| render_week_summary(frame, area, &bundle, Units::Celsius, theme))
        .expect("draw");
}

#[test]
fn render_week_summary_returns_early_on_narrow_width() {
    draw_week_summary(Rect::new(0, 0, 15, 10), vec![sample_day(1.0, 10.0, 2.0)]);
}

#[test]
fn render_week_summary_returns_early_on_zero_height() {
    draw_week_summary(Rect::new(0, 0, 80, 0), vec![sample_day(1.0, 10.0, 2.0)]);
}

#[test]
fn render_week_summary_returns_early_on_empty_daily() {
    draw_week_summary(Rect::new(0, 0, 80, 10), vec![]);
}

#[test]
fn render_week_summary_hits_meta_and_sunrise_layout_paths() {
    let bundle = sample_bundle_with_daily(vec![sample_day(1.0, 10.0, 2.0)]);
    let theme = test_theme();

    let mut wide_terminal = Terminal::new(TestBackend::new(80, 10)).expect("terminal");
    wide_terminal
        .draw(|frame| {
            render_week_summary(
                frame,
                Rect::new(0, 0, 80, 10),
                &bundle,
                Units::Celsius,
                theme,
            );
        })
        .expect("draw");

    let mut medium_terminal = Terminal::new(TestBackend::new(48, 10)).expect("terminal");
    medium_terminal
        .draw(|frame| {
            render_week_summary(
                frame,
                Rect::new(0, 0, 48, 10),
                &bundle,
                Units::Celsius,
                theme,
            );
        })
        .expect("draw");
}
