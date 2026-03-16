use super::*;

mod accumulator;
pub(super) mod utils;

use accumulator::WeekAccumulator;
use utils::{day_cue, first_day_shifted_time, first_day_time, profile_bar};

#[cfg(test)]
use accumulator::{
    average_duration, average_precip_hours, format_day_value_mm, format_day_value_mps,
    format_uv_peak, week_thermal_span,
};

pub(super) fn render_week_summary(
    frame: &mut Frame,
    area: Rect,
    bundle: &ForecastBundle,
    units: Units,
    theme: crate::ui::theme::Theme,
) {
    if area.width < 20 || area.height == 0 || bundle.daily.is_empty() {
        return;
    }

    let summary = summarize_week(bundle, units);
    let mut lines = week_summary_header_lines(&summary, theme);
    if area.width >= 64 {
        append_week_meta_line(&mut lines, bundle, theme);
    } else if area.width >= 38 {
        append_sunrise_sunset_line(&mut lines, bundle, theme);
    }
    let mut remaining_rows = (area.height as usize).saturating_sub(lines.len());
    append_week_profiles(&mut lines, &mut remaining_rows, area, &summary, theme);
    append_day_cues(&mut lines, &mut remaining_rows, bundle, theme);
    append_compact_profiles(&mut lines, remaining_rows, area, &summary, theme);
    frame.render_widget(Paragraph::new(lines), area);
}

#[derive(Debug, Default)]
pub(super) struct WeekSummaryData {
    pub(super) precip_total: f32,
    pub(super) rain_total: f32,
    pub(super) snow_total: f32,
    pub(super) avg_daylight: String,
    pub(super) avg_sun: String,
    pub(super) breeziest_txt: String,
    pub(super) wettest_txt: String,
    pub(super) comfort_best_txt: String,
    pub(super) precip_hours_avg: String,
    pub(super) uv_peak: String,
    pub(super) week_thermal: String,
    pub(super) highs: Vec<f32>,
    pub(super) precip: Vec<f32>,
    pub(super) gusts: Vec<f32>,
}

pub(super) fn summarize_week(bundle: &ForecastBundle, units: Units) -> WeekSummaryData {
    let mut accumulator = WeekAccumulator::default();
    for day in &bundle.daily {
        accumulator.ingest(day);
    }
    accumulator.finish(units, &bundle.daily)
}

fn week_summary_header_lines(
    summary: &WeekSummaryData,
    theme: crate::ui::theme::Theme,
) -> Vec<Line<'static>> {
    let mut lines = week_actionability_lines(summary, theme);
    lines.extend([
        week_totals_line(summary, theme),
        week_sun_line(summary, theme),
        week_extrema_line(summary, theme),
        week_precip_uv_line(summary, theme),
    ]);
    lines
}

fn week_actionability_lines(
    summary: &WeekSummaryData,
    theme: crate::ui::theme::Theme,
) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Highlights ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("Precip peak {}", summary.wettest_txt),
                Style::default().fg(theme.info),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Windiest {}", summary.breeziest_txt),
                Style::default().fg(theme.warning),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Comfort {}", summary.comfort_best_txt),
                Style::default().fg(theme.success),
            ),
        ]),
        Line::from(vec![
            Span::styled("Plan ", Style::default().fg(theme.muted_text)),
            Span::styled(
                actionability_summary(summary),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]),
    ]
}

fn actionability_summary(summary: &WeekSummaryData) -> &'static str {
    if summary.precip_total >= 20.0 {
        "Precip-heavy week: prioritize dry windows"
    } else if summary.breeziest_txt != "--" && summary.breeziest_txt.contains("m/s") {
        "Mixed week: track wind and UV day by day"
    } else {
        "Stable week: low planning friction"
    }
}

