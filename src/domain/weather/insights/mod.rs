mod derive;
mod types;

pub use derive::{derive_nowcast_insight, next_notable_change};
pub use types::{
    ActionCue, ChangeEvent, ChangeKind, InsightConfidence, NowcastInsight, ReliabilitySummary,
};

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, Utc};

    use super::*;
    use crate::{
        domain::weather::{
            CurrentConditions, DailyForecast, HourlyForecast, Location, RefreshMetadata, Units,
        },
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

    fn clear_bundle() -> crate::domain::weather::ForecastBundle {
        crate::domain::weather::ForecastBundle {
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

    fn rainy_bundle() -> crate::domain::weather::ForecastBundle {
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
        bundle.current.weather_code = 71;
        bundle.current.temperature_2m_c = -2.0;
        let insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
        assert_eq!(insight.action, ActionCue::WinterTraction);
    }
}
