use super::*;

#[derive(Debug, Clone, Copy)]
struct ExtendedScaffold {
    top: Rgb,
    bottom: Rgb,
    surface: Rgb,
    surface_alt: Rgb,
    popup_surface: Rgb,
    dark_text: bool,
    all_backgrounds: [Rgb; 5],
    hero_backgrounds: [Rgb; 3],
}

#[derive(Debug, Clone, Copy)]
struct ExtendedBaseColors {
    accent: Rgb,
    text: Rgb,
    muted: Rgb,
    popup_text: Rgb,
    popup_muted_text: Rgb,
    particle: Rgb,
    border: Rgb,
    popup_border: Rgb,
}

#[derive(Debug, Clone, Copy)]
struct ExtendedSemanticColors {
    info: Rgb,
    success: Rgb,
    warning: Rgb,
    danger: Rgb,
    range_track: Rgb,
    landmark_warm: Rgb,
    landmark_cool: Rgb,
    landmark_neutral: Rgb,
    temp_freezing: Rgb,
    temp_cold: Rgb,
    temp_mild: Rgb,
    temp_warm: Rgb,
    temp_hot: Rgb,
}

pub(super) fn theme_for_extended(
    top: Rgb,
    bottom: Rgb,
    accent_seed: Rgb,
    capability: ColorCapability,
) -> Theme {
    let scaffold = extended_scaffold(top, bottom, accent_seed);
    let base = extended_base_colors(&scaffold, accent_seed);
    let semantic = extended_semantic_colors(&scaffold, accent_seed, &base);
    assemble_extended_theme(scaffold, base, semantic, capability)
}

fn extended_scaffold(top: Rgb, bottom: Rgb, accent_seed: Rgb) -> ExtendedScaffold {
    let avg_luma = (luma(top.0, top.1, top.2) + luma(bottom.0, bottom.1, bottom.2)) / 2.0;
    let dark_text = avg_luma >= 170.0;
    let (top, bottom) = adjust_light_gradients(top, bottom, dark_text);
    let base_surface = mix_rgb(top, bottom, 0.80);
    let base_surface_alt = mix_rgb(top, bottom, 0.60);
    let (tint_factor, tint_factor_alt) = surface_tint_factors(dark_text, avg_luma);
    let surface = mix_rgb(base_surface, accent_seed, tint_factor);
    let surface_alt = mix_rgb(base_surface_alt, accent_seed, tint_factor_alt);
    let popup_surface = choose_rgb(
        dark_text,
        mix_rgb(surface_alt, accent_seed, 0.20),
        mix_rgb(surface_alt, (236, 243, 251), 0.18),
    );
    ExtendedScaffold {
        top,
        bottom,
        surface,
        surface_alt,
        popup_surface,
        dark_text,
        all_backgrounds: [surface, surface_alt, popup_surface, top, bottom],
        hero_backgrounds: [top, bottom, surface],
    }
}

fn extended_base_colors(scaffold: &ExtendedScaffold, accent_seed: Rgb) -> ExtendedBaseColors {
    let text_seed = choose_rgb(scaffold.dark_text, (12, 16, 24), (240, 245, 250));
    let muted_seed = choose_rgb(scaffold.dark_text, (55, 68, 85), (183, 198, 214));
    let text = ensure_contrast_multi(
        text_seed,
        &scaffold.all_backgrounds,
        if scaffold.dark_text { 4.9 } else { 4.7 },
    );
    let muted = ensure_contrast_multi(
        muted_seed,
        &scaffold.all_backgrounds,
        if scaffold.dark_text { 4.5 } else { 4.2 },
    );
    let accent = ensure_contrast_multi(
        accent_seed,
        &scaffold.all_backgrounds,
        if scaffold.dark_text { 4.5 } else { 4.0 },
    );
    let border_seed = choose_rgb(
        scaffold.dark_text,
        mix_rgb(scaffold.surface, (18, 26, 38), 0.74),
        mix_rgb(scaffold.surface, accent, 0.54),
    );
    let popup_border_seed = choose_rgb(
        scaffold.dark_text,
        mix_rgb(scaffold.popup_surface, (5, 11, 18), 0.82),
        mix_rgb(scaffold.popup_surface, accent, 0.70),
    );
    ExtendedBaseColors {
        accent,
        text,
        muted,
        popup_text: ensure_contrast(text_seed, scaffold.popup_surface, 4.7),
        popup_muted_text: ensure_contrast(muted_seed, scaffold.popup_surface, 4.5),
        particle: choose_rgb(scaffold.dark_text, (92, 108, 124), (202, 218, 235)),
        border: ensure_contrast_multi(
            border_seed,
            &[
                scaffold.surface,
                scaffold.surface_alt,
                scaffold.top,
                scaffold.bottom,
            ],
            3.0,
        ),
        popup_border: ensure_contrast(popup_border_seed, scaffold.popup_surface, 3.2),
    }
}

