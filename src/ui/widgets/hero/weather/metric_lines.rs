use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::{domain::weather::AirQualityCategory, ui::theme::Theme};

#[derive(Debug)]
pub(super) struct WeatherMetricsData {
    pub(super) feels: i32,
    pub(super) humidity: i32,
    pub(super) dew: i32,
    pub(super) wind_dir: &'static str,
    pub(super) wind: i32,
    pub(super) gust: i32,
    pub(super) visibility: String,
    pub(super) pressure: i32,
    pub(super) pressure_trend: &'static str,
    pub(super) uv_today: String,
    pub(super) cloud_total: i32,
    pub(super) cloud_split: String,
    pub(super) precip_probability: String,
    pub(super) aqi: String,
    pub(super) aqi_category: AirQualityCategory,
    pub(super) aqi_available: bool,
}

pub(super) fn push_metric_lines(
    lines: &mut Vec<Line<'static>>,
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
    compact: bool,
) {
    let rows: &[MetricRow] = if compact {
        &COMPACT_ROWS
    } else {
        &STANDARD_ROWS
    };
    lines.extend(
        rows.iter()
            .copied()
            .map(|row| build_metric_row(data, theme, metric_gap, row)),
    );
}

#[derive(Clone, Copy)]
enum MetricSlot {
    Feels,
    DewCompact,
    DewStandard,
    Wind,
    Visibility,
    PressureCompact,
    PressureStandard,
    Humidity,
    Cloud,
    Uv,
    RainChance,
    Aqi,
}

#[derive(Clone, Copy)]
enum MetricRow {
    Pair(MetricSlot, MetricSlot),
    Single(MetricSlot),
    CloudUv,
}

const COMPACT_ROWS: [MetricRow; 4] = [
    MetricRow::Pair(MetricSlot::Wind, MetricSlot::Visibility),
    MetricRow::Single(MetricSlot::PressureCompact),
    MetricRow::Pair(MetricSlot::DewCompact, MetricSlot::Humidity),
    MetricRow::Pair(MetricSlot::RainChance, MetricSlot::Aqi),
];

const STANDARD_ROWS: [MetricRow; 5] = [
    MetricRow::Pair(MetricSlot::Feels, MetricSlot::DewStandard),
    MetricRow::Pair(MetricSlot::Wind, MetricSlot::Visibility),
    MetricRow::Pair(MetricSlot::PressureStandard, MetricSlot::Humidity),
    MetricRow::CloudUv,
    MetricRow::Pair(MetricSlot::RainChance, MetricSlot::Aqi),
];

fn build_metric_row(
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
    row: MetricRow,
) -> Line<'static> {
    let mut spans = Vec::new();
    match row {
        MetricRow::Pair(left, right) => {
            append_metric(&mut spans, descriptor(data, theme, left), theme);
            spans.push(Span::raw(metric_gap));
            append_metric(&mut spans, descriptor(data, theme, right), theme);
        }
        MetricRow::Single(slot) => {
            append_metric(&mut spans, descriptor(data, theme, slot), theme);
        }
        MetricRow::CloudUv => {
            append_metric(
                &mut spans,
                descriptor(data, theme, MetricSlot::Cloud),
                theme,
            );
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                data.cloud_split.clone(),
                Style::default().fg(theme.muted_text),
            ));
            spans.push(Span::raw(metric_gap));
            append_metric(&mut spans, descriptor(data, theme, MetricSlot::Uv), theme);
        }
    }
    Line::from(spans)
}

struct MetricDescriptor {
    label: &'static str,
    value: String,
    color: Color,
}

fn descriptor(data: &WeatherMetricsData, theme: Theme, slot: MetricSlot) -> MetricDescriptor {
    match slot {
        MetricSlot::Feels
        | MetricSlot::DewCompact
        | MetricSlot::DewStandard
        | MetricSlot::Wind
        | MetricSlot::Visibility
        | MetricSlot::PressureCompact => descriptor_primary(data, theme, slot),
        MetricSlot::PressureStandard
        | MetricSlot::Humidity
        | MetricSlot::Cloud
        | MetricSlot::Uv
        | MetricSlot::RainChance
        | MetricSlot::Aqi => descriptor_secondary(data, theme, slot),
    }
}

