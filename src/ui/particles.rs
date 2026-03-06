use std::time::Duration;

use crate::{
    domain::weather::WeatherCategory,
    ui::animation::{MotionMode, SeededMotion, WeatherMotionProfile},
};

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub glyph: char,
    pub ttl: f32,
    pub age: f32,
}

#[derive(Debug)]
pub struct ParticleEngine {
    motion_mode: MotionMode,
    no_flash: bool,
    pub particles: Vec<Particle>,
    accumulator: f32,
    spawn_counter: u64,
    storm_charge: f32,
    flash_timer: f32,
    storm_cooldown: f32,
}

impl ParticleEngine {
    #[must_use]
    pub fn new(motion_mode: MotionMode, no_flash: bool) -> Self {
        Self {
            motion_mode,
            no_flash,
            particles: Vec::new(),
            accumulator: 0.0,
            spawn_counter: 0,
            storm_charge: 0.0,
            flash_timer: 0.0,
            storm_cooldown: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.particles.clear();
        self.accumulator = 0.0;
        self.spawn_counter = 0;
        self.storm_charge = 0.0;
        self.flash_timer = 0.0;
        self.storm_cooldown = 0.0;
    }

    pub fn set_options(&mut self, motion_mode: MotionMode, no_flash: bool) {
        let mode_changed = self.motion_mode != motion_mode;
        self.motion_mode = motion_mode;
        self.no_flash = no_flash;
        if mode_changed || matches!(motion_mode, MotionMode::Off) {
            self.reset();
        }
    }

    #[must_use]
    pub fn flash_active(&self) -> bool {
        !self.no_flash && self.motion_mode.allows_flash() && self.flash_timer > 0.0
    }

    pub fn update(&mut self, profile: Option<WeatherMotionProfile>, dt: Duration, seed: u64) {
        if !self.motion_mode.allows_animation() {
            self.reset();
            return;
        }

        let dt = dt.as_secs_f32().clamp(0.0, 0.25);
        self.accumulator += dt;

        let Some(profile) = profile else {
            self.advance_particles(dt);
            self.flash_timer = (self.flash_timer - dt).max(0.0);
            return;
        };

        self.maybe_trigger_flash(profile, dt, seed);
        self.spawn_for_profile(profile, seed);
        self.advance_particles(dt);
        self.flash_timer = (self.flash_timer - dt).max(0.0);
    }

    fn spawn_for_profile(&mut self, profile: WeatherMotionProfile, seed: u64) {
        let interval = spawn_interval(self.motion_mode, profile.category);
        while self.accumulator >= interval {
            self.accumulator -= interval;
            self.spawn_batch(profile, seed);
        }
    }

    fn spawn_batch(&mut self, profile: WeatherMotionProfile, seed: u64) {
        let base_seed = SeededMotion::new(seed).lane(category_label(profile.category));
        let density_scale = self.motion_mode.particle_density_scale();
        let count = particle_count(profile, density_scale);

        for lane in 0..count {
            let emitter = base_seed.lane(match lane % 4 {
                0 => "front",
                1 => "mid",
                2 => "back",
                _ => "accent",
            });
            if let Some(particle) = spawn_particle(
                profile,
                emitter,
                self.spawn_counter + lane as u64,
                self.motion_mode,
            ) {
                self.particles.push(particle);
            }
        }
        self.spawn_counter = self.spawn_counter.saturating_add(count as u64);
    }

    fn advance_particles(&mut self, dt: f32) {
        let speed = if self.motion_mode.is_cinematic() {
            1.0
        } else {
            0.86
        };
        self.particles.retain_mut(|particle| {
            particle.age += dt;
            particle.x += particle.vx * dt * 40.0 * speed;
            particle.y += particle.vy * dt * 40.0 * speed;
            particle.age < particle.ttl
                && particle.y < 1.25
                && particle.x > -0.3
                && particle.x < 1.3
        });
    }

    fn maybe_trigger_flash(&mut self, profile: WeatherMotionProfile, dt: f32, seed: u64) {
        if profile.category != WeatherCategory::Thunder
            || self.no_flash
            || !self.motion_mode.allows_flash()
        {
            self.storm_charge = 0.0;
            self.storm_cooldown = (self.storm_cooldown - dt).max(0.0);
            return;
        }

        self.storm_cooldown = (self.storm_cooldown - dt).max(0.0);
        self.storm_charge += dt * (0.45 + profile.intensity * 0.85 + profile.gustiness * 0.35);
        if self.storm_cooldown > 0.0 {
            return;
        }

        let storm_seed = SeededMotion::new(seed).lane("storm");
        let threshold = 0.55 + storm_seed.unit(self.spawn_counter / 2 + 11) * 0.95;
        if self.storm_charge < threshold {
            return;
        }

        self.flash_timer = 0.08 + profile.intensity * 0.08;
        self.storm_charge = 0.0;
        self.storm_cooldown = 0.35 + storm_seed.unit(self.spawn_counter + 97) * 1.1;
    }
}

fn spawn_interval(mode: MotionMode, category: WeatherCategory) -> f32 {
    scale_spawn_interval(base_spawn_interval(category), mode)
}

fn base_spawn_interval(category: WeatherCategory) -> f32 {
    match category {
        WeatherCategory::Rain | WeatherCategory::Thunder => 0.030,
        WeatherCategory::Snow => 0.050,
        WeatherCategory::Fog => 0.075,
        WeatherCategory::Cloudy => 0.090,
        WeatherCategory::Clear | WeatherCategory::Unknown => 0.120,
    }
}

fn scale_spawn_interval(base: f32, mode: MotionMode) -> f32 {
    match mode {
        MotionMode::Cinematic => base,
        MotionMode::Standard => base * 1.15,
        MotionMode::Reduced => base * 1.8,
        MotionMode::Off => f32::INFINITY,
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn particle_count(profile: WeatherMotionProfile, density_scale: f32) -> usize {
    let base = match profile.category {
        WeatherCategory::Clear => 1,
        WeatherCategory::Cloudy | WeatherCategory::Fog => 2,
        WeatherCategory::Rain => 4,
        WeatherCategory::Snow => 3,
        WeatherCategory::Thunder => 5,
        WeatherCategory::Unknown => 0,
    };

    let scaled = (base as f32
        + profile.intensity * 5.0
        + profile.wind_strength * 2.0
        + profile.gustiness * 1.5)
        * density_scale;
    scaled.round().clamp(0.0, 10.0) as usize
}

fn spawn_particle(
    profile: WeatherMotionProfile,
    seed: SeededMotion,
    index: u64,
    motion_mode: MotionMode,
) -> Option<Particle> {
    match profile.category {
        WeatherCategory::Rain | WeatherCategory::Thunder => {
            Some(spawn_rain_particle(profile, seed, index, motion_mode))
        }
        WeatherCategory::Snow => Some(spawn_snow_particle(profile, seed, index, motion_mode)),
        WeatherCategory::Fog => Some(spawn_fog_particle(profile, seed, index)),
        WeatherCategory::Cloudy => spawn_cloud_mote(profile, seed, index, motion_mode),
        WeatherCategory::Clear => spawn_clear_mote(profile, seed, index, motion_mode),
        WeatherCategory::Unknown => None,
    }
}

fn spawn_rain_particle(
    profile: WeatherMotionProfile,
    seed: SeededMotion,
    index: u64,
    motion_mode: MotionMode,
) -> Particle {
    let cinematic = motion_mode.is_cinematic();
    let x = seed.unit(index);
    let lane_bias = seed.signed(index + 1) * 0.006;
    let vy = 0.020 + profile.intensity * 0.014 + seed.unit(index + 2) * 0.008;
    let vx = profile.wind_drift_x * 0.020 + lane_bias;
    let glyph = if cinematic && seed.unit(index + 3) > 0.72 {
        '/'
    } else if seed.unit(index + 4) > 0.55 {
        '╱'
    } else {
        '│'
    };
    Particle {
        x,
        y: -seed.unit(index + 5) * 0.12,
        vx,
        vy,
        glyph,
        ttl: 1.2,
        age: 0.0,
    }
}

fn spawn_snow_particle(
    profile: WeatherMotionProfile,
    seed: SeededMotion,
    index: u64,
    motion_mode: MotionMode,
) -> Particle {
    let x = seed.unit(index);
    let sway = seed.signed(index + 1) * 0.010;
    let wind = profile.wind_drift_x * 0.008;
    let vy = 0.004 + seed.unit(index + 2) * 0.004;
    let glyph = if motion_mode.is_cinematic() && !profile.is_day && seed.unit(index + 3) > 0.72 {
        '✦'
    } else if seed.unit(index + 4) > 0.55 {
        '*'
    } else {
        '•'
    };
    Particle {
        x,
        y: -seed.unit(index + 5) * 0.10,
        vx: wind + sway,
        vy,
        glyph,
        ttl: 3.4,
        age: 0.0,
    }
}

fn spawn_fog_particle(profile: WeatherMotionProfile, seed: SeededMotion, index: u64) -> Particle {
    let glyph = if seed.unit(index + 3) > 0.65 {
        '░'
    } else {
        '·'
    };
    Particle {
        x: seed.unit(index),
        y: 0.20 + seed.unit(index + 1) * 0.70,
        vx: profile.wind_drift_x * 0.010 + seed.signed(index + 2) * 0.0025,
        vy: seed.signed(index + 4) * 0.0005,
        glyph,
        ttl: 4.8,
        age: 0.0,
    }
}

fn spawn_cloud_mote(
    profile: WeatherMotionProfile,
    seed: SeededMotion,
    index: u64,
    motion_mode: MotionMode,
) -> Option<Particle> {
    if matches!(motion_mode, MotionMode::Reduced | MotionMode::Off) {
        return None;
    }

    let glyph = if seed.unit(index + 1) > 0.58 {
        '░'
    } else {
        '·'
    };
    Some(Particle {
        x: seed.unit(index),
        y: 0.08 + seed.unit(index + 2) * 0.40,
        vx: profile.wind_drift_x * 0.007 + 0.002 + seed.signed(index + 3) * 0.0015,
        vy: seed.signed(index + 4) * 0.0004,
        glyph,
        ttl: 5.6,
        age: 0.0,
    })
}

fn spawn_clear_mote(
    profile: WeatherMotionProfile,
    seed: SeededMotion,
    index: u64,
    motion_mode: MotionMode,
) -> Option<Particle> {
    if !motion_mode.is_cinematic() || !profile.is_day || profile.wind_strength > 0.55 {
        return None;
    }

    Some(Particle {
        x: seed.unit(index),
        y: 0.10 + seed.unit(index + 1) * 0.45,
        vx: 0.003 + profile.wind_drift_x * 0.003 + seed.signed(index + 2) * 0.001,
        vy: -0.0007 + seed.signed(index + 3) * 0.0003,
        glyph: '·',
        ttl: 4.2,
        age: 0.0,
    })
}

fn category_label(category: WeatherCategory) -> &'static str {
    match category {
        WeatherCategory::Clear => "clear",
        WeatherCategory::Cloudy => "cloudy",
        WeatherCategory::Rain => "rain",
        WeatherCategory::Snow => "snow",
        WeatherCategory::Fog => "fog",
        WeatherCategory::Thunder => "thunder",
        WeatherCategory::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rain_profile() -> WeatherMotionProfile {
        WeatherMotionProfile {
            category: WeatherCategory::Rain,
            intensity: 0.7,
            wind_strength: 0.5,
            wind_drift_x: 0.3,
            gustiness: 0.4,
            visibility_band: crate::ui::animation::VisibilityBand::Muted,
            cloud_density: crate::ui::animation::CloudDensity::Layered,
            is_day: true,
            is_transitioning: false,
        }
    }

    #[test]
    fn particle_engine_spawns_particles_for_rain_profile() {
        let mut engine = ParticleEngine::new(MotionMode::Cinematic, false);
        engine.update(Some(rain_profile()), Duration::from_millis(70), 42);
        assert!(!engine.particles.is_empty());
    }

    #[test]
    fn particle_engine_clears_when_motion_mode_is_off() {
        let mut engine = ParticleEngine::new(MotionMode::Cinematic, false);
        engine.update(Some(rain_profile()), Duration::from_millis(70), 42);
        assert!(!engine.particles.is_empty());
        engine.set_options(MotionMode::Off, false);
        engine.update(Some(rain_profile()), Duration::from_millis(70), 42);
        assert!(engine.particles.is_empty());
    }

    #[test]
    fn reduced_motion_uses_lower_density_than_cinematic() {
        let mut cinematic = ParticleEngine::new(MotionMode::Cinematic, false);
        let mut reduced = ParticleEngine::new(MotionMode::Reduced, false);
        for tick in 0..4 {
            let dt = Duration::from_millis(60);
            cinematic.update(Some(rain_profile()), dt, 10 + tick);
            reduced.update(Some(rain_profile()), dt, 10 + tick);
        }
        assert!(cinematic.particles.len() > reduced.particles.len());
    }

    #[test]
    fn thunder_flash_respects_motion_mode() {
        let profile = WeatherMotionProfile {
            category: WeatherCategory::Thunder,
            ..rain_profile()
        };
        let mut engine = ParticleEngine::new(MotionMode::Reduced, false);
        for tick in 0..20 {
            engine.update(Some(profile), Duration::from_millis(80), 500 + tick);
        }
        assert!(!engine.flash_active());
    }
}
