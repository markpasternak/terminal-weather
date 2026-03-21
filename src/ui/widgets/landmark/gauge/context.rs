use super::data::GaugeData;

pub(super) fn gauge_context_line(data: &GaugeData) -> String {
    const RULES: [fn(&GaugeData) -> Option<String>; 8] = [
        critical_uv_line,
        severe_gust_line,
        high_uv_line,
        gusty_line,
        low_visibility_line,
        active_precip_line,
        muggy_line,
        damp_line,
    ];
    for rule in RULES {
        if let Some(line) = rule(data) {
            return line;
        }
    }
    format!("All readings nominal · {:.0} hPa", data.pressure)
}

fn critical_uv_line(data: &GaugeData) -> Option<String> {
    simple_context_line(data.uv > 7.0, || {
        format!("⚠ UV {:.1} very high — limit sun exposure", data.uv)
    })
}

fn severe_gust_line(data: &GaugeData) -> Option<String> {
    wind_line(data, 50.0, "⚠ Gusts", "— secure loose objects")
}

fn high_uv_line(data: &GaugeData) -> Option<String> {
    simple_context_line(data.uv > 5.0, || {
        format!("UV {:.1} high · sunscreen advised", data.uv)
    })
}

fn gusty_line(data: &GaugeData) -> Option<String> {
    wind_line(data, 30.0, "Gusty winds", "· dress for wind")
}

fn low_visibility_line(data: &GaugeData) -> Option<String> {
    (data.vis_km < 1.0).then(|| format!("Visibility {:.1}km · reduced visibility", data.vis_km))
}

fn active_precip_line(data: &GaugeData) -> Option<String> {
    simple_context_line(data.precip_now > 0.5, || {
        format!("Active precipitation {:.1}mm", data.precip_now)
    })
}

fn muggy_line(data: &GaugeData) -> Option<String> {
    simple_context_line(data.humidity > 85.0 && data.temp_c > 15, || {
        format!("Humidity {:.0}% · feels muggy", data.humidity)
    })
}

fn damp_line(data: &GaugeData) -> Option<String> {
    simple_context_line(data.humidity > 85.0, || {
        format!("Humidity {:.0}% · feels damp", data.humidity)
    })
}

fn simple_context_line(condition: bool, message: impl FnOnce() -> String) -> Option<String> {
    condition.then(message)
}

fn wind_line(data: &GaugeData, min_gust: f32, prefix: &str, suffix: &str) -> Option<String> {
    simple_context_line(data.gust > min_gust, || {
        format!(
            "{prefix} {} m/s {suffix}",
            crate::domain::weather::round_wind_speed(data.gust)
        )
    })
}
