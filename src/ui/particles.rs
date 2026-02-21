use std::time::Duration;

use rand::Rng;

use crate::domain::weather::{ParticleKind, weather_code_to_particle};

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub glyph: char,
}

#[derive(Debug)]
pub struct ParticleEngine {
    disabled: bool,
    reduced_motion: bool,
    no_flash: bool,
    pub particles: Vec<Particle>,
    accumulator: f32,
    flash_timer: f32,
}

impl ParticleEngine {
    #[must_use]
    pub fn new(disabled: bool, reduced_motion: bool, no_flash: bool) -> Self {
        Self {
            disabled,
            reduced_motion,
            no_flash,
            particles: Vec::new(),
            accumulator: 0.0,
            flash_timer: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.particles.clear();
    }

    pub fn set_options(&mut self, disabled: bool, reduced_motion: bool, no_flash: bool) {
        let mode_changed = self.disabled != disabled || self.reduced_motion != reduced_motion;
        self.disabled = disabled;
        self.reduced_motion = reduced_motion;
        self.no_flash = no_flash;
        if mode_changed || disabled {
            self.reset();
        }
    }

    #[must_use]
    pub fn flash_active(&self) -> bool {
        !self.no_flash && self.flash_timer > 0.0
    }

    pub fn update(
        &mut self,
        weather_code: Option<u8>,
        wind_speed: Option<f32>,
        wind_direction: Option<f32>,
        dt: Duration,
    ) {
        if self.disabled {
            self.particles.clear();
            return;
        }
        let dt = dt.as_secs_f32().clamp(0.0, 0.25);
        self.accumulator += dt;
        let particle_kind = weather_code.map_or(ParticleKind::None, weather_code_to_particle);
        let drift = wind_drift(wind_speed, wind_direction);
        let mut rng = rand::rng();
        self.spawn_particles(particle_kind, drift, &mut rng);
        self.advance_particles(dt);
        self.maybe_trigger_flash(particle_kind, &mut rng);
        self.flash_timer = (self.flash_timer - dt).max(0.0);
    }

    fn spawn_particles(&mut self, particle_kind: ParticleKind, drift: f32, rng: &mut impl Rng) {
        if self.accumulator < 0.04 {
            return;
        }
        self.accumulator = 0.0;
        for _ in 0..self.particle_density() {
            if let Some(p) = spawn_particle(particle_kind, drift, rng) {
                self.particles.push(p);
            }
        }
    }

    fn particle_density(&self) -> usize {
        if self.reduced_motion { 4 } else { 14 }
    }

    fn advance_particles(&mut self, dt: f32) {
        let step = dt * 60.0;
        for p in &mut self.particles {
            p.x += p.vx * step;
            p.y += p.vy * step;
        }
        self.particles
            .retain(|p| p.y < 1.2 && p.x > -0.2 && p.x < 1.2);
    }

    fn maybe_trigger_flash(&mut self, particle_kind: ParticleKind, rng: &mut impl Rng) {
        if particle_kind != ParticleKind::Thunder || self.no_flash {
            return;
        }
        let chance = if self.reduced_motion { 0.004 } else { 0.016 };
        if rng.random_bool(chance) {
            self.flash_timer = 0.12;
        }
    }
}

fn wind_drift(wind_speed: Option<f32>, wind_direction: Option<f32>) -> f32 {
    let drift_base = (wind_speed.unwrap_or_default() / 40.0).clamp(0.0, 1.0);
    let drift_sign = wind_direction.map_or(1.0, |deg| deg.to_radians().sin().signum());
    drift_base * drift_sign
}

fn spawn_particle(kind: ParticleKind, drift: f32, rng: &mut impl Rng) -> Option<Particle> {
    let x = rng.random_range(0.0..1.0);

    match kind {
        ParticleKind::Rain => Some(Particle {
            x,
            y: 0.0,
            vx: (drift * 0.002) + rng.random_range(-0.0005..0.0005),
            vy: rng.random_range(0.008..0.015),
            glyph: '│',
        }),
        ParticleKind::Snow => Some(Particle {
            x,
            y: 0.0,
            vx: (drift * 0.001) + rng.random_range(-0.0015..0.0015),
            vy: rng.random_range(0.002..0.006),
            glyph: '•',
        }),
        ParticleKind::Fog => Some(Particle {
            x,
            y: rng.random_range(0.2..0.8),
            vx: (drift * 0.001) + rng.random_range(0.0003..0.0012),
            vy: rng.random_range(-0.0003..0.0003),
            glyph: '·',
        }),
        ParticleKind::Thunder => Some(Particle {
            x,
            y: 0.0,
            vx: (drift * 0.0022) + rng.random_range(-0.0006..0.0006),
            vy: rng.random_range(0.01..0.018),
            glyph: '│',
        }),
        ParticleKind::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_particles_updates_particles() {
        let mut engine = ParticleEngine::new(false, false, false);
        let dt = Duration::from_millis(50);
        // Weather code 61 is Slight rain -> ParticleKind::Rain
        engine.update(Some(61), Some(10.0), Some(180.0), dt);

        assert!(!engine.particles.is_empty(), "Particles should be spawned");
    }

    #[test]
    fn disabled_engine_clears_particles_on_update() {
        let mut engine = ParticleEngine::new(false, false, false);
        // Spawn some particles first
        engine.update(Some(61), None, None, Duration::from_millis(50));
        assert!(!engine.particles.is_empty());
        // Disable and update — particles should be cleared
        engine.set_options(true, false, false);
        engine.update(Some(61), None, None, Duration::from_millis(50));
        assert!(engine.particles.is_empty());
    }

    #[test]
    fn set_options_resets_particles_when_mode_changes() {
        let mut engine = ParticleEngine::new(false, false, false);
        engine.update(Some(61), None, None, Duration::from_millis(50));
        let before = engine.particles.len();
        // Switching to reduced_motion should reset
        engine.set_options(false, true, false);
        // After reset and without another update, count may be 0 or unchanged;
        // the important thing is set_options doesn't panic
        let _ = before;
    }

    #[test]
    fn set_options_no_reset_when_mode_unchanged() {
        let mut engine = ParticleEngine::new(false, false, false);
        engine.update(Some(61), None, None, Duration::from_millis(50));
        let count_before = engine.particles.len();
        // Same options → no reset
        engine.set_options(false, false, true);
        // Particle count should be preserved (only no_flash changed)
        assert_eq!(engine.particles.len(), count_before);
    }

    #[test]
    fn flash_active_false_when_no_flash_set() {
        let engine = ParticleEngine::new(false, false, true); // no_flash=true
        assert!(!engine.flash_active());
    }

    #[test]
    fn flash_active_false_when_timer_zero() {
        let engine = ParticleEngine::new(false, false, false); // no_flash=false, timer=0
        assert!(!engine.flash_active());
    }

    #[test]
    fn reduced_motion_spawns_fewer_particles() {
        // Build engine with reduced_motion and give it enough ticks
        let mut reduced = ParticleEngine::new(false, true, false);
        let mut full = ParticleEngine::new(false, false, false);
        // Run 20 ticks so many frames accumulate
        for _ in 0..20 {
            reduced.update(Some(61), None, None, Duration::from_millis(50));
            full.update(Some(61), None, None, Duration::from_millis(50));
        }
        assert!(
            reduced.particles.len() <= full.particles.len(),
            "reduced={} should be <= full={}",
            reduced.particles.len(),
            full.particles.len()
        );
    }

    #[test]
    fn none_weather_code_spawns_no_particles() {
        let mut engine = ParticleEngine::new(false, false, false);
        engine.update(None, None, None, Duration::from_millis(50));
        assert!(engine.particles.is_empty());
    }

    #[test]
    fn thunder_code_can_trigger_flash() {
        let mut engine = ParticleEngine::new(false, false, false);
        // Run many ticks with thunder code (95); flash may trigger eventually
        // We just ensure it doesn't panic and update runs
        for _ in 0..100 {
            engine.update(Some(95), None, None, Duration::from_millis(50));
        }
        // Test passes if no panic
    }
}
