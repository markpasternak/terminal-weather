use chrono::Timelike;

use crate::domain::weather::ForecastBundle;

use super::glyphs::{precip_symbol, symbol_for_code};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BandDensity {
    Full24,
    Balanced18,
    Compact12,
}

impl BandDensity {
    pub(super) const fn sample_count(self) -> usize {
        match self {
            Self::Full24 => 24,
            Self::Balanced18 => 18,
            Self::Compact12 => 12,
        }
    }

    pub(super) const fn label_interval(self) -> usize {
        match self {
            Self::Full24 => 3,
            Self::Balanced18 => 4,
            Self::Compact12 => 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HourSample {
    pub(super) x: usize,
    pub(super) hour_label: String,
    pub(super) weather_symbol: char,
    pub(super) precip_symbol: char,
    pub(super) show_label: bool,
}

pub(super) fn band_density(width: usize) -> BandDensity {
    if width >= 100 {
        BandDensity::Full24
    } else if width >= 80 {
        BandDensity::Balanced18
    } else {
        BandDensity::Compact12
    }
}

pub(super) fn build_hour_samples(
    bundle: &ForecastBundle,
    width: usize,
) -> (BandDensity, Vec<HourSample>) {
    let density = band_density(width);
    let prefix_cols = usize::from(width >= 80) * 2;
    let plot_width = width.saturating_sub(prefix_cols).max(1);
    let source = bundle.hourly.iter().take(24).collect::<Vec<_>>();
    let target_count = density.sample_count().min(source.len());
    let mut samples = Vec::with_capacity(target_count);

    if source.is_empty() {
        return (density, samples);
    }

    for sample_idx in 0..target_count {
        let source_idx = sample_idx.saturating_mul(source.len()) / target_count;
        let hour = source[source_idx];
        let x = prefix_cols
            + position_for_index(sample_idx, target_count, plot_width)
                .min(width.saturating_sub(1).saturating_sub(prefix_cols))
                .min(plot_width.saturating_sub(1));
        let code = hour.weather_code.unwrap_or(bundle.current.weather_code);
        samples.push(HourSample {
            x,
            hour_label: format!("{:02}", hour.time.hour()),
            weather_symbol: symbol_for_code(code),
            precip_symbol: precip_symbol(hour.precipitation_mm),
            show_label: sample_idx % density.label_interval() == 0,
        });
    }

    (density, samples)
}

pub(super) fn horizon_y(height: usize) -> usize {
    height.saturating_sub(5)
}

pub(super) fn paint_horizon_strip(canvas: &mut [Vec<char>], horizon_y: usize, width: usize) {
    for cell in canvas[horizon_y].iter_mut().take(width) {
        *cell = '─';
    }
    if width > 0 {
        canvas[horizon_y][0] = 'E';
    }
    if width > 1 {
        canvas[horizon_y][width - 1] = 'W';
    }
}

pub(super) fn paint_observatory_band(
    canvas: &mut [Vec<char>],
    width: usize,
    tick_y: usize,
    weather_y: usize,
    precip_y: usize,
    density: BandDensity,
    samples: &[HourSample],
) {
    if width >= 80 {
        canvas[tick_y][0] = 't';
        canvas[weather_y][0] = 'w';
        canvas[precip_y][0] = 'p';
    }

    for sample in samples {
        canvas[tick_y][sample.x] = '·';
        canvas[weather_y][sample.x] = sample.weather_symbol;
        canvas[precip_y][sample.x] = sample.precip_symbol;
    }

    let mut last_label_end = 0usize;
    let label_width = 2usize;
    for sample in samples.iter().filter(|sample| sample.show_label) {
        let start = sample.x.saturating_sub(1);
        let end = start + label_width;
        if end > width || start < last_label_end {
            continue;
        }
        for (offset, ch) in sample.hour_label.chars().take(label_width).enumerate() {
            canvas[tick_y][start + offset] = ch;
        }
        last_label_end = end + usize::from(density != BandDensity::Compact12);
    }
}

pub(super) fn write_summary_line(
    canvas: &mut [Vec<char>],
    summary_y: usize,
    width: usize,
    segments: &[String],
) {
    let summary = segments.join("  ");
    for (idx, ch) in summary.chars().enumerate().take(width) {
        canvas[summary_y][idx] = ch;
    }
}

pub(super) fn summary_segments(
    width: usize,
    sunrise_text: &str,
    sunset_text: &str,
    daylight_text: &str,
    sunshine_text: Option<&str>,
    moon_symbol: char,
) -> Vec<String> {
    let mut segments = vec![format!("Rise {sunrise_text}"), format!("Set {sunset_text}")];

    if width >= 72 {
        segments.push(format!("Daylight {daylight_text}"));
    }
    if width >= 100
        && let Some(sunshine_text) = sunshine_text
    {
        segments.push(format!("Sunshine {sunshine_text}"));
    }
    segments.push(format!("Moon {moon_symbol}"));

    segments
}

fn position_for_index(index: usize, total: usize, width: usize) -> usize {
    if total <= 1 || width <= 1 {
        return 0;
    }

    ((index as f32 / (total - 1) as f32) * (width - 1) as f32).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{sample_bundle, sample_hourly};

    #[test]
    fn band_density_switches_at_expected_widths() {
        assert_eq!(band_density(79), BandDensity::Compact12);
        assert_eq!(band_density(80), BandDensity::Balanced18);
        assert_eq!(band_density(100), BandDensity::Full24);
    }

    #[test]
    fn build_hour_samples_uses_density_targets_and_prefix_offset() {
        let mut bundle = sample_bundle();
        bundle.hourly = (0..24)
            .map(|hour| {
                let mut forecast = sample_hourly();
                forecast.time += chrono::Duration::hours(i64::from(hour));
                forecast
            })
            .collect();

        let (density, samples) = build_hour_samples(&bundle, 96);
        assert_eq!(density, BandDensity::Balanced18);
        assert_eq!(samples.len(), 18);
        assert!(samples.iter().all(|sample| sample.x >= 2));
    }

    #[test]
    fn summary_segments_expand_by_width() {
        let wide = summary_segments(120, "06:30", "18:00", "11:30", Some("04:00"), '◑');
        assert!(wide.iter().any(|segment| segment.contains("Sunshine")));

        let medium = summary_segments(88, "06:30", "18:00", "11:30", Some("04:00"), '◑');
        assert!(medium.iter().any(|segment| segment.contains("Daylight")));
        assert!(!medium.iter().any(|segment| segment.contains("Sunshine")));

        let narrow = summary_segments(60, "06:30", "18:00", "11:30", Some("04:00"), '◑');
        assert_eq!(narrow.len(), 3);
        assert!(
            narrow
                .last()
                .is_some_and(|segment| segment.contains("Moon"))
        );
    }

    #[test]
    fn paint_observatory_band_places_labels_without_overlap() {
        let mut canvas = vec![vec![' '; 64]; 4];
        let samples = vec![
            HourSample {
                x: 0,
                hour_label: "06".to_string(),
                weather_symbol: 'o',
                precip_symbol: '·',
                show_label: true,
            },
            HourSample {
                x: 12,
                hour_label: "12".to_string(),
                weather_symbol: '~',
                precip_symbol: '▒',
                show_label: true,
            },
            HourSample {
                x: 24,
                hour_label: "18".to_string(),
                weather_symbol: '/',
                precip_symbol: '█',
                show_label: true,
            },
        ];

        paint_observatory_band(&mut canvas, 64, 0, 1, 2, BandDensity::Compact12, &samples);

        let tick_row = canvas[0].iter().collect::<String>();
        assert!(tick_row.contains("06"));
        assert!(tick_row.contains("12"));
        assert!(tick_row.contains("18"));
    }
}