fn extended_semantic_colors(
    scaffold: &ExtendedScaffold,
    accent_seed: Rgb,
    base: &ExtendedBaseColors,
) -> ExtendedSemanticColors {
    let (info, success, warning, danger) = extended_status_colors(scaffold, accent_seed);
    let (landmark_warm, landmark_cool, landmark_neutral) = extended_landmark_colors(scaffold, base);
    let (temp_freezing, temp_cold, temp_mild, temp_warm, temp_hot) = extended_temp_colors(scaffold);
    ExtendedSemanticColors {
        info,
        success,
        warning,
        danger,
        range_track: ensure_contrast(
            base.muted,
            scaffold.surface_alt,
            if scaffold.dark_text { 4.0 } else { 3.2 },
        ),
        landmark_warm,
        landmark_cool,
        landmark_neutral,
        temp_freezing,
        temp_cold,
        temp_mild,
        temp_warm,
        temp_hot,
    }
}

fn extended_status_colors(scaffold: &ExtendedScaffold, accent_seed: Rgb) -> (Rgb, Rgb, Rgb, Rgb) {
    let info = ensure_contrast_multi(
        choose_rgb(scaffold.dark_text, (3, 105, 161), (125, 211, 252)),
        &scaffold.all_backgrounds,
        4.5,
    );
    let success = ensure_contrast_multi(
        choose_rgb(scaffold.dark_text, (21, 128, 61), (74, 222, 128)),
        &scaffold.all_backgrounds,
        4.5,
    );
    let warning = ensure_contrast_multi(
        warning_seed_for(accent_seed, scaffold.dark_text),
        &scaffold.all_backgrounds,
        4.5,
    );
    let danger = ensure_contrast_multi(
        choose_rgb(scaffold.dark_text, (185, 28, 28), (248, 113, 113)),
        &scaffold.all_backgrounds,
        4.5,
    );
    (info, success, warning, danger)
}

fn extended_landmark_colors(
    scaffold: &ExtendedScaffold,
    base: &ExtendedBaseColors,
) -> (Rgb, Rgb, Rgb) {
    (
        ensure_contrast_multi(
            (253, 230, 138),
            &scaffold.hero_backgrounds,
            if scaffold.dark_text { 4.5 } else { 3.5 },
        ),
        ensure_contrast_multi(
            (147, 197, 253),
            &scaffold.hero_backgrounds,
            if scaffold.dark_text { 4.5 } else { 3.5 },
        ),
        ensure_contrast_multi(
            base.muted,
            &scaffold.hero_backgrounds,
            if scaffold.dark_text { 4.2 } else { 3.2 },
        ),
    )
}

fn extended_temp_colors(scaffold: &ExtendedScaffold) -> (Rgb, Rgb, Rgb, Rgb, Rgb) {
    let threshold = if scaffold.dark_text { 4.5 } else { 3.8 };
    (
        ensure_contrast((147, 197, 253), scaffold.surface_alt, threshold),
        ensure_contrast((56, 189, 248), scaffold.surface_alt, threshold),
        ensure_contrast((110, 231, 183), scaffold.surface_alt, threshold),
        ensure_contrast((251, 191, 36), scaffold.surface_alt, threshold),
        ensure_contrast((248, 113, 113), scaffold.surface_alt, threshold),
    )
}

