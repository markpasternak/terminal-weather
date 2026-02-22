use crate::resilience::freshness::FreshnessState;

use super::{
    ForecastBundle, HourlyForecast, RefreshMetadata, Units, WeatherCategory, convert_temp,
    round_temp, round_wind_speed, weather_code_to_category, weather_label_for_time,
};

const NOTABLE_SCAN_HOURS: usize = 24;
const ACTION_WINDOW_HOURS: usize = 4;
const ACTION_PROBABLE_PRECIP_THRESHOLD: f32 = 70.0;
const ACTION_SIGNIFICANT_PRECIP_MM: f32 = 0.5;
const ACTION_SIGNIFICANT_PRECIP_TOTAL_MM: f32 = 1.0;
const FREEZING_TEMP_C: f32 = 1.0;
const SIGNIFICANT_PRECIP_MM: f32 = 0.2;
const SIGNIFICANT_WIND_JUMP_KMH: f32 = 18.0;
const SIGNIFICANT_TEMP_SHIFT_C: f32 = 4.0;

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

#[must_use]
pub fn derive_nowcast_insight(
    bundle: &ForecastBundle,
    units: Units,
    refresh_meta: &RefreshMetadata,
) -> NowcastInsight {
    let base_confidence = confidence_from_hourly(&bundle.hourly);
    let confidence = adjust_confidence_for_freshness(base_confidence, refresh_meta.state);
    let action = choose_action(bundle);

    NowcastInsight {
        action,
        action_text: action_text(action, bundle, units),
        next_change: next_notable_change(&bundle.hourly, units),
        reliability: ReliabilitySummary {
            state: refresh_meta.state,
            age_minutes: refresh_meta.age_minutes(),
            retry_in_seconds: refresh_meta.retry_in_seconds(),
            consecutive_failures: refresh_meta.consecutive_failures,
        },
        confidence,
        next_6h_summary: summarize_next_6h(&bundle.hourly, units),
    }
}

#[must_use]
pub fn next_notable_change(hourly: &[HourlyForecast], units: Units) -> Option<ChangeEvent> {
    if hourly.len() < 2 {
        return None;
    }

    let base = &hourly[0];
    let max_scan = hourly.len().min(NOTABLE_SCAN_HOURS);
    for idx in 1..max_scan {
        let previous = &hourly[idx - 1];
        let current = &hourly[idx];
        let hours_from_now = elapsed_hours(base, current);

        if precipitation_start(previous, current) {
            return Some(ChangeEvent {
                hours_from_now,
                kind: ChangeKind::PrecipStart,
                message: "Precipitation starts".to_string(),
            });
        }

        if let Some(message) = condition_shift_message(previous, current) {
            return Some(ChangeEvent {
                hours_from_now,
                kind: ChangeKind::ConditionShift,
                message,
            });
        }

        if let Some(message) = wind_jump_message(base, current) {
            return Some(ChangeEvent {
                hours_from_now,
                kind: ChangeKind::WindIncrease,
                message,
            });
        }

        if let Some(message) = temp_shift_message(base, current, units) {
            return Some(ChangeEvent {
                hours_from_now,
                kind: ChangeKind::TempShift,
                message,
            });
        }
    }

    None
}

fn elapsed_hours(base: &HourlyForecast, current: &HourlyForecast) -> usize {
    let delta = current.time - base.time;
    usize::try_from(delta.num_hours().max(0)).unwrap_or(0)
}

fn precipitation_start(previous: &HourlyForecast, current: &HourlyForecast) -> bool {
    let previous_mm = previous.precipitation_mm.unwrap_or(0.0).max(0.0);
    let current_mm = current.precipitation_mm.unwrap_or(0.0).max(0.0);
    previous_mm < SIGNIFICANT_PRECIP_MM && current_mm >= SIGNIFICANT_PRECIP_MM
}

