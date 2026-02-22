use ratatui::style::Color;

use super::ColorCapability;

pub fn quantize(color: Color, capability: ColorCapability) -> Color {
    match (capability, color) {
        (ColorCapability::TrueColor, c) => c,
        (ColorCapability::Xterm256, Color::Rgb(r, g, b)) => {
            let to_cube = |v: u8| -> u8 { ((f32::from(v) / 255.0) * 5.0).round() as u8 };
            let ri = to_cube(r);
            let gi = to_cube(g);
            let bi = to_cube(b);
            let index = 16 + 36 * ri + 6 * gi + bi;
            Color::Indexed(index)
        }
        (ColorCapability::Basic16, Color::Rgb(r, g, b)) => basic16_from_rgb(r, g, b),
        (_, c) => c,
    }
}

fn basic16_from_rgb(r: u8, g: u8, b: u8) -> Color {
    let rf = f32::from(r) / 255.0;
    let gf = f32::from(g) / 255.0;
    let bf = f32::from(b) / 255.0;

    let max = rf.max(gf.max(bf));
    let min = rf.min(gf.min(bf));
    let delta = max - min;
    let light = (max + min) / 2.0;

    if delta < 0.08 {
        return achromatic_basic16(light);
    }

    let hue = hue_from_rgb_components(rf, gf, bf, max, delta);
    hue_to_basic16(hue, light >= 0.55)
}

fn achromatic_basic16(light: f32) -> Color {
    if light < 0.20 {
        return Color::Black;
    }
    if light < 0.40 {
        return Color::DarkGray;
    }
    if light < 0.72 {
        return Color::Gray;
    }
    Color::White
}

fn hue_from_rgb_components(rf: f32, gf: f32, bf: f32, max: f32, delta: f32) -> f32 {
    if (max - rf).abs() < f32::EPSILON {
        return 60.0 * ((gf - bf) / delta).rem_euclid(6.0);
    }
    if (max - gf).abs() < f32::EPSILON {
        return 60.0 * (((bf - rf) / delta) + 2.0);
    }
    60.0 * (((rf - gf) / delta) + 4.0)
}

fn hue_to_basic16(hue: f32, bright: bool) -> Color {
    let band = if !(30.0..330.0).contains(&hue) {
        0
    } else if hue < 90.0 {
        1
    } else if hue < 150.0 {
        2
    } else if hue < 210.0 {
        3
    } else if hue < 270.0 {
        4
    } else {
        5
    };
    hue_band_color(band, bright)
}

fn hue_band_color(band: usize, bright: bool) -> Color {
    const DIM: [Color; 6] = [
        Color::Red,
        Color::Yellow,
        Color::Green,
        Color::Cyan,
        Color::Blue,
        Color::Magenta,
    ];
    const BRIGHT: [Color; 6] = [
        Color::LightRed,
        Color::LightYellow,
        Color::LightGreen,
        Color::LightCyan,
        Color::LightBlue,
        Color::LightMagenta,
    ];
    if bright { BRIGHT[band] } else { DIM[band] }
}
