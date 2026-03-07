use chrono::{NaiveDateTime, NaiveTime, Timelike};

use super::{AutoPaletteAnchor, AutoPhase, AutoThemeSignal};
use crate::ui::theme::{Rgb, data::ThemeAppearance, extended::ThemePalette, mix_rgb};

#[derive(Debug, Clone, Copy)]
pub(super) struct AutoBlend {
    pub(super) phase: AutoPhase,
    top: Rgb,
    bottom: Rgb,
    accent: Rgb,
    ambient: Rgb,
}

#[derive(Debug, Clone, Copy)]
struct OverlayFactors {
    cloud: f32,
    rain: f32,
    snow: f32,
}

#[derive(Debug, Clone, Copy)]
struct SurfaceTokens {
    appearance: ThemeAppearance,
    surface: Rgb,
    surface_alt: Rgb,
    popup_surface: Rgb,
    text_hint: Rgb,
    muted_hint: Rgb,
}

#[derive(Debug, Clone, Copy)]
struct AccentTokens {
    info: Rgb,
    success: Rgb,
    danger: Rgb,
    temp_freezing: Rgb,
    temp_cold: Rgb,
    temp_mild: Rgb,
    temp_hot: Rgb,
    landmark_warm: Rgb,
    landmark_cool: Rgb,
    landmark_neutral: Rgb,
    particle: Rgb,
    border: Rgb,
    popup_border: Rgb,
    range_track: Rgb,
}

const ANCHOR_STOPS: [(AutoPhase, Rgb, Rgb, Rgb, Rgb); 8] = [
    (
        AutoPhase::PreDawn,
        (10, 18, 44),
        (24, 46, 92),
        (150, 190, 255),
        (72, 96, 160),
    ),
    (
        AutoPhase::Sunrise,
        (71, 42, 78),
        (236, 130, 88),
        (255, 225, 163),
        (244, 168, 122),
    ),
    (
        AutoPhase::Morning,
        (64, 141, 196),
        (170, 217, 234),
        (255, 218, 128),
        (119, 184, 221),
    ),
    (
        AutoPhase::SolarNoon,
        (38, 118, 189),
        (132, 190, 214),
        (255, 239, 184),
        (100, 168, 210),
    ),
    (
        AutoPhase::LateAfternoon,
        (45, 101, 145),
        (126, 176, 182),
        (255, 209, 128),
        (132, 165, 178),
    ),
    (
        AutoPhase::GoldenHour,
        (86, 68, 120),
        (228, 139, 84),
        (255, 216, 146),
        (194, 125, 102),
    ),
    (
        AutoPhase::BlueHour,
        (15, 34, 79),
        (37, 63, 124),
        (164, 201, 255),
        (88, 120, 186),
    ),
    (
        AutoPhase::DeepNight,
        (7, 13, 33),
        (16, 29, 60),
        (136, 170, 230),
        (62, 84, 138),
    ),
];

pub(super) fn auto_theme_palette(signal: &AutoThemeSignal<'_>) -> ThemePalette {
    let blended = apply_weather_overlays(signal, blend_auto_palette(signal));
    palette_from_blend(blended)
}

pub(super) fn blend_auto_palette(signal: &AutoThemeSignal<'_>) -> AutoBlend {
    let anchors = build_anchors(signal);
    let now_minutes = minutes_since_midnight(signal.now_local);

    for idx in 0..anchors.len() {
        let start = anchors[idx];
        let end = anchors[(idx + 1) % anchors.len()];
        let start_minutes = start.minutes;
        let mut end_minutes = end.minutes;
        let mut now = now_minutes;

        if idx == anchors.len() - 1 {
            end_minutes += 1440;
            if now < start_minutes {
                now += 1440;
            }
        }

        if now >= start_minutes && now <= end_minutes {
            return interpolated_blend(start, end, start_minutes, end_minutes, now);
        }
    }

    let fallback = anchors[0];
    AutoBlend {
        phase: fallback.phase,
        top: fallback.top,
        bottom: fallback.bottom,
        accent: fallback.accent,
        ambient: fallback.ambient,
    }
}

