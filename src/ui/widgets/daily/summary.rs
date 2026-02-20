use super::*;

mod utils;

use utils::{format_duration_hm, gust_cue, precipitation_cue, profile_bar, sunlight_cue};

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
    pub(super) precip_hours_avg: String,
    pub(super) uv_peak: String,
    pub(super) week_thermal: String,
    pub(super) highs: Vec<f32>,
    pub(super) precip: Vec<f32>,
    pub(super) gusts: Vec<f32>,
}

#[derive(Debug, Default)]
struct WeekAccumulator {
    precip_total: f32,
    rain_total: f32,
    snow_total: f32,
    daylight_total: f32,
    sunshine_total: f32,
    daylight_count: usize,
    sunshine_count: usize,
    precipitation_hours_total: f32,
    precipitation_hours_count: usize,
    breeziest: Option<(String, f32)>,
    wettest: Option<(String, f32)>,
    strongest_uv: Option<(String, f32)>,
    week_min_temp_c: Option<f32>,
    week_max_temp_c: Option<f32>,
}

impl WeekAccumulator {
    fn ingest(&mut self, day: &DailyForecast) {
        self.ingest_precipitation(day);
        Self::add_non_negative_with_count(
            &mut self.daylight_total,
            &mut self.daylight_count,
            day.daylight_duration_s,
        );
        Self::add_non_negative_with_count(
            &mut self.sunshine_total,
            &mut self.sunshine_count,
            day.sunshine_duration_s,
        );
        Self::update_tagged_max(&mut self.breeziest, day, day.wind_gusts_10m_max);
        Self::update_tagged_max(&mut self.strongest_uv, day, day.uv_index_max);
        Self::update_min(&mut self.week_min_temp_c, day.temperature_min_c);
        Self::update_max(&mut self.week_max_temp_c, day.temperature_max_c);
    }

    fn ingest_precipitation(&mut self, day: &DailyForecast) {
        if let Some(v) = day.precipitation_sum_mm {
            self.precip_total += v.max(0.0);
            Self::update_tagged_max(&mut self.wettest, day, Some(v));
        }
        Self::add_non_negative(&mut self.rain_total, day.rain_sum_mm);
        Self::add_non_negative(&mut self.snow_total, day.snowfall_sum_cm);
        Self::add_non_negative_with_count(
            &mut self.precipitation_hours_total,
            &mut self.precipitation_hours_count,
            day.precipitation_hours,
        );
    }

    fn add_non_negative(total: &mut f32, value: Option<f32>) {
        if let Some(v) = value {
            *total += v.max(0.0);
        }
    }

    fn add_non_negative_with_count(total: &mut f32, count: &mut usize, value: Option<f32>) {
        if let Some(v) = value {
            *total += v.max(0.0);
            *count += 1;
        }
    }

    fn update_tagged_max(
        slot: &mut Option<(String, f32)>,
        day: &DailyForecast,
        value: Option<f32>,
    ) {
        if let Some(v) = value
            && slot.as_ref().is_none_or(|(_, best)| v > *best)
        {
            *slot = Some((day.date.format("%a").to_string(), v));
        }
    }

    fn update_min(slot: &mut Option<f32>, value: Option<f32>) {
        if let Some(v) = value {
            *slot = Some(slot.map_or(v, |current| current.min(v)));
        }
    }

    fn update_max(slot: &mut Option<f32>, value: Option<f32>) {
        if let Some(v) = value {
            *slot = Some(slot.map_or(v, |current| current.max(v)));
        }
    }

    fn finish(self, units: Units, daily: &[DailyForecast]) -> WeekSummaryData {
        let wettest_txt = format_day_value_mm(self.wettest);
        let breeziest_txt = format_day_value_mps(self.breeziest);
        let avg_daylight = average_duration(self.daylight_total, self.daylight_count);
        let avg_sun = average_duration(self.sunshine_total, self.sunshine_count);
        let precip_hours_avg = average_precip_hours(
            self.precipitation_hours_total,
            self.precipitation_hours_count,
        );
        let uv_peak = format_uv_peak(self.strongest_uv);
        let week_thermal = week_thermal_span(self.week_min_temp_c, self.week_max_temp_c, units);
        let highs = collect_highs(daily, units);
        let precip = collect_precip(daily);
        let gusts = collect_gusts(daily);

        WeekSummaryData {
            precip_total: self.precip_total,
            rain_total: self.rain_total,
            snow_total: self.snow_total,
            avg_daylight,
            avg_sun,
            breeziest_txt,
            wettest_txt,
            precip_hours_avg,
            uv_peak,
            week_thermal,
            highs,
            precip,
            gusts,
        }
    }
}

