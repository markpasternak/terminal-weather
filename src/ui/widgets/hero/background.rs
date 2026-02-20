#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::match_same_arms
)]

use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use crate::ui::particles::Particle;

pub struct GradientBackground<'a> {
    pub top: Color,
    pub bottom: Color,
    pub text: Color,
    pub particle: Color,
    pub particles: &'a [Particle],
    pub flash: bool,
    pub flash_bg: Color,
    pub flash_fg: Color,
}

impl Widget for GradientBackground<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.flash {
            paint_flash_background(area, buf, self.flash_bg);
            return;
        }

        paint_gradient_background(area, buf, self.top, self.bottom);
        paint_particles(area, buf, self.particles, self.particle);
    }
}

fn paint_flash_background(area: Rect, buf: &mut Buffer, flash_bg: Color) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ').set_bg(flash_bg);
            }
        }
    }
}

fn paint_gradient_background(area: Rect, buf: &mut Buffer, top: Color, bottom: Color) {
    let bg_top = color_to_rgb(top);
    let bg_bottom = color_to_rgb(bottom);
    for y in area.top()..area.bottom() {
        let t = gradient_ratio(area, y);
        let color = lerp_color(bg_top, bg_bottom, t);
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ').set_bg(color);
            }
        }
    }
}

fn gradient_ratio(area: Rect, y: u16) -> f32 {
    if area.height <= 1 {
        0.0
    } else {
        (y - area.top()) as f32 / (area.height - 1) as f32
    }
}

fn paint_particles(area: Rect, buf: &mut Buffer, particles: &[Particle], particle_color: Color) {
    for particle in particles {
        if let Some((x, y)) = particle_position(area, particle)
            && let Some(cell) = buf.cell_mut((x, y))
        {
            let bg = cell.bg;
            cell.set_symbol(&particle.glyph.to_string())
                .set_fg(particle_color)
                .set_bg(bg);
        }
    }
}

fn particle_position(area: Rect, particle: &Particle) -> Option<(u16, u16)> {
    let x = area.x + ((particle.x.clamp(0.0, 1.0)) * area.width as f32) as u16;
    let y = area.y + ((particle.y.clamp(0.0, 1.0)) * area.height as f32) as u16;
    if x < area.right() && y < area.bottom() {
        Some((x, y))
    } else {
        None
    }
}

fn color_to_rgb(c: Color) -> (f32, f32, f32) {
    match c {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        Color::Black => (0., 0., 0.),
        Color::DarkGray => (85., 85., 85.),
        Color::Gray => (170., 170., 170.),
        Color::White => (255., 255., 255.),
        _ => (0., 0., 0.),
    }
}

fn lerp_color(a: (f32, f32, f32), b: (f32, f32, f32), t: f32) -> Color {
    let r = (a.0 + (b.0 - a.0) * t).clamp(0.0, 255.0) as u8;
    let g = (a.1 + (b.1 - a.1) * t).clamp(0.0, 255.0) as u8;
    let b_val = (a.2 + (b.2 - a.2) * t).clamp(0.0, 255.0) as u8;
    Color::Rgb(r, g, b_val)
}
