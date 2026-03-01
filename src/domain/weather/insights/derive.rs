use super::{
    super::{
        ForecastBundle, HourlyForecast, RefreshMetadata, Units, WeatherCategory, convert_temp,
        round_temp, round_wind_speed, weather_code_to_category, weather_label_for_time,
    },
    ActionCue, ChangeEvent, ChangeKind, InsightConfidence, NowcastInsight, ReliabilitySummary,
};
use crate::resilience::freshness::FreshnessState;

const NOTABLE_SCAN_HOURS: usize = 24;
const ACTION_WINDOW_HOURS: usize = 4;
const ACTION_PROBABLE_PRECIP_THRESHOLD: f32 = 70.0;
const ACTION_SIGNIFICANT_PRECIP_MM: f32 = 0.5;
const ACTION_SIGNIFICANT_PRECIP_TOTAL_MM: f32 = 1.0;
const FREEZING_TEMP_C: f32 = 1.0;
const SIGNIFICANT_PRECIP_MM: f32 = 0.2;
const SIGNIFICANT_WIND_JUMP_KMH: f32 = 18.0;
const SIGNIFICANT_TEMP_SHIFT_C: f32 = 4.0;

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
    Some(format!("Temperature {sign} to {to}°{}", units.symbol()))
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

    if let Some(precip_action) = choose_precip_action(bundle, action_window) {
        return precip_action;
    }
    choose_non_precip_action(bundle, max_gust, uv_today).unwrap_or(ActionCue::Comfortable)
}

fn choose_precip_action(
    bundle: &ForecastBundle,
    action_window: &[HourlyForecast],
) -> Option<ActionCue> {
    if !has_actionable_precip(bundle, action_window) {
        return None;
    }
    if likely_frozen_precip(bundle, action_window) {
        Some(ActionCue::WinterTraction)
    } else {
        Some(ActionCue::CarryUmbrella)
    }
}

fn choose_non_precip_action(
    bundle: &ForecastBundle,
    max_gust: f32,
    uv_today: f32,
) -> Option<ActionCue> {
    if max_gust >= 60.0 {
        return Some(ActionCue::SecureLooseItems);
    }
    if uv_today >= 7.0 {
        return Some(ActionCue::SunProtection);
    }
    if bundle.current.temperature_2m_c >= 32.0 {
        return Some(ActionCue::Hydrate);
    }
    if bundle.current.temperature_2m_c <= 0.0 {
        return Some(ActionCue::LayerUp);
    }
    if bundle.current.visibility_m < 3_000.0 {
        return Some(ActionCue::LowVisibility);
    }
    None
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
    is_current_frozen_precip(bundle)
        || window_has_frozen_precip(bundle, action_window)
        || (is_freezing_now(bundle) && freezing_temperatures_in_window(bundle, action_window))
}

fn is_current_frozen_precip(bundle: &ForecastBundle) -> bool {
    matches!(
        weather_code_to_category(bundle.current.weather_code),
        WeatherCategory::Snow
    ) || is_freezing_precip_code(bundle.current.weather_code)
}

fn window_has_frozen_precip(bundle: &ForecastBundle, action_window: &[HourlyForecast]) -> bool {
    action_window
        .iter()
        .any(|hour| hour_has_frozen_precip(bundle.current.temperature_2m_c, hour))
}

fn hour_has_frozen_precip(current_temp_c: f32, hour: &HourlyForecast) -> bool {
    let hour_temp = hour.temperature_2m_c.unwrap_or(current_temp_c);
    let snowfall_freezing = hour.snowfall_cm.unwrap_or(0.0) > 0.0 && hour_temp <= FREEZING_TEMP_C;
    let snow_code = hour
        .weather_code
        .is_some_and(|code| matches!(weather_code_to_category(code), WeatherCategory::Snow));
    let freezing_precip_code = hour.weather_code.is_some_and(is_freezing_precip_code);
    snowfall_freezing || snow_code || freezing_precip_code
}

fn is_freezing_precip_code(code: u8) -> bool {
    matches!(code, 56 | 57 | 66 | 67)
}

fn is_freezing_now(bundle: &ForecastBundle) -> bool {
    bundle.current.temperature_2m_c <= FREEZING_TEMP_C
}

