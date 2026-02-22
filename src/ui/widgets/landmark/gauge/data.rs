use crate::domain::weather::{ForecastBundle, Units, convert_temp, round_temp};

#[derive(Debug)]
pub(super) struct GaugeData {
    pub(super) temp_c: i32,
    pub(super) temp_display: i32,
    pub(super) temp_unit: &'static str,
    pub(super) humidity: f32,
    pub(super) pressure: f32,
    pub(super) wind: f32,
    pub(super) gust: f32,
    pub(super) wind_direction_10m: f32,
    pub(super) uv: f32,
    pub(super) vis_km: f32,
    pub(super) precip_now: f32,
    pub(super) cloud: f32,
    pub(super) meter_w: usize,
    pub(super) left_col_width: usize,
    pub(super) right_trend_width: usize,
    pub(super) sunrise: String,
    pub(super) sunset: String,
    pub(super) temp_track_display: Vec<f32>,
    pub(super) precip_track: Vec<f32>,
    pub(super) gust_track: Vec<f32>,
}

pub(super) fn collect_gauge_data(bundle: &ForecastBundle, units: Units, width: usize) -> GaugeData {
    let current = &bundle.current;
    let left_col_width = left_column_width(width);
    let trend_width = width.saturating_sub(left_col_width + 12).clamp(8, 28);
    let (sunrise, sunset) = gauge_sun_times(bundle);
    let (temp_track_display, precip_track, gust_track) = gauge_tracks(bundle, units);

    GaugeData {
        temp_c: current.temperature_2m_c.round() as i32,
        temp_display: round_temp(convert_temp(current.temperature_2m_c, units)),
        temp_unit: if matches!(units, Units::Celsius) {
            "C"
        } else {
            "F"
        },
        humidity: current.relative_humidity_2m.clamp(0.0, 100.0),
        pressure: current.pressure_msl_hpa,
        wind: current.wind_speed_10m.max(0.0),
        gust: current.wind_gusts_10m.max(0.0),
        wind_direction_10m: current.wind_direction_10m,
        uv: bundle
            .daily
            .first()
            .and_then(|day| day.uv_index_max)
            .unwrap_or(0.0),
        vis_km: (current.visibility_m / 1000.0).max(0.0),
        precip_now: current.precipitation_mm.max(0.0),
        cloud: current.cloud_cover.clamp(0.0, 100.0),
        meter_w: width.saturating_sub(26).clamp(10, 56),
        left_col_width,
        right_trend_width: trend_width.saturating_sub(6),
        sunrise,
        sunset,
        temp_track_display,
        precip_track,
        gust_track,
    }
}

pub(super) fn gauge_sun_times(bundle: &ForecastBundle) -> (String, String) {
    let sunrise = bundle
        .daily
        .first()
        .and_then(|day| day.sunrise)
        .map_or_else(
            || "--:--".to_string(),
            |value| value.format("%H:%M").to_string(),
        );
    let sunset = bundle.daily.first().and_then(|day| day.sunset).map_or_else(
        || "--:--".to_string(),
        |value| value.format("%H:%M").to_string(),
    );
    (sunrise, sunset)
}

fn gauge_tracks(bundle: &ForecastBundle, units: Units) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let temp_track = bundle
        .hourly
        .iter()
        .take(24)
        .filter_map(|hour| {
            hour.temperature_2m_c
                .map(|temp_c| convert_temp(temp_c, units))
        })
        .collect::<Vec<_>>();
    let precip_track = bundle
        .hourly
        .iter()
        .take(24)
        .map(|hour| hour.precipitation_mm.unwrap_or(0.0))
        .collect::<Vec<_>>();
    let gust_track = bundle
        .hourly
        .iter()
        .take(24)
        .map(|hour| hour.wind_gusts_10m.unwrap_or(0.0))
        .collect::<Vec<_>>();
    (temp_track, precip_track, gust_track)
}

pub(super) fn left_column_width(width: usize) -> usize {
    if width >= 86 {
        width.saturating_mul(58) / 100
    } else if width >= 74 {
        width.saturating_mul(62) / 100
    } else {
        width
    }
}