fn assemble_extended_theme(
    scaffold: ExtendedScaffold,
    base: ExtendedBaseColors,
    semantic: ExtendedSemanticColors,
    capability: ColorCapability,
) -> Theme {
    Theme {
        top: quantize_rgb(scaffold.top, capability),
        bottom: quantize_rgb(scaffold.bottom, capability),
        surface: quantize_rgb(scaffold.surface, capability),
        surface_alt: quantize_rgb(scaffold.surface_alt, capability),
        popup_surface: quantize_rgb(scaffold.popup_surface, capability),
        accent: quantize_rgb(base.accent, capability),
        text: quantize_rgb(base.text, capability),
        muted_text: quantize_rgb(base.muted, capability),
        popup_text: quantize_rgb(base.popup_text, capability),
        popup_muted_text: quantize_rgb(base.popup_muted_text, capability),
        particle: quantize_rgb(base.particle, capability),
        border: quantize_rgb(base.border, capability),
        popup_border: quantize_rgb(base.popup_border, capability),
        info: quantize_rgb(semantic.info, capability),
        success: quantize_rgb(semantic.success, capability),
        warning: quantize_rgb(semantic.warning, capability),
        danger: quantize_rgb(semantic.danger, capability),
        temp_freezing: quantize_rgb(semantic.temp_freezing, capability),
        temp_cold: quantize_rgb(semantic.temp_cold, capability),
        temp_mild: quantize_rgb(semantic.temp_mild, capability),
        temp_warm: quantize_rgb(semantic.temp_warm, capability),
        temp_hot: quantize_rgb(semantic.temp_hot, capability),
        range_track: quantize_rgb(semantic.range_track, capability),
        landmark_warm: quantize_rgb(semantic.landmark_warm, capability),
        landmark_cool: quantize_rgb(semantic.landmark_cool, capability),
        landmark_neutral: quantize_rgb(semantic.landmark_neutral, capability),
    }
}

pub(super) fn quantize_rgb(rgb: Rgb, capability: ColorCapability) -> Color {
    quantize(Color::Rgb(rgb.0, rgb.1, rgb.2), capability)
}

fn adjust_light_gradients(
    top: (u8, u8, u8),
    bottom: (u8, u8, u8),
    dark_text: bool,
) -> ((u8, u8, u8), (u8, u8, u8)) {
    if dark_text {
        // Keep light themes readable by pulling gradients away from near-white.
        (
            mix_rgb(top, (198, 210, 226), 0.42),
            mix_rgb(bottom, (176, 193, 214), 0.40),
        )
    } else {
        (top, bottom)
    }
}

fn surface_tint_factors(dark_text: bool, avg_luma: f32) -> (f32, f32) {
    // Reduce accent tint on very dark backgrounds to avoid hue-on-hue illegibility.
    if dark_text || avg_luma < 40.0 {
        (0.08, 0.12)
    } else {
        (0.16, 0.24)
    }
}

fn choose_rgb(condition: bool, when_true: (u8, u8, u8), when_false: (u8, u8, u8)) -> (u8, u8, u8) {
    if condition { when_true } else { when_false }
}

fn warning_seed_for(accent_seed: (u8, u8, u8), dark_text: bool) -> (u8, u8, u8) {
    // Shift warning toward orange-red when accent is already warm/amber to avoid collision.
    let warm_accent = accent_seed.0 > 180 && accent_seed.1 > 140 && accent_seed.2 < 140;
    if warm_accent {
        // Use pink-red so it stays distinct from the amber/gold accent after contrast push.
        choose_rgb(dark_text, (180, 40, 60), (255, 110, 130))
    } else {
        choose_rgb(dark_text, (161, 98, 7), (251, 191, 36))
    }
}
