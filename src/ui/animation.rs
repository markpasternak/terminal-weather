use std::{
    collections::hash_map::DefaultHasher,
    f32::consts::TAU,
    hash::{Hash, Hasher},
    time::Duration,
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::domain::weather::{
    CurrentConditions, ForecastBundle, WeatherCategory, weather_code_to_category,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum, Default)]
pub enum MotionMode {
    #[default]
    Cinematic,
    Standard,
    Reduced,
    Off,
}

impl MotionMode {
    #[must_use]
    pub const fn allows_animation(self) -> bool {
        !matches!(self, Self::Off)
    }

    #[must_use]
    pub const fn allows_transitions(self) -> bool {
        matches!(self, Self::Cinematic | Self::Standard)
    }

    #[must_use]
    pub const fn allows_flash(self) -> bool {
        matches!(self, Self::Cinematic | Self::Standard)
    }

    #[must_use]
    pub const fn is_cinematic(self) -> bool {
        matches!(self, Self::Cinematic)
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cinematic => "Cinematic",
            Self::Standard => "Standard",
            Self::Reduced => "Reduced",
            Self::Off => "Off",
        }
    }

    #[must_use]
    pub const fn legacy_flags(self) -> (bool, bool) {
        match self {
            Self::Cinematic | Self::Standard => (false, false),
            Self::Reduced => (false, true),
            Self::Off => (true, true),
        }
    }

    #[must_use]
    pub const fn particle_density_scale(self) -> f32 {
        match self {
            Self::Cinematic => 1.0,
            Self::Standard => 0.72,
            Self::Reduced => 0.32,
            Self::Off => 0.0,
        }
    }

    #[must_use]
    pub const fn transition_scale(self) -> f32 {
        match self {
            Self::Cinematic => 1.0,
            Self::Standard => 0.75,
            Self::Reduced | Self::Off => 0.0,
        }
    }

    #[must_use]
    pub const fn from_legacy(no_animation: bool, reduced_motion: bool) -> Self {
        if no_animation {
            Self::Off
        } else if reduced_motion {
            Self::Reduced
        } else {
            Self::Cinematic
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationClockState {
    pub dt_seconds: f32,
    pub elapsed_seconds: f32,
    pub frame_index: u64,
}

impl Default for AnimationClockState {
    fn default() -> Self {
        Self {
            dt_seconds: 0.0,
            elapsed_seconds: 0.0,
            frame_index: 0,
        }
    }
}

impl AnimationClockState {
    pub fn advance(&mut self, delta: Duration) {
        let dt_seconds = delta.as_secs_f32().clamp(0.0, 0.25);
        self.dt_seconds = dt_seconds;
        self.elapsed_seconds += dt_seconds;
        self.frame_index = self.frame_index.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityBand {
    Open,
    Muted,
    Obscured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudDensity {
    Sparse,
    Layered,
    Dense,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherMotionProfile {
    pub category: WeatherCategory,
    pub intensity: f32,
    pub wind_strength: f32,
    pub wind_drift_x: f32,
    pub gustiness: f32,
    pub visibility_band: VisibilityBand,
    pub cloud_density: CloudDensity,
    pub is_day: bool,
    pub is_transitioning: bool,
}

impl WeatherMotionProfile {
    #[must_use]
    pub fn from_bundle(bundle: &ForecastBundle, is_transitioning: bool) -> Self {
        Self::from_current(&bundle.current, is_transitioning)
    }

    #[must_use]
    pub fn from_current(current: &CurrentConditions, is_transitioning: bool) -> Self {
        let category = weather_code_to_category(current.weather_code);
        let intensity = weather_intensity(current, category);
        let wind_strength = (current.wind_speed_10m / 24.0).clamp(0.0, 1.0);
        let wind_drift_x = drift_from_direction(current.wind_speed_10m, current.wind_direction_10m);
        let gustiness =
            ((current.wind_gusts_10m - current.wind_speed_10m).max(0.0) / 18.0).clamp(0.0, 1.0);
        let visibility_band = visibility_band(current.visibility_m);
        let cloud_density = cloud_density(current.cloud_cover);

        Self {
            category,
            intensity,
            wind_strength,
            wind_drift_x,
            gustiness,
            visibility_band,
            cloud_density,
            is_day: current.is_day,
            is_transitioning,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionKind {
    AppReady,
    CitySwitch,
    FetchReveal,
    HeroVisualSwitch,
    FreshnessPulse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneTransitionState {
    pub kind: TransitionKind,
    elapsed_seconds: f32,
    duration_seconds: f32,
}

impl SceneTransitionState {
    #[must_use]
    pub fn new(kind: TransitionKind, duration_seconds: f32) -> Self {
        Self {
            kind,
            elapsed_seconds: 0.0,
            duration_seconds: duration_seconds.max(0.01),
        }
    }

    #[must_use]
    pub fn app_ready(mode: MotionMode) -> Option<Self> {
        transition_for_mode(mode, TransitionKind::AppReady, 0.30)
    }

    #[must_use]
    pub fn city_switch(mode: MotionMode) -> Option<Self> {
        transition_for_mode(mode, TransitionKind::CitySwitch, 0.32)
    }

    #[must_use]
    pub fn fetch_reveal(mode: MotionMode) -> Option<Self> {
        transition_for_mode(mode, TransitionKind::FetchReveal, 0.24)
    }

    #[must_use]
    pub fn hero_visual_switch(mode: MotionMode) -> Option<Self> {
        transition_for_mode(mode, TransitionKind::HeroVisualSwitch, 0.22)
    }

    #[must_use]
    pub fn freshness_pulse(mode: MotionMode) -> Option<Self> {
        transition_for_mode(mode, TransitionKind::FreshnessPulse, 0.18)
    }

    pub fn advance(&mut self, dt_seconds: f32) {
        self.elapsed_seconds = (self.elapsed_seconds + dt_seconds).min(self.duration_seconds);
    }

    #[must_use]
    pub fn finished(self) -> bool {
        self.elapsed_seconds >= self.duration_seconds
    }

    #[must_use]
    pub fn progress(self) -> f32 {
        (self.elapsed_seconds / self.duration_seconds).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn eased_progress(self) -> f32 {
        let t = self.progress();
        match self.kind {
            TransitionKind::AppReady | TransitionKind::FetchReveal => ease_out_cubic(t),
            TransitionKind::CitySwitch | TransitionKind::HeroVisualSwitch => ease_in_out_sine(t),
            TransitionKind::FreshnessPulse => ease_out_quad(t),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SeededMotion {
    seed: u64,
}

impl SeededMotion {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    #[must_use]
    pub fn lane(self, label: &str) -> Self {
        Self::new(stable_hash(&(self.seed, label)))
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn unit(self, index: u64) -> f32 {
        let hashed = stable_hash(&(self.seed, index));
        (hashed as f64 / u64::MAX as f64) as f32
    }

    #[must_use]
    pub fn signed(self, index: u64) -> f32 {
        self.unit(index) * 2.0 - 1.0
    }

    #[must_use]
    pub fn pulse(self, elapsed_seconds: f32, speed: f32, index: u64) -> f32 {
        let phase = elapsed_seconds * speed + self.unit(index) * TAU;
        phase.sin() * 0.5 + 0.5
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiMotionContext {
    pub elapsed_seconds: f32,
    pub dt_seconds: f32,
    pub frame_index: u64,
    pub motion_mode: MotionMode,
    pub seed: SeededMotion,
    pub weather_profile: Option<WeatherMotionProfile>,
    pub transition_progress: Option<f32>,
    pub animate: bool,
}

impl UiMotionContext {
    #[must_use]
    pub fn transition_mix(self) -> f32 {
        self.transition_progress.unwrap_or(1.0)
    }

    #[must_use]
    pub fn lane(self, label: &str) -> SeededMotion {
        self.seed.lane(label)
    }
}

#[must_use]
pub fn stable_hash(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn transition_for_mode(
    mode: MotionMode,
    kind: TransitionKind,
    duration_seconds: f32,
) -> Option<SceneTransitionState> {
    mode.allows_transitions()
        .then_some(SceneTransitionState::new(
            kind,
            duration_seconds * mode.transition_scale().max(0.4),
        ))
}

fn weather_intensity(current: &CurrentConditions, category: WeatherCategory) -> f32 {
    let precip = (current.precipitation_mm / 6.0).clamp(0.0, 1.0);
    let cloud = (current.cloud_cover / 100.0).clamp(0.0, 1.0);
    let wind = (current.wind_speed_10m / 20.0).clamp(0.0, 1.0);
    match category {
        WeatherCategory::Clear => (wind * 0.35).clamp(0.0, 0.4),
        WeatherCategory::Cloudy => (cloud * 0.85 + wind * 0.2).clamp(0.2, 0.9),
        WeatherCategory::Rain => (precip * 0.8 + wind * 0.35).clamp(0.25, 1.0),
        WeatherCategory::Snow => ((current.precipitation_mm / 3.5) + wind * 0.2).clamp(0.2, 0.9),
        WeatherCategory::Fog => {
            ((1.0 - (current.visibility_m / 16_000.0).clamp(0.0, 1.0)) * 0.9).clamp(0.3, 1.0)
        }
        WeatherCategory::Thunder => (0.75 + precip * 0.2 + wind * 0.2).clamp(0.75, 1.0),
        WeatherCategory::Unknown => 0.0,
    }
}

fn drift_from_direction(wind_speed: f32, wind_direction: f32) -> f32 {
    let drift_base = (wind_speed / 36.0).clamp(0.0, 1.0);
    let drift_sign = wind_direction.to_radians().sin().signum();
    drift_base * drift_sign
}

fn visibility_band(visibility_m: f32) -> VisibilityBand {
    match visibility_m {
        v if v < 2_000.0 => VisibilityBand::Obscured,
        v if v < 7_000.0 => VisibilityBand::Muted,
        _ => VisibilityBand::Open,
    }
}

fn cloud_density(cloud_cover: f32) -> CloudDensity {
    match cloud_cover {
        c if c < 25.0 => CloudDensity::Sparse,
        c if c < 70.0 => CloudDensity::Layered,
        _ => CloudDensity::Dense,
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

fn ease_in_out_sine(t: f32) -> f32 {
    -((std::f32::consts::PI * t).cos() - 1.0) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_mode_legacy_flags_map_as_expected() {
        assert_eq!(MotionMode::from_legacy(false, false), MotionMode::Cinematic);
        assert_eq!(MotionMode::from_legacy(false, true), MotionMode::Reduced);
        assert_eq!(MotionMode::from_legacy(true, false), MotionMode::Off);
    }

    #[test]
    fn animation_clock_advances_elapsed_and_frame_index() {
        let mut clock = AnimationClockState::default();
        clock.advance(Duration::from_millis(20));
        assert!(clock.elapsed_seconds > 0.0);
        assert_eq!(clock.frame_index, 1);
    }

    #[test]
    fn seeded_motion_is_stable() {
        let seed = SeededMotion::new(42).lane("rain");
        let same_lane = SeededMotion::new(42).lane("rain").unit(7);
        let other_lane = SeededMotion::new(42).lane("snow").unit(7);
        assert!((seed.unit(7) - same_lane).abs() < f32::EPSILON);
        assert!((seed.unit(7) - other_lane).abs() > f32::EPSILON);
    }

    #[test]
    fn weather_motion_profile_uses_visibility_and_cloud_thresholds() {
        let current = CurrentConditions {
            temperature_2m_c: 4.0,
            relative_humidity_2m: 90.0,
            apparent_temperature_c: 2.0,
            dew_point_2m_c: 1.0,
            weather_code: 45,
            precipitation_mm: 0.1,
            cloud_cover: 92.0,
            pressure_msl_hpa: 1000.0,
            visibility_m: 1_500.0,
            wind_speed_10m: 3.0,
            wind_gusts_10m: 4.0,
            wind_direction_10m: 90.0,
            is_day: false,
            high_today_c: None,
            low_today_c: None,
        };

        let profile = WeatherMotionProfile::from_current(&current, true);
        assert_eq!(profile.visibility_band, VisibilityBand::Obscured);
        assert_eq!(profile.cloud_density, CloudDensity::Dense);
        assert!(profile.is_transitioning);
    }

    #[test]
    fn transition_progress_uses_easing() {
        let mut transition = SceneTransitionState::new(TransitionKind::FetchReveal, 0.2);
        transition.advance(0.1);
        assert!(transition.progress() > 0.4);
        assert!(transition.eased_progress() > transition.progress());
    }
}