fn freezing_temperatures_in_window(
    bundle: &ForecastBundle,
    action_window: &[HourlyForecast],
) -> bool {
    action_window.iter().any(|hour| {
        hour.temperature_2m_c
            .unwrap_or(bundle.current.temperature_2m_c)
            <= FREEZING_TEMP_C
    })
}

fn action_text(action: ActionCue, bundle: &ForecastBundle, units: Units) -> String {
    if let Some(text) = static_action_text(action) {
        return text.to_string();
    }
    dynamic_action_text(action, bundle, units)
}

fn static_action_text(action: ActionCue) -> Option<&'static str> {
    match action {
        ActionCue::CarryUmbrella => Some("Now action: keep precipitation gear ready"),
        ActionCue::LowVisibility => Some("Now action: travel with extra visibility caution"),
        ActionCue::Comfortable => Some("Now action: conditions look comfortable"),
        ActionCue::WinterTraction
        | ActionCue::SecureLooseItems
        | ActionCue::SunProtection
        | ActionCue::Hydrate
        | ActionCue::LayerUp => None,
    }
}

fn dynamic_action_text(action: ActionCue, bundle: &ForecastBundle, units: Units) -> String {
    match action {
        ActionCue::WinterTraction => format_temp_action(
            "use winter traction + warm layers",
            bundle.current.temperature_2m_c,
            units,
        ),
        ActionCue::SecureLooseItems => format!(
            "Now action: secure loose items (gusts {} m/s)",
            round_wind_speed(bundle.current.wind_gusts_10m)
        ),
        ActionCue::SunProtection => sun_protection_action(bundle),
        ActionCue::Hydrate => format_temp_action(
            "hydrate and limit heat load",
            bundle.current.temperature_2m_c,
            units,
        ),
        ActionCue::LayerUp => {
            format_temp_action("layer up for cold", bundle.current.temperature_2m_c, units)
        }
        ActionCue::CarryUmbrella | ActionCue::LowVisibility | ActionCue::Comfortable => {
            unreachable!("handled in static_action_text")
        }
    }
}

fn summarize_next_6h(hourly: &[HourlyForecast], units: Units) -> String {
    let Some(window) = hourly.get(..hourly.len().min(6)) else {
        return "Next 6h: data unavailable".to_string();
    };
    if window.is_empty() {
        return "Next 6h: data unavailable".to_string();
    }
    let precip_total = next_6h_precip_total(window);
    let max_probability = next_6h_max_probability(window);
    let max_wind = next_6h_max_wind(window);
    let delta_temp = next_6h_delta_temp(window, units);

    format!(
        "Next 6h: P {precip_total:.1}mm · Pmax {max_probability} · Gust {max_wind} m/s · ΔT {delta_temp}"
    )
}

fn format_temp_action(action: &str, temp_c: f32, units: Units) -> String {
    format!(
        "Now action: {action} ({:>2}°{})",
        round_temp(convert_temp(temp_c, units)),
        units.symbol()
    )
}

fn sun_protection_action(bundle: &ForecastBundle) -> String {
    let uv = bundle
        .daily
        .first()
        .and_then(|day| day.uv_index_max)
        .map_or_else(|| "--".to_string(), |value| format!("{value:.0}"));
    format!("Now action: sun protection advised (UV {uv})")
}

fn next_6h_precip_total(window: &[HourlyForecast]) -> f32 {
    window
        .iter()
        .map(|hour| hour.precipitation_mm.unwrap_or(0.0).max(0.0))
        .sum::<f32>()
}

fn next_6h_max_probability(window: &[HourlyForecast]) -> String {
    window
        .iter()
        .filter_map(|hour| hour.precipitation_probability)
        .max_by(f32::total_cmp)
        .map_or_else(|| "--".to_string(), |value| format!("{value:.0}%"))
}

fn next_6h_max_wind(window: &[HourlyForecast]) -> String {
    window
        .iter()
        .filter_map(wind_reference)
        .max_by(f32::total_cmp)
        .map_or_else(
            || "--".to_string(),
            |value| round_wind_speed(value).to_string(),
        )
}

fn next_6h_delta_temp(window: &[HourlyForecast], units: Units) -> String {
    window
        .first()
        .and_then(|first| first.temperature_2m_c)
        .zip(window.last().and_then(|last| last.temperature_2m_c))
        .map_or_else(
            || "--".to_string(),
            |(start, end)| {
                let delta = round_temp(convert_temp(end - start, units));
                format!("{delta:+}°{}", units.symbol())
            },
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
