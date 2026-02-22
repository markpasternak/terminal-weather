use crate::domain::weather::{DailyForecast, Units, convert_temp, round_temp};

use super::WeekSummaryData;
use super::utils::format_duration_hm;

#[derive(Debug, Default)]
pub(super) struct WeekAccumulator {
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
    comfort_best: Option<(String, f32)>,
    week_min_temp_c: Option<f32>,
    week_max_temp_c: Option<f32>,
}

impl WeekAccumulator {
    pub(super) fn ingest(&mut self, day: &DailyForecast) {
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
        Self::update_comfort_best(&mut self.comfort_best, day);
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

    fn update_comfort_best(slot: &mut Option<(String, f32)>, day: &DailyForecast) {
        let avg_temp = match (day.temperature_min_c, day.temperature_max_c) {
            (Some(min), Some(max)) => Some(f32::midpoint(min, max)),
            (Some(temp), None) | (None, Some(temp)) => Some(temp),
            (None, None) => None,
        };

        let Some(avg_temp) = avg_temp else {
            return;
        };
        let precip_penalty = day.precipitation_sum_mm.unwrap_or(0.0).max(0.0) * 3.0;
        let wind_penalty = day.wind_gusts_10m_max.unwrap_or(0.0).max(0.0) * 0.2;
        let comfort_penalty = (avg_temp - 20.0).abs();
        let score = comfort_penalty + precip_penalty + wind_penalty;
        let day_label = day.date.format("%a").to_string();

        if slot.as_ref().is_none_or(|(_, best)| score < *best) {
            *slot = Some((day_label, score));
        }
    }

    pub(super) fn finish(self, units: Units, daily: &[DailyForecast]) -> WeekSummaryData {
        let wettest_txt = format_day_value_mm(self.wettest);
        let breeziest_txt = format_day_value_mps(self.breeziest);
        let comfort_best_txt = format_best_day(self.comfort_best);
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
            comfort_best_txt,
            precip_hours_avg,
            uv_peak,
            week_thermal,
            highs,
            precip,
            gusts,
        }
    }
}

pub(super) fn format_day_value_mm(value: Option<(String, f32)>) -> String {
    value.map_or_else(|| "--".to_string(), |(day, mm)| format!("{day} {mm:.1}mm"))
}

pub(super) fn format_day_value_mps(value: Option<(String, f32)>) -> String {
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

pub(super) fn average_duration(total_seconds: f32, count: usize) -> String {
    if count > 0 {
        format_duration_hm(total_seconds / count as f32)
    } else {
        "--:--".to_string()
    }
}

pub(super) fn average_precip_hours(total_hours: f32, count: usize) -> String {
    if count > 0 {
        format!("{:.1}h/day", total_hours / count as f32)
    } else {
        "--".to_string()
    }
}

pub(super) fn format_uv_peak(value: Option<(String, f32)>) -> String {
    value.map_or_else(|| "--".to_string(), |(day, uv)| format!("{day} {uv:.1}"))
}

pub(super) fn format_best_day(value: Option<(String, f32)>) -> String {
    value.map_or_else(|| "--".to_string(), |(day, _)| day)
}

pub(super) fn week_thermal_span(min_c: Option<f32>, max_c: Option<f32>, units: Units) -> String {
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
