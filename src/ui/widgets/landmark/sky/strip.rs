use crate::domain::weather::ForecastBundle;

use super::astronomy::format_time_hm;
use super::glyphs::{precip_symbol, symbol_for_code};

pub(super) fn paint_horizon_strip(canvas: &mut [Vec<char>], strip_y: usize, width: usize) {
    for cell in canvas[strip_y.saturating_sub(1)].iter_mut().take(width) {
        if *cell == ' ' {
            *cell = '─';
        }
    }
}

pub(super) fn plot_hourly_strip(
    bundle: &ForecastBundle,
    canvas: &mut [Vec<char>],
    strip_y: usize,
    precip_y: usize,
    width: usize,
) {
    let slice = bundle.hourly.iter().take(width.min(24)).collect::<Vec<_>>();
    for (i, hour) in slice.iter().enumerate() {
        let x = ((i as f32 / slice.len().max(1) as f32) * (width.saturating_sub(1)) as f32).round()
            as usize;
        let code = hour.weather_code.unwrap_or(bundle.current.weather_code);
        canvas[strip_y][x] = symbol_for_code(code);
        canvas[precip_y][x] = precip_symbol(hour.precipitation_mm);
    }
}

pub(super) fn write_summary_line(
    canvas: &mut [Vec<char>],
    summary_y: usize,
    width: usize,
    sunrise_h: f32,
    sunset_h: f32,
    now_h: f32,
) {
    let summary = format!(
        "sun {} -> {}  now {}",
        format_time_hm(sunrise_h),
        format_time_hm(sunset_h),
        format_time_hm(now_h)
    );
    for (idx, ch) in summary.chars().enumerate().take(width) {
        canvas[summary_y][idx] = ch;
    }
}
