pub(super) fn luma(r: u8, g: u8, b: u8) -> f32 {
    (0.2126 * f32::from(r)) + (0.7152 * f32::from(g)) + (0.0722 * f32::from(b))
}

pub(super) fn mix_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| -> u8 {
        (f32::from(x) + (f32::from(y) - f32::from(x)) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

pub(super) fn ensure_contrast(fg: (u8, u8, u8), bg: (u8, u8, u8), min_ratio: f32) -> (u8, u8, u8) {
    ensure_contrast_multi(fg, &[bg], min_ratio)
}

pub(super) fn ensure_contrast_multi(
    fg: (u8, u8, u8),
    backgrounds: &[(u8, u8, u8)],
    min_ratio: f32,
) -> (u8, u8, u8) {
    if backgrounds.is_empty() {
        return fg;
    }
    if min_contrast_ratio(fg, backgrounds) >= min_ratio {
        return fg;
    }

    let black = (0, 0, 0);
    let white = (255, 255, 255);
    let black_score = min_contrast_ratio(black, backgrounds);
    let white_score = min_contrast_ratio(white, backgrounds);
    let target = if white_score >= black_score {
        white
    } else {
        black
    };

    let mut best = fg;
    let mut best_ratio = min_contrast_ratio(fg, backgrounds);
    for step in 1..=24 {
        let t = step as f32 / 24.0;
        let candidate = mix_rgb(fg, target, t);
        let ratio = min_contrast_ratio(candidate, backgrounds);
        if ratio > best_ratio {
            best = candidate;
            best_ratio = ratio;
        }
        if ratio >= min_ratio {
            return candidate;
        }
    }
    best
}

pub(super) fn min_contrast_ratio(color: (u8, u8, u8), backgrounds: &[(u8, u8, u8)]) -> f32 {
    backgrounds
        .iter()
        .map(|bg| contrast_ratio(color, *bg))
        .fold(f32::INFINITY, f32::min)
}

pub(super) fn contrast_ratio(a: (u8, u8, u8), b: (u8, u8, u8)) -> f32 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

pub(super) fn relative_luminance(rgb: (u8, u8, u8)) -> f32 {
    let r = srgb_to_linear(rgb.0);
    let g = srgb_to_linear(rgb.1);
    let b = srgb_to_linear(rgb.2);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn srgb_to_linear(v: u8) -> f32 {
    let s = f32::from(v) / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}