fn apply_weather_overlays(signal: &AutoThemeSignal<'_>, mut blended: AutoBlend) -> AutoBlend {
    let factors = overlay_factors(signal);
    apply_cloud_haze(&mut blended, factors.cloud);
    apply_precip_overlay(
        &mut blended,
        super::signal::rain_now_or_soon(signal),
        factors.rain,
        (76, 126, 196),
        0.85,
    );
    apply_precip_overlay(
        &mut blended,
        super::signal::snow_now_or_soon(signal),
        factors.snow,
        (170, 220, 255),
        0.90,
    );
    apply_optional_overlay(
        &mut blended,
        super::signal::fog_now(signal),
        (154, 165, 180),
        0.18,
        false,
    );
    apply_thunder_overlay(signal, &mut blended);
    if signal.clearing_soon {
        blended.accent = mix_rgb(blended.accent, (255, 214, 140), 0.10);
    }
    blended
}

fn overlay_factors(signal: &AutoThemeSignal<'_>) -> OverlayFactors {
    let clear_scale = if signal.clearing_soon { 0.8 } else { 1.0 };
    OverlayFactors {
        cloud: signal
            .current_cloud_cover
            .max(signal.max_cloud_cover_6h)
            .clamp(0.0, 100.0)
            / 100.0
            * 0.18,
        rain: (signal.max_precip_mm_6h / 4.0).clamp(0.0, 1.0) * 0.32 * clear_scale,
        snow: (signal.max_snow_cm_6h / 2.0).clamp(0.0, 1.0) * 0.32 * clear_scale,
    }
}

fn apply_cloud_haze(blended: &mut AutoBlend, factor: f32) {
    blended.top = mix_rgb(blended.top, (160, 176, 196), factor);
    blended.bottom = mix_rgb(blended.bottom, (160, 176, 196), factor);
    blended.ambient = mix_rgb(blended.ambient, (160, 176, 196), factor * 0.85);
}

fn apply_precip_overlay(
    blended: &mut AutoBlend,
    active: bool,
    factor: f32,
    tint: Rgb,
    ambient_factor: f32,
) {
    if !active {
        return;
    }
    blended.top = mix_rgb(blended.top, tint, factor);
    blended.bottom = mix_rgb(blended.bottom, tint, factor);
    blended.ambient = mix_rgb(blended.ambient, tint, factor * ambient_factor);
}

fn apply_optional_overlay(
    blended: &mut AutoBlend,
    active: bool,
    tint: Rgb,
    factor: f32,
    accent_shift: bool,
) {
    if !active {
        return;
    }
    blended.top = mix_rgb(blended.top, tint, factor);
    blended.bottom = mix_rgb(blended.bottom, tint, factor);
    blended.ambient = mix_rgb(blended.ambient, tint, factor);
    if accent_shift {
        blended.accent = mix_rgb(blended.accent, (190, 210, 255), 0.12);
    }
}

fn apply_thunder_overlay(signal: &AutoThemeSignal<'_>, blended: &mut AutoBlend) {
    apply_optional_overlay(
        blended,
        signal.thunder_soon || signal.strong_wind_soon,
        (72, 64, 132),
        0.28,
        true,
    );
    if signal.thunder_soon || signal.strong_wind_soon {
        blended.ambient = mix_rgb(blended.ambient, (72, 64, 132), 0.22);
    }
}

fn palette_from_blend(blended: AutoBlend) -> ThemePalette {
    let top = blended.top;
    let bottom = blended.bottom;
    let accent = blended.accent;
    let ambient = blended.ambient;
    let surface_tokens = surface_tokens(top, bottom, ambient);
    let muted_hint = surface_tokens.muted_hint;
    let accent_tokens = accent_tokens(surface_tokens, accent, ambient);

    ThemePalette {
        appearance: surface_tokens.appearance,
        top,
        bottom,
        surface: surface_tokens.surface,
        surface_alt: surface_tokens.surface_alt,
        popup_surface: surface_tokens.popup_surface,
        accent,
        text_hint: surface_tokens.text_hint,
        muted_hint,
        info: accent_tokens.info,
        success: accent_tokens.success,
        warning: (255, 191, 107),
        danger: accent_tokens.danger,
        temp_freezing: accent_tokens.temp_freezing,
        temp_cold: accent_tokens.temp_cold,
        temp_mild: accent_tokens.temp_mild,
        temp_warm: (255, 201, 109),
        temp_hot: accent_tokens.temp_hot,
        landmark_warm: accent_tokens.landmark_warm,
        landmark_cool: accent_tokens.landmark_cool,
        landmark_neutral: accent_tokens.landmark_neutral,
        particle: Some(accent_tokens.particle),
        border: Some(accent_tokens.border),
        popup_border: Some(accent_tokens.popup_border),
        range_track: Some(accent_tokens.range_track),
    }
}