fn format_day_value_mm(value: Option<(String, f32)>) -> String {
    value.map_or_else(|| "--".to_string(), |(day, mm)| format!("{day} {mm:.1}mm"))
}

fn format_day_value_mps(value: Option<(String, f32)>) -> String {
    value.map_or_else(
        || "--".to_string(),
        |(day, speed)| {
            format!(
                "{day} {}m/s",
                crate::domain::weather::round_wind_speed(speed)
            )
        },
    )
}

fn average_duration(total_seconds: f32, count: usize) -> String {
    if count > 0 {
        format_duration_hm(total_seconds / count as f32)
    } else {
        "--:--".to_string()
    }
}

fn average_precip_hours(total_hours: f32, count: usize) -> String {
    if count > 0 {
        format!("{:.1}h/day", total_hours / count as f32)
    } else {
        "--".to_string()
    }
}

fn format_uv_peak(value: Option<(String, f32)>) -> String {
    value.map_or_else(|| "--".to_string(), |(day, uv)| format!("{day} {uv:.1}"))
}

fn week_thermal_span(min_c: Option<f32>, max_c: Option<f32>, units: Units) -> String {
    match (min_c, max_c) {
        (Some(low), Some(high)) => {
            let low = round_temp(convert_temp(low, units));
            let high = round_temp(convert_temp(high, units));
            format!("{low}°..{high}°")
        }
        _ => "--".to_string(),
    }
}

fn collect_highs(daily: &[DailyForecast], units: Units) -> Vec<f32> {
    daily
        .iter()
        .filter_map(|d| d.temperature_max_c)
        .map(|t| convert_temp(t, units))
        .collect::<Vec<_>>()
}

fn collect_precip(daily: &[DailyForecast]) -> Vec<f32> {
    daily
        .iter()
        .map(|d| d.precipitation_sum_mm.unwrap_or(0.0))
        .collect::<Vec<_>>()
}

fn collect_gusts(daily: &[DailyForecast]) -> Vec<f32> {
    daily
        .iter()
        .map(|d| crate::domain::weather::convert_wind_speed(d.wind_gusts_10m_max.unwrap_or(0.0)))
        .collect::<Vec<_>>()
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
    vec![
        week_totals_line(summary, theme),
        week_sun_line(summary, theme),
        week_extrema_line(summary, theme),
        week_precip_uv_line(summary, theme),
    ]
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
        Span::styled("Wettest ", Style::default().fg(theme.muted_text)),
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
    let tz = bundle
        .location
        .timezone
        .as_deref()
        .unwrap_or("--")
        .to_string();
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

fn first_day_time(
    bundle: &ForecastBundle,
    projection: impl Fn(&DailyForecast) -> Option<chrono::NaiveDateTime>,
) -> String {
    bundle.daily.first().and_then(projection).map_or_else(
        || "--:--".to_string(),
        |value| value.format("%H:%M").to_string(),
    )
}

fn first_day_shifted_time(
    bundle: &ForecastBundle,
    projection: impl Fn(&DailyForecast) -> Option<chrono::NaiveDateTime>,
    shift_minutes: i64,
) -> String {
    bundle
        .daily
        .first()
        .and_then(projection)
        .map(|value| value + chrono::Duration::minutes(shift_minutes))
        .map_or_else(
            || "--:--".to_string(),
            |value| value.format("%H:%M").to_string(),
        )
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

fn day_cue(day: &DailyForecast) -> String {
    let mut parts = vec![precipitation_cue(day)];
    if let Some(gust) = gust_cue(day.wind_gusts_10m_max.unwrap_or(0.0)) {
        parts.push(gust);
    }
    if let Some(sunlight) = sunlight_cue(day) {
        parts.push(sunlight.to_string());
    }
    parts.join(", ")
}
