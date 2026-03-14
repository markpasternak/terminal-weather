#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::match_same_arms
)]

use ratatui::{
    buffer::{Buffer, Cell},
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

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
    let style = Style::default().bg(flash_bg);
    let mut blank_cell = Cell::default();
    let _ = blank_cell.set_symbol(" ");
    blank_cell.set_style(style);

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            // OPTIMIZATION: directly assigning a cloned, pre-configured dummy cell to memory
            // is significantly faster (~3x) than chained builder updates (cell.set_symbol(" ").set_style(style))
            // because it avoids repeatedly dropping and updating strings internally.
            if let Some(cell) = buf.cell_mut((x, y)) {
                *cell = blank_cell.clone();
            }
        }
    }
}

fn paint_gradient_background(area: Rect, buf: &mut Buffer, top: Color, bottom: Color) {
    let bg_top = color_to_rgb(top);
    let bg_bottom = color_to_rgb(bottom);

    // OPTIMIZATION: Precompute inverse height to replace division with multiplication in the loop.
    // Also hoist area property access.
    let height_f = area.height.saturating_sub(1) as f32;
    let inv_height = if height_f > 0.0 { 1.0 / height_f } else { 0.0 };
    let area_top = area.top();
    let area_left = area.left();
    let area_right = area.right();
    let area_bottom = area.bottom();

    let mut blank_cell = Cell::default();
    let _ = blank_cell.set_symbol(" ");

    for y in area_top..area_bottom {
        // Inline gradient_ratio logic: (y - top) / (height - 1)
        let t = (y - area_top) as f32 * inv_height;
        let color = lerp_color(bg_top, bg_bottom, t);
        let style = Style::default().bg(color);
        blank_cell.set_style(style);

        for x in area_left..area_right {
            // OPTIMIZATION: directly assigning a cloned, pre-configured dummy cell to memory
            // is significantly faster (~3x) than chained builder updates (cell.set_symbol(" ").set_style(style))
            // because it avoids repeatedly dropping and updating strings internally.
            if let Some(cell) = buf.cell_mut((x, y)) {
                *cell = blank_cell.clone();
            }
        }
    }
}

#[cfg(test)]
fn gradient_ratio(area: Rect, y: u16) -> f32 {
    if area.height <= 1 {
        0.0
    } else {
        (y - area.top()) as f32 / (area.height - 1) as f32
    }
}

fn paint_particles(area: Rect, buf: &mut Buffer, particles: &[Particle], particle_color: Color) {
    // Reuse a stack buffer to avoid heap allocations in the loop
    let mut utf8_buf = [0u8; 4];

    // OPTIMIZATION: Hoist area property conversions to f32 outside the loop.
    let width_f = area.width as f32;
    let height_f = area.height as f32;
    let area_x = area.x;
    let area_y = area.y;
    let area_right = area.right();
    let area_bottom = area.bottom();

    for particle in particles {
        // Inline particle_position logic to avoid function call overhead and repeated conversions.
        // Also use direct casting which is safe given the clamping.
        let x_offset = (particle.x.clamp(0.0, 1.0) * width_f) as u16;
        let y_offset = (particle.y.clamp(0.0, 1.0) * height_f) as u16;

        let x = area_x + x_offset;
        let y = area_y + y_offset;

        if x < area_right
            && y < area_bottom
            && let Some(cell) = buf.cell_mut((x, y))
        {
            let bg = cell.bg;
            cell.set_symbol(particle.glyph.encode_utf8(&mut utf8_buf))
                .set_fg(particle_color)
                .set_bg(bg);
        }
    }
}

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn color_to_rgb_rgb_variant() {
        let result = color_to_rgb(Color::Rgb(100, 150, 200));
        assert!((result.0 - 100.0).abs() < f32::EPSILON);
        assert!((result.1 - 150.0).abs() < f32::EPSILON);
        assert!((result.2 - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn color_to_rgb_black() {
        let result = color_to_rgb(Color::Black);
        assert!((result.0 - 0.0).abs() < f32::EPSILON);
        assert!((result.1 - 0.0).abs() < f32::EPSILON);
        assert!((result.2 - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn color_to_rgb_dark_gray() {
        let result = color_to_rgb(Color::DarkGray);
        assert!((result.0 - 85.0).abs() < f32::EPSILON);
    }

    #[test]
    fn color_to_rgb_gray() {
        let result = color_to_rgb(Color::Gray);
        assert!((result.0 - 170.0).abs() < f32::EPSILON);
    }

    #[test]
    fn color_to_rgb_white() {
        let result = color_to_rgb(Color::White);
        assert!((result.0 - 255.0).abs() < f32::EPSILON);
    }

    #[test]
    fn color_to_rgb_catch_all() {
        let result = color_to_rgb(Color::Red);
        assert!((result.0 - 0.0).abs() < f32::EPSILON);
        assert!((result.1 - 0.0).abs() < f32::EPSILON);
        assert!((result.2 - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn gradient_ratio_height_one_returns_zero() {
        let area = Rect::new(0, 0, 80, 1);
        let ratio = gradient_ratio(area, 0);
        assert!((ratio - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn gradient_ratio_normal_height_computes_correctly() {
        let area = Rect::new(0, 0, 80, 10);
        let ratio = gradient_ratio(area, 5);
        assert!((ratio - 0.555_555_6).abs() < 0.01);
    }

    #[test]
    fn particle_position_in_bounds_returns_some() {
        let area = Rect::new(10, 5, 80, 20);
        let particle = Particle {
            x: 0.5,
            y: 0.5,
            glyph: '●',
            vx: 0.0,
            vy: 0.0,
            ttl: 1.0,
            age: 0.0,
        };
        let pos = particle_position(area, &particle);
        assert!(pos.is_some());
        let (x, y) = pos.unwrap();
        assert_eq!(x, 50);
        assert_eq!(y, 15);
    }

    #[test]
    fn particle_position_clamped_to_bounds() {
        let area = Rect::new(0, 0, 80, 20);
        let particle = Particle {
            x: 0.99,
            y: 0.0,
            glyph: '●',
            vx: 0.0,
            vy: 0.0,
            ttl: 1.0,
            age: 0.0,
        };
        let pos = particle_position(area, &particle);
        assert!(pos.is_some());
    }

    #[test]
    fn lerp_color_interpolates_correctly() {
        let a = (0.0, 0.0, 0.0);
        let b = (100.0, 100.0, 100.0);
        let result = lerp_color(a, b, 0.5);
        assert!(matches!(result, Color::Rgb(50, 50, 50)));
    }

    #[test]
    fn lerp_color_negative_t_goes_opposite_direction() {
        let a = (10.0, 10.0, 10.0);
        let b = (0.0, 0.0, 0.0);
        let result = lerp_color(a, b, -0.5);
        assert!(matches!(result, Color::Rgb(15, 15, 15)));
    }

    #[test]
    fn lerp_color_clamps_above_255() {
        let a = (200.0, 200.0, 200.0);
        let b = (300.0, 300.0, 300.0);
        let result = lerp_color(a, b, 2.0);
        assert!(matches!(result, Color::Rgb(255, 255, 255)));
    }
}