fn condition_shift_message(previous: &HourlyForecast, current: &HourlyForecast) -> Option<String> {
    let prev_code = previous.weather_code?;
    let next_code = current.weather_code?;
    let prev_category = weather_code_to_category(prev_code);
    let next_category = weather_code_to_category(next_code);
    if prev_category == next_category {
        return None;
    }

    let is_day = current.is_day.unwrap_or(true);
    let next_label = weather_label_for_time(next_code, is_day);
    Some(format!("Conditions shift to {next_label}"))
}

fn wind_jump_message(base: &HourlyForecast, current: &HourlyForecast) -> Option<String> {
    let base_wind = wind_reference(base)?;
    let next_wind = wind_reference(current)?;
    if next_wind - base_wind < SIGNIFICANT_WIND_JUMP_KMH {
        return None;
    }
    Some(format!(
        "Wind picks up to {} m/s",
        round_wind_speed(next_wind)
    ))
}

fn wind_reference(hour: &HourlyForecast) -> Option<f32> {
    hour.wind_gusts_10m.or(hour.wind_speed_10m)
}

fn temp_shift_message(
    base: &HourlyForecast,
    current: &HourlyForecast,
    units: Units,
) -> Option<String> {
    let base_temp = base.temperature_2m_c?;
    let next_temp = current.temperature_2m_c?;
    let shift = next_temp - base_temp;
    if shift.abs() < SIGNIFICANT_TEMP_SHIFT_C {
        return None;
    }

    let to = round_temp(convert_temp(next_temp, units));
    let sign = if shift > 0.0 { "rises" } else { "drops" };
    Some(format!("Temperature {sign} to {to}°{}", unit_symbol(units)))
}

fn unit_symbol(units: Units) -> &'static str {
    match units {
        Units::Celsius => "C",
        Units::Fahrenheit => "F",
    }
}

fn choose_action(bundle: &ForecastBundle) -> ActionCue {
    let action_window = &bundle.hourly[..bundle.hourly.len().min(ACTION_WINDOW_HOURS)];
    let near_window = &bundle.hourly[..bundle.hourly.len().min(12)];
    let max_gust = near_window
        .iter()
        .filter_map(wind_reference)
        .max_by(f32::total_cmp)
        .unwrap_or(bundle.current.wind_gusts_10m);
    let uv_today = bundle
        .daily
        .first()
        .and_then(|day| day.uv_index_max)
        .unwrap_or(0.0);

    if has_actionable_precip(bundle, action_window) {
        if likely_frozen_precip(bundle, action_window) {
            return ActionCue::WinterTraction;
        }
        return ActionCue::CarryUmbrella;
    }
    if max_gust >= 60.0 {
        return ActionCue::SecureLooseItems;
    }
    if uv_today >= 7.0 {
        return ActionCue::SunProtection;
    }
    if bundle.current.temperature_2m_c >= 32.0 {
        return ActionCue::Hydrate;
    }
    if bundle.current.temperature_2m_c <= 0.0 {
        return ActionCue::LayerUp;
    }
    if bundle.current.visibility_m < 3_000.0 {
        return ActionCue::LowVisibility;
    }
    ActionCue::Comfortable
}

fn has_actionable_precip(bundle: &ForecastBundle, action_window: &[HourlyForecast]) -> bool {
    if bundle.current.precipitation_mm.max(0.0) >= SIGNIFICANT_PRECIP_MM {
        return true;
    }
    if action_window.is_empty() {
        return false;
    }

    let max_precip_probability = action_window
        .iter()
        .filter_map(|hour| hour.precipitation_probability)
        .max_by(f32::total_cmp)
        .unwrap_or(0.0);
    let max_precip_mm = action_window
        .iter()
        .map(|hour| hour.precipitation_mm.unwrap_or(0.0).max(0.0))
        .max_by(f32::total_cmp)
        .unwrap_or(0.0);
    let total_precip_mm = action_window
        .iter()
        .map(|hour| hour.precipitation_mm.unwrap_or(0.0).max(0.0))
        .sum::<f32>();

    max_precip_probability >= ACTION_PROBABLE_PRECIP_THRESHOLD
        || max_precip_mm >= ACTION_SIGNIFICANT_PRECIP_MM
        || total_precip_mm >= ACTION_SIGNIFICANT_PRECIP_TOTAL_MM
}

