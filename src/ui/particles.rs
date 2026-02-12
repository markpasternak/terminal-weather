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

        let particle_kind = weather_code
            .map(weather_code_to_particle)
            .unwrap_or(ParticleKind::None);

        let drift_base = (wind_speed.unwrap_or_default() / 40.0).clamp(0.0, 1.0);
        let drift_sign = wind_direction
            .map(|deg| deg.to_radians().sin().signum())
            .unwrap_or(1.0);
        let drift = drift_base * drift_sign;

        let density = if self.reduced_motion { 4 } else { 14 };

        if self.accumulator >= 0.04 {
            self.accumulator = 0.0;
            for _ in 0..density {
                if let Some(p) = spawn_particle(particle_kind, drift) {
                    self.particles.push(p);
                }
            }
        }

        let step = dt * 60.0;
        for p in &mut self.particles {
            p.x += p.vx * step;
            p.y += p.vy * step;
        }
        self.particles
            .retain(|p| p.y < 1.2 && p.x > -0.2 && p.x < 1.2);

        if particle_kind == ParticleKind::Thunder && !self.no_flash {
            let mut rng = rand::rng();
            if rng.random_bool(if self.reduced_motion { 0.004 } else { 0.016 }) {
                self.flash_timer = 0.12;
            }
        }
        self.flash_timer = (self.flash_timer - dt).max(0.0);
    }
}

fn spawn_particle(kind: ParticleKind, drift: f32) -> Option<Particle> {
    let mut rng = rand::rng();
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