fn week_totals_line(summary: &WeekSummaryData, theme: crate::ui::theme::Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled("Totals ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("P {:.1}mm", summary.precip_total),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Rain ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{:.1}mm", summary.rain_total),
            Style::default().fg(theme.info),
        ),
        Span::raw("  "),
        Span::styled("Snow ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{:.1}cm", summary.snow_total),
            Style::default().fg(theme.temp_cold),
        ),
    ])
}

fn week_sun_line(summary: &WeekSummaryData, theme: crate::ui::theme::Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled("Avg daylight ", Style::default().fg(theme.muted_text)),
        Span::styled(
            summary.avg_daylight.clone(),
            Style::default().fg(theme.warning),
        ),
        Span::raw("  "),
        Span::styled("Avg sun ", Style::default().fg(theme.muted_text)),
        Span::styled(summary.avg_sun.clone(), Style::default().fg(theme.accent)),
    ])
}

fn week_extrema_line(summary: &WeekSummaryData, theme: crate::ui::theme::Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled("Breeziest ", Style::default().fg(theme.muted_text)),
        Span::styled(
            summary.breeziest_txt.clone(),
            Style::default().fg(theme.warning),
        ),
        Span::raw("  "),
        Span::styled("Precip peak ", Style::default().fg(theme.muted_text)),
        Span::styled(summary.wettest_txt.clone(), Style::default().fg(theme.info)),
    ])
}

fn week_precip_uv_line(summary: &WeekSummaryData, theme: crate::ui::theme::Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled("Avg precip hrs ", Style::default().fg(theme.muted_text)),
        Span::styled(
            summary.precip_hours_avg.clone(),
            Style::default().fg(theme.info),
        ),
        Span::raw("  "),
        Span::styled("Peak UV ", Style::default().fg(theme.muted_text)),
        Span::styled(summary.uv_peak.clone(), Style::default().fg(theme.warning)),
        Span::raw("  "),
        Span::styled("Week span ", Style::default().fg(theme.muted_text)),
        Span::styled(
            summary.week_thermal.clone(),
            Style::default().fg(theme.accent),
        ),
    ])
}

fn append_week_meta_line(
    lines: &mut Vec<Line<'static>>,
    bundle: &ForecastBundle,
    theme: crate::ui::theme::Theme,
) {
    let tz = bundle.location.timezone.as_deref().unwrap_or("--");
    let sunrise = first_day_time(bundle, |day| day.sunrise);
    let sunset = first_day_time(bundle, |day| day.sunset);
    let dawn = first_day_shifted_time(bundle, |day| day.sunrise, -30);
    let dusk = first_day_shifted_time(bundle, |day| day.sunset, 30);

    lines.push(Line::from(vec![
        Span::styled("Meta ", Style::default().fg(theme.muted_text)),
        Span::styled(format!("TZ {tz}"), Style::default().fg(theme.text)),
        Span::raw("  "),
        Span::styled(
            format!("Dawn {dawn} Sun {sunrise} Set {sunset} Dusk {dusk}"),
            Style::default().fg(theme.info),
        ),
        Span::raw("  "),
        Span::styled(
            bundle.location.name.clone(),
            Style::default().fg(theme.accent),
        ),
    ]));
}

fn append_sunrise_sunset_line(
    lines: &mut Vec<Line<'static>>,
    bundle: &ForecastBundle,
    theme: crate::ui::theme::Theme,
) {
    let sunrise = first_day_time(bundle, |day| day.sunrise);
    let sunset = first_day_time(bundle, |day| day.sunset);
    lines.push(Line::from(vec![
        Span::styled("Sunrise ", Style::default().fg(theme.muted_text)),
        Span::styled(sunrise, Style::default().fg(theme.warning)),
        Span::raw("  "),
        Span::styled("Sunset ", Style::default().fg(theme.muted_text)),
        Span::styled(sunset, Style::default().fg(theme.warning)),
    ]));
}