fn likely_frozen_precip(bundle: &ForecastBundle, action_window: &[HourlyForecast]) -> bool {
    let freezing_now = bundle.current.temperature_2m_c <= FREEZING_TEMP_C;
    let freezing_window = action_window.iter().any(|hour| {
        hour.temperature_2m_c
            .unwrap_or(bundle.current.temperature_2m_c)
            <= FREEZING_TEMP_C
    });
    let current_snow = matches!(
        weather_code_to_category(bundle.current.weather_code),
        WeatherCategory::Snow
    );
    let current_freezing_precip = matches!(bundle.current.weather_code, 56 | 57 | 66 | 67);
    let window_snow_or_ice = action_window.iter().any(|hour| {
        (hour.snowfall_cm.unwrap_or(0.0) > 0.0
            && hour
                .temperature_2m_c
                .unwrap_or(bundle.current.temperature_2m_c)
                <= FREEZING_TEMP_C)
            || hour
                .weather_code
                .is_some_and(|code| matches!(weather_code_to_category(code), WeatherCategory::Snow))
            || hour
                .weather_code
                .is_some_and(|code| matches!(code, 56 | 57 | 66 | 67))
    });

    current_snow
        || current_freezing_precip
        || window_snow_or_ice
        || (freezing_now && freezing_window)
}

fn action_text(action: ActionCue, bundle: &ForecastBundle, units: Units) -> String {
    match action {
        ActionCue::CarryUmbrella => "Now action: keep precipitation gear ready".to_string(),
        ActionCue::WinterTraction => format!(
            "Now action: use winter traction + warm layers ({:>2}°{})",
            round_temp(convert_temp(bundle.current.temperature_2m_c, units)),
            unit_symbol(units)
        ),
        ActionCue::SecureLooseItems => format!(
            "Now action: secure loose items (gusts {} m/s)",
            round_wind_speed(bundle.current.wind_gusts_10m)
        ),
        ActionCue::SunProtection => {
            let uv = bundle
                .daily
                .first()
                .and_then(|day| day.uv_index_max)
                .map_or_else(|| "--".to_string(), |value| format!("{value:.0}"));
            format!("Now action: sun protection advised (UV {uv})")
        }
        ActionCue::Hydrate => format!(
            "Now action: hydrate and limit heat load ({:>2}°{})",
            round_temp(convert_temp(bundle.current.temperature_2m_c, units)),
            unit_symbol(units)
        ),
        ActionCue::LayerUp => format!(
            "Now action: layer up for cold ({:>2}°{})",
            round_temp(convert_temp(bundle.current.temperature_2m_c, units)),
            unit_symbol(units)
        ),
        ActionCue::LowVisibility => "Now action: travel with extra visibility caution".to_string(),
        ActionCue::Comfortable => "Now action: conditions look comfortable".to_string(),
    }
}

fn summarize_next_6h(hourly: &[HourlyForecast], units: Units) -> String {
    if hourly.is_empty() {
        return "Next 6h: data unavailable".to_string();
    }

    let window = &hourly[..hourly.len().min(6)];
    let precip_total = window
        .iter()
        .map(|hour| hour.precipitation_mm.unwrap_or(0.0).max(0.0))
        .sum::<f32>();
    let max_probability = window
        .iter()
        .filter_map(|hour| hour.precipitation_probability)
        .max_by(f32::total_cmp)
        .map_or_else(|| "--".to_string(), |value| format!("{value:.0}%"));
    let max_wind = window
        .iter()
        .filter_map(wind_reference)
        .max_by(f32::total_cmp)
        .map_or_else(
            || "--".to_string(),
            |value| round_wind_speed(value).to_string(),
        );

    let delta_temp = window
        .first()
        .and_then(|first| first.temperature_2m_c)
        .zip(window.last().and_then(|last| last.temperature_2m_c))
        .map_or_else(
            || "--".to_string(),
            |(start, end)| {
                let delta = round_temp(convert_temp(end - start, units));
                format!("{delta:+}°{}", unit_symbol(units))
            },
        );

    format!(
        "Next 6h: P {precip_total:.1}mm · Pmax {max_probability} · Gust {max_wind} m/s · ΔT {delta_temp}"
    )
}