fn surface_tokens(top: Rgb, bottom: Rgb, ambient: Rgb) -> SurfaceTokens {
    let appearance = appearance_for(top, bottom);
    let base_surface = mix_rgb(top, bottom, 0.78);
    let surface = mix_rgb(base_surface, ambient, surface_mix(appearance));
    let surface_alt = mix_rgb(
        mix_rgb(top, bottom, 0.60),
        ambient,
        alt_surface_mix(appearance),
    );

    SurfaceTokens {
        appearance,
        surface,
        surface_alt,
        popup_surface: popup_surface_for(surface_alt, appearance),
        text_hint: text_hint_for(appearance),
        muted_hint: muted_hint_for(appearance, ambient),
    }
}

fn accent_tokens(surface_tokens: SurfaceTokens, accent: Rgb, ambient: Rgb) -> AccentTokens {
    AccentTokens {
        info: mix_rgb((98, 179, 255), ambient, 0.12),
        success: mix_rgb((102, 214, 162), ambient, 0.08),
        danger: mix_rgb((244, 113, 135), ambient, 0.10),
        temp_freezing: mix_rgb((162, 211, 255), ambient, 0.12),
        temp_cold: mix_rgb((98, 191, 255), ambient, 0.10),
        temp_mild: mix_rgb((126, 219, 171), ambient, 0.08),
        temp_hot: mix_rgb((247, 123, 111), ambient, 0.08),
        landmark_warm: mix_rgb((255, 220, 150), accent, 0.16),
        landmark_cool: mix_rgb((152, 206, 255), ambient, 0.22),
        landmark_neutral: mix_rgb(
            surface_tokens.muted_hint,
            ambient,
            landmark_neutral_mix(surface_tokens.appearance),
        ),
        particle: particle_for(surface_tokens.appearance, ambient),
        border: border_for(
            surface_tokens.appearance,
            surface_tokens.surface_alt,
            ambient,
            accent,
            0.28,
            0.18,
        ),
        popup_border: border_for(
            surface_tokens.appearance,
            surface_tokens.popup_surface,
            ambient,
            accent,
            0.24,
            0.24,
        ),
        range_track: mix_rgb(surface_tokens.muted_hint, ambient, 0.12),
    }
}

fn interpolated_blend(
    start: AutoPaletteAnchor,
    end: AutoPaletteAnchor,
    start_minutes: u16,
    end_minutes: u16,
    now: u16,
) -> AutoBlend {
    let span = (end_minutes - start_minutes).max(1) as f32;
    let t = (now - start_minutes) as f32 / span;
    AutoBlend {
        phase: start.phase,
        top: mix_rgb(start.top, end.top, t),
        bottom: mix_rgb(start.bottom, end.bottom, t),
        accent: mix_rgb(start.accent, end.accent, t),
        ambient: mix_rgb(start.ambient, end.ambient, t),
    }
}

fn build_anchors(signal: &AutoThemeSignal<'_>) -> [AutoPaletteAnchor; 8] {
    let minutes = daylight_anchor_minutes(signal);
    std::array::from_fn(|index| anchor_stop(index, minutes[index]))
}

fn anchor_stop(index: usize, minutes: u16) -> AutoPaletteAnchor {
    let (phase, top, bottom, accent, ambient) = ANCHOR_STOPS[index];
    anchor(phase, minutes, top, bottom, accent, ambient)
}

fn daylight_anchor_minutes(signal: &AutoThemeSignal<'_>) -> [u16; 8] {
    match (signal.sunrise_today, signal.sunset_today) {
        (Some(sunrise_today), Some(sunset_today)) => {
            let sunrise = minutes_since_midnight(sunrise_today);
            let sunset = minutes_since_midnight(sunset_today);
            let midpoint = sunrise + (sunset.saturating_sub(sunrise)) / 2;
            [
                sunrise.saturating_sub(90),
                sunrise,
                sunrise.saturating_add(120),
                midpoint,
                sunset.saturating_sub(120),
                sunset.saturating_sub(45),
                sunset.saturating_add(30),
                sunset.saturating_add(150),
            ]
        }
        _ => fallback_minutes(signal.now_local),
    }
}