fn descriptor_primary(
    data: &WeatherMetricsData,
    theme: Theme,
    slot: MetricSlot,
) -> MetricDescriptor {
    match slot {
        MetricSlot::Feels => MetricDescriptor {
            label: "Feels ",
            value: format!("{}°", data.feels),
            color: theme.text,
        },
        MetricSlot::DewCompact => MetricDescriptor {
            label: "Dew ",
            value: format!("{}°", data.dew),
            color: theme.text,
        },
        MetricSlot::DewStandard => MetricDescriptor {
            label: "Dew ",
            value: format!("{}°", data.dew),
            color: theme.info,
        },
        MetricSlot::Wind => MetricDescriptor {
            label: "Wind ",
            value: format!("{}/{} m/s {}", data.wind, data.gust, data.wind_dir),
            color: theme.success,
        },
        MetricSlot::Visibility => MetricDescriptor {
            label: "Visibility ",
            value: data.visibility.clone(),
            color: theme.accent,
        },
        MetricSlot::PressureCompact => MetricDescriptor {
            label: "Pressure ",
            value: format!("{}{}", data.pressure, data.pressure_trend),
            color: theme.warning,
        },
        _ => unreachable!("unsupported slot in descriptor_primary"),
    }
}

fn descriptor_secondary(
    data: &WeatherMetricsData,
    theme: Theme,
    slot: MetricSlot,
) -> MetricDescriptor {
    match slot {
        MetricSlot::PressureStandard => MetricDescriptor {
            label: "Pressure ",
            value: format!("{}hPa{}", data.pressure, data.pressure_trend),
            color: theme.warning,
        },
        MetricSlot::Humidity => MetricDescriptor {
            label: "Humidity ",
            value: format!("{}%", data.humidity),
            color: theme.info,
        },
        MetricSlot::Cloud => MetricDescriptor {
            label: "Cloud ",
            value: format!("{}%", data.cloud_total),
            color: theme.landmark_neutral,
        },
        MetricSlot::Uv => MetricDescriptor {
            label: "UV ",
            value: data.uv_today.clone(),
            color: theme.warning,
        },
        MetricSlot::RainChance => MetricDescriptor {
            label: "Rain chance ",
            value: data.precip_probability.clone(),
            color: theme.info,
        },
        MetricSlot::Aqi => MetricDescriptor {
            label: "AQI ",
            value: data.aqi.clone(),
            color: aqi_color(data, theme),
        },
        _ => unreachable!("unsupported slot in descriptor_secondary"),
    }
}

fn append_metric(spans: &mut Vec<Span<'static>>, descriptor: MetricDescriptor, theme: Theme) {
    spans.push(Span::styled(
        descriptor.label,
        Style::default().fg(theme.muted_text),
    ));
    spans.push(Span::styled(
        descriptor.value,
        Style::default().fg(descriptor.color),
    ));
}

fn aqi_color(data: &WeatherMetricsData, theme: Theme) -> Color {
    super::hero_shared::aqi_color(theme, data.aqi_category, data.aqi_available)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cli::ThemeArg,
        domain::weather::{AirQualityCategory, WeatherCategory},
        ui::theme::{ColorCapability, theme_for},
    };

    fn sample_theme() -> Theme {
        theme_for(
            WeatherCategory::Clear,
            true,
            ColorCapability::TrueColor,
            ThemeArg::Auto,
        )
    }

    fn sample_data() -> WeatherMetricsData {
        WeatherMetricsData {
            feels: -3,
            humidity: 72,
            dew: -8,
            wind_dir: "NW",
            wind: 15,
            gust: 28,
            visibility: "10 km".to_string(),
            pressure: 1012,
            pressure_trend: "→",
            uv_today: "3".to_string(),
            cloud_total: 40,
            cloud_split: "40%".to_string(),
            precip_probability: "20%".to_string(),
            aqi: "—".to_string(),
            aqi_category: AirQualityCategory::Good,
            aqi_available: false,
        }
    }

    #[test]
    fn push_metric_lines_standard_produces_five_rows() {
        let mut lines: Vec<Line<'static>> = Vec::new();
        push_metric_lines(&mut lines, &sample_data(), sample_theme(), "  ", false);
        assert_eq!(lines.len(), STANDARD_ROWS.len());
    }

    #[test]
    fn push_metric_lines_compact_produces_four_rows() {
        let mut lines: Vec<Line<'static>> = Vec::new();
        push_metric_lines(&mut lines, &sample_data(), sample_theme(), "  ", true);
        assert_eq!(lines.len(), COMPACT_ROWS.len());
    }
}
