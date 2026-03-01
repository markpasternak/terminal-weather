use super::*;

mod aqi_tests;
mod conversion_tests;
mod daypart_tests;
mod insight_tests;
mod refresh_tests;

fn minimal_bundle(high_c: Option<f32>, low_c: Option<f32>) -> ForecastBundle {
    ForecastBundle {
        location: Location::from_coords(0.0, 0.0),
        current: CurrentConditions {
            temperature_2m_c: 20.0,
            relative_humidity_2m: 50.0,
            apparent_temperature_c: 20.0,
            dew_point_2m_c: 10.0,
            weather_code: 0,
            precipitation_mm: 0.0,
            cloud_cover: 0.0,
            pressure_msl_hpa: 1013.0,
            visibility_m: 10000.0,
            wind_speed_10m: 10.0,
            wind_gusts_10m: 15.0,
            wind_direction_10m: 180.0,
            is_day: true,
            high_today_c: high_c,
            low_today_c: low_c,
        },
        hourly: vec![],
        daily: vec![],
        air_quality: None,
        fetched_at: chrono::Utc::now(),
    }
}

fn sample_hour(
    time: chrono::NaiveDateTime,
    temp_c: f32,
    weather_code: u8,
    precip_probability: f32,
    precip_mm: f32,
    wind_kmh: f32,
    visibility_m: f32,
) -> HourlyForecast {
    HourlyForecast {
        time,
        temperature_2m_c: Some(temp_c),
        weather_code: Some(weather_code),
        is_day: Some(true),
        relative_humidity_2m: None,
        precipitation_probability: Some(precip_probability),
        precipitation_mm: Some(precip_mm),
        rain_mm: None,
        snowfall_cm: None,
        wind_speed_10m: Some(wind_kmh),
        wind_gusts_10m: None,
        pressure_msl_hpa: None,
        visibility_m: Some(visibility_m),
        cloud_cover: None,
        cloud_cover_low: None,
        cloud_cover_mid: None,
        cloud_cover_high: None,
    }
}