fn append_week_profiles(
    lines: &mut Vec<Line<'static>>,
    remaining_rows: &mut usize,
    area: Rect,
    summary: &WeekSummaryData,
    theme: crate::ui::theme::Theme,
) {
    if area.width < 72 || *remaining_rows < 4 {
        return;
    }

    lines.push(Line::from(Span::styled(
        "Week profiles",
        Style::default()
            .fg(theme.muted_text)
            .add_modifier(Modifier::BOLD),
    )));
    *remaining_rows = remaining_rows.saturating_sub(1);

    let profile_width = area.width.saturating_sub(28).clamp(10, 56) as usize;
    let specs = [
        ProfileLineSpec {
            label: "Temp arc    ",
            values: &summary.highs,
            suffix: summary.week_thermal.clone(),
            color: theme.accent,
        },
        ProfileLineSpec {
            label: "Precip lane ",
            values: &summary.precip,
            suffix: format!("{:.1}mm", summary.precip_total),
            color: theme.info,
        },
        ProfileLineSpec {
            label: "Wind lane   ",
            values: &summary.gusts,
            suffix: summary.breeziest_txt.clone(),
            color: theme.warning,
        },
    ];

    for spec in specs {
        append_profile_line(lines, remaining_rows, spec, profile_width, theme);
    }
}

struct ProfileLineSpec<'a> {
    label: &'static str,
    values: &'a [f32],
    suffix: String,
    color: Color,
}

fn append_profile_line(
    lines: &mut Vec<Line<'static>>,
    remaining_rows: &mut usize,
    spec: ProfileLineSpec<'_>,
    profile_width: usize,
    theme: crate::ui::theme::Theme,
) {
    if *remaining_rows == 0 || spec.values.is_empty() {
        return;
    }
    lines.push(Line::from(vec![
        Span::styled(spec.label, Style::default().fg(theme.muted_text)),
        Span::styled(
            profile_bar(spec.values, profile_width),
            Style::default().fg(spec.color),
        ),
        Span::raw(" "),
        Span::styled(spec.suffix, Style::default().fg(spec.color)),
    ]));
    *remaining_rows = remaining_rows.saturating_sub(1);
}

fn append_day_cues(
    lines: &mut Vec<Line<'static>>,
    remaining_rows: &mut usize,
    bundle: &ForecastBundle,
    theme: crate::ui::theme::Theme,
) {
    if *remaining_rows < 2 {
        return;
    }

    lines.push(Line::from(Span::styled(
        "Day cues",
        Style::default()
            .fg(theme.muted_text)
            .add_modifier(Modifier::BOLD),
    )));
    *remaining_rows = remaining_rows.saturating_sub(1);

    let cue_rows = (*remaining_rows).min(bundle.daily.len());
    for day in bundle.daily.iter().take(cue_rows) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>3} ", day.date.format("%a")),
                Style::default().fg(theme.muted_text),
            ),
            Span::styled(day_cue(day), Style::default().fg(theme.text)),
        ]));
    }
    *remaining_rows = remaining_rows.saturating_sub(cue_rows);
}

fn append_compact_profiles(
    lines: &mut Vec<Line<'static>>,
    remaining_rows: usize,
    area: Rect,
    summary: &WeekSummaryData,
    theme: crate::ui::theme::Theme,
) {
    if remaining_rows == 0 || area.width < 32 {
        return;
    }

    let profile_width = area.width.saturating_sub(18).clamp(8, 40) as usize;
    let mut slots_left = remaining_rows;
    let specs = [
        ("Temp profile ", &summary.highs, theme.accent),
        ("Precip lane  ", &summary.precip, theme.info),
        ("Wind lane    ", &summary.gusts, theme.warning),
    ];

    for (label, values, color) in specs {
        if slots_left == 0 || values.is_empty() {
            continue;
        }
        lines.push(Line::from(vec![
            Span::styled(label, Style::default().fg(theme.muted_text)),
            Span::styled(
                profile_bar(values, profile_width),
                Style::default().fg(color),
            ),
        ]));
        slots_left = slots_left.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests;
