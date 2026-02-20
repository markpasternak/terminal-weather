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

impl<'a> Widget for GradientBackground<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.flash {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(' ').set_bg(self.flash_bg);
                    }
                }
            }
            return;
        }

        let bg_top = color_to_rgb(self.top);
        let bg_bot = color_to_rgb(self.bottom);
        for y in area.top()..area.bottom() {
            let t = if area.height <= 1 {
                0.0
            } else {
                (y - area.top()) as f32 / (area.height - 1) as f32
            };
            let c = lerp_color(bg_top, bg_bot, t);
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ').set_bg(c);
                }
            }
        }

        for p in self.particles {
            let px = area.x + ((p.x.clamp(0.0, 1.0)) * area.width as f32) as u16;
            let py = area.y + ((p.y.clamp(0.0, 1.0)) * area.height as f32) as u16;
            if px < area.right()
                && py < area.bottom()
                && let Some(cell) = buf.cell_mut((px, py))
            {
                let bg = cell.bg;
                cell.set_symbol(&p.glyph.to_string())
                    .set_fg(self.particle)
                    .set_bg(bg);
            }
        }
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