fn confidence_from_hourly(hourly: &[HourlyForecast]) -> InsightConfidence {
    if hourly.is_empty() {
        return InsightConfidence::Low;
    }

    let window = &hourly[..hourly.len().min(12)];
    let mut present = 0usize;
    let mut total = 0usize;
    for hour in window {
        total += 4;
        present += usize::from(hour.temperature_2m_c.is_some());
        present += usize::from(hour.precipitation_probability.is_some());
        present += usize::from(wind_reference(hour).is_some());
        present += usize::from(hour.weather_code.is_some());
    }

    let coverage_pct = if total == 0 {
        0usize
    } else {
        present.saturating_mul(100) / total
    };
    if coverage_pct >= 75 {
        InsightConfidence::High
    } else if coverage_pct >= 45 {
        InsightConfidence::Medium
    } else {
        InsightConfidence::Low
    }
}

fn adjust_confidence_for_freshness(
    confidence: InsightConfidence,
    freshness: FreshnessState,
) -> InsightConfidence {
    match freshness {
        FreshnessState::Fresh => confidence,
        FreshnessState::Stale => degrade_confidence(confidence),
        FreshnessState::Offline => InsightConfidence::Low,
    }
}

fn degrade_confidence(confidence: InsightConfidence) -> InsightConfidence {
    match confidence {
        InsightConfidence::High => InsightConfidence::Medium,
        InsightConfidence::Medium | InsightConfidence::Low => InsightConfidence::Low,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, Utc};

    use super::*;
    use crate::{
        domain::weather::{CurrentConditions, DailyForecast, Location},
        resilience::freshness::FreshnessState,
    };

    fn fresh_meta() -> RefreshMetadata {
        RefreshMetadata {
            state: FreshnessState::Fresh,
            ..Default::default()
        }
    }

    fn stale_meta() -> RefreshMetadata {
        RefreshMetadata {
            state: FreshnessState::Stale,
            ..Default::default()
        }
    }

    fn base_hour(i: i64) -> HourlyForecast {
        let time = NaiveDate::from_ymd_opt(2026, 2, 22)
            .expect("valid date")
            .and_hms_opt(0, 0, 0)
            .expect("valid time")
            + chrono::Duration::hours(i);
        HourlyForecast {
            time,
            temperature_2m_c: Some(18.0),
            weather_code: Some(1),
            is_day: Some(true),
            relative_humidity_2m: Some(60.0),
            precipitation_probability: Some(10.0),
            precipitation_mm: Some(0.0),
            rain_mm: Some(0.0),
            snowfall_cm: Some(0.0),
            wind_speed_10m: Some(8.0),
            wind_gusts_10m: Some(15.0),
            pressure_msl_hpa: Some(1013.0),
            visibility_m: Some(10_000.0),
            cloud_cover: Some(20.0),
            cloud_cover_low: Some(5.0),
            cloud_cover_mid: Some(10.0),
            cloud_cover_high: Some(5.0),
        }
    }

    fn clear_bundle() -> ForecastBundle {
        ForecastBundle {
            location: Location::from_coords(59.3, 18.0),
            current: CurrentConditions {
                temperature_2m_c: 18.0,
                relative_humidity_2m: 60.0,
                apparent_temperature_c: 17.0,
                dew_point_2m_c: 10.0,
                weather_code: 1,
                precipitation_mm: 0.0,
                cloud_cover: 20.0,
                pressure_msl_hpa: 1013.0,
                visibility_m: 10_000.0,
                wind_speed_10m: 8.0,
                wind_gusts_10m: 15.0,
                wind_direction_10m: 90.0,
                is_day: true,
                high_today_c: Some(22.0),
                low_today_c: Some(14.0),
            },
            hourly: (0..24).map(base_hour).collect(),
            daily: vec![DailyForecast {
                date: NaiveDate::from_ymd_opt(2026, 2, 22).expect("valid date"),
                weather_code: Some(1),
                temperature_max_c: Some(22.0),
                temperature_min_c: Some(14.0),
                sunrise: None,
                sunset: None,
                uv_index_max: Some(2.0),
                precipitation_probability_max: Some(5.0),
                precipitation_sum_mm: Some(0.0),
                rain_sum_mm: Some(0.0),
                snowfall_sum_cm: Some(0.0),
                precipitation_hours: Some(0.0),
                wind_gusts_10m_max: Some(15.0),
                daylight_duration_s: Some(36_000.0),
                sunshine_duration_s: Some(28_000.0),
            }],
            air_quality: None,
            fetched_at: Utc::now(),
        }
    }

    fn rainy_bundle() -> ForecastBundle {
        let mut bundle = clear_bundle();
        bundle.hourly = (0..24)
            .map(|i| {
                let mut h = base_hour(i);
                if i >= 2 {
                    h.weather_code = Some(61);
                    h.precipitation_mm = Some(1.5);
                    h.precipitation_probability = Some(85.0);
                }
                h
            })
            .collect();
        bundle
    }

    #[test]
    fn comfortable_action_for_clear_mild_weather() {
        let bundle = clear_bundle();
        let insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
        assert_eq!(insight.action, ActionCue::Comfortable);
        assert_eq!(insight.confidence, InsightConfidence::High);
    }

    #[test]
    fn carry_umbrella_when_rain_imminent() {
        let bundle = rainy_bundle();
        let insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
        assert_eq!(insight.action, ActionCue::CarryUmbrella);
    }

    #[test]
    fn next_notable_change_detects_precip_start() {
        let bundle = rainy_bundle();
        let change = next_notable_change(&bundle.hourly, Units::Celsius);
        assert!(change.is_some(), "expected a change event for rain onset");
        let change = change.unwrap();
        assert_eq!(change.kind, ChangeKind::PrecipStart);
        assert!(change.hours_from_now <= 4);
    }

    #[test]
    fn next_notable_change_returns_none_for_stable_clear() {
        let bundle = clear_bundle();
        assert!(next_notable_change(&bundle.hourly, Units::Celsius).is_none());
    }

    #[test]
    fn reliability_line_contains_freshness_state() {
        let insight = derive_nowcast_insight(&clear_bundle(), Units::Celsius, &fresh_meta());
        assert!(
            insight.reliability.line().contains("fresh"),
            "got: {}",
            insight.reliability.line()
        );
    }

    #[test]
    fn confidence_degrades_under_stale_data() {
        let bundle = clear_bundle();
        let fresh_insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
        let stale_insight = derive_nowcast_insight(&bundle, Units::Celsius, &stale_meta());
        assert_eq!(fresh_insight.confidence, InsightConfidence::High);
        assert_ne!(stale_insight.confidence, InsightConfidence::High);
    }

    #[test]
    fn layer_up_when_below_freezing() {
        let mut bundle = clear_bundle();
        bundle.current.temperature_2m_c = -5.0;
        let insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
        assert_eq!(insight.action, ActionCue::LayerUp);
    }

    #[test]
    fn winter_traction_when_snow_with_precip() {
        let mut bundle = rainy_bundle();
        bundle.current.weather_code = 71; // snow
        bundle.current.temperature_2m_c = -2.0;
        let insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
        assert_eq!(insight.action, ActionCue::WinterTraction);
    }
}