fn anchor(
    phase: AutoPhase,
    minutes: u16,
    top: Rgb,
    bottom: Rgb,
    accent: Rgb,
    ambient: Rgb,
) -> AutoPaletteAnchor {
    AutoPaletteAnchor {
        phase,
        minutes,
        top,
        bottom,
        accent,
        ambient,
    }
}

fn fallback_minutes(now_local: NaiveDateTime) -> [u16; 8] {
    let date = now_local.date();
    [
        clock_minutes(date.and_time(NaiveTime::from_hms_opt(4, 30, 0).expect("valid time"))),
        clock_minutes(date.and_time(NaiveTime::from_hms_opt(6, 30, 0).expect("valid time"))),
        clock_minutes(date.and_time(NaiveTime::from_hms_opt(9, 0, 0).expect("valid time"))),
        clock_minutes(date.and_time(NaiveTime::from_hms_opt(13, 0, 0).expect("valid time"))),
        clock_minutes(date.and_time(NaiveTime::from_hms_opt(17, 0, 0).expect("valid time"))),
        clock_minutes(date.and_time(NaiveTime::from_hms_opt(18, 30, 0).expect("valid time"))),
        clock_minutes(date.and_time(NaiveTime::from_hms_opt(19, 30, 0).expect("valid time"))),
        clock_minutes(date.and_time(NaiveTime::from_hms_opt(22, 0, 0).expect("valid time"))),
    ]
}

fn clock_minutes(value: NaiveDateTime) -> u16 {
    minutes_since_midnight(value)
}

fn minutes_since_midnight(value: NaiveDateTime) -> u16 {
    let minutes = value.hour() * 60 + value.minute();
    u16::try_from(minutes).unwrap_or(u16::MAX)
}

fn appearance_for(top: Rgb, bottom: Rgb) -> ThemeAppearance {
    let avg_luma = ((top.0 as f32 + bottom.0 as f32) + (top.1 as f32 + bottom.1 as f32)) / 4.0;
    if avg_luma >= 150.0 {
        ThemeAppearance::Light
    } else {
        ThemeAppearance::Dark
    }
}

fn popup_surface_for(surface_alt: Rgb, appearance: ThemeAppearance) -> Rgb {
    if appearance.is_light() {
        mix_rgb(surface_alt, (246, 248, 251), 0.14)
    } else {
        mix_rgb(surface_alt, (11, 16, 28), 0.18)
    }
}

fn text_hint_for(appearance: ThemeAppearance) -> Rgb {
    if appearance.is_light() {
        (16, 22, 34)
    } else {
        (236, 241, 250)
    }
}

fn muted_hint_for(appearance: ThemeAppearance, ambient: Rgb) -> Rgb {
    if appearance.is_light() {
        (77, 91, 112)
    } else {
        mix_rgb((184, 194, 209), ambient, 0.10)
    }
}

fn surface_mix(appearance: ThemeAppearance) -> f32 {
    if appearance.is_light() { 0.18 } else { 0.16 }
}

fn alt_surface_mix(appearance: ThemeAppearance) -> f32 {
    if appearance.is_light() { 0.28 } else { 0.24 }
}

fn landmark_neutral_mix(appearance: ThemeAppearance) -> f32 {
    if appearance.is_light() { 0.18 } else { 0.24 }
}

fn particle_for(appearance: ThemeAppearance, ambient: Rgb) -> Rgb {
    if appearance.is_light() {
        mix_rgb(ambient, (255, 255, 255), 0.24)
    } else {
        mix_rgb(ambient, (210, 226, 245), 0.32)
    }
}

fn border_for(
    appearance: ThemeAppearance,
    surface: Rgb,
    ambient: Rgb,
    accent: Rgb,
    light_mix: f32,
    dark_mix: f32,
) -> Rgb {
    if appearance.is_light() {
        mix_rgb(surface, ambient, light_mix)
    } else {
        mix_rgb(surface, accent, dark_mix)
    }
}
