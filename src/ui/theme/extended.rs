use super::data::ThemeAppearance;
use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct ThemePalette {
    pub appearance: ThemeAppearance,
    pub top: Rgb,
    pub bottom: Rgb,
    pub surface: Rgb,
    pub surface_alt: Rgb,
    pub popup_surface: Rgb,
    pub accent: Rgb,
    pub text_hint: Rgb,
    pub muted_hint: Rgb,
    pub info: Rgb,
    pub success: Rgb,
    pub warning: Rgb,
    pub danger: Rgb,
    pub temp_freezing: Rgb,
    pub temp_cold: Rgb,
    pub temp_mild: Rgb,
    pub temp_warm: Rgb,
    pub temp_hot: Rgb,
    pub landmark_warm: Rgb,
    pub landmark_cool: Rgb,
    pub landmark_neutral: Rgb,
    pub particle: Option<Rgb>,
    pub border: Option<Rgb>,
    pub popup_border: Option<Rgb>,
    pub range_track: Option<Rgb>,
}

#[derive(Debug, Clone, Copy)]
struct PaletteContext {
    is_light: bool,
    backgrounds: [Rgb; 5],
    hero_backgrounds: [Rgb; 3],
}

#[derive(Debug, Clone, Copy)]
struct BaseTokens {
    accent: Rgb,
    text: Rgb,
    muted: Rgb,
    popup_text: Rgb,
    popup_muted: Rgb,
    particle: Rgb,
    border: Rgb,
    popup_border: Rgb,
    range_track: Rgb,
}

#[derive(Debug, Clone, Copy)]
struct SemanticTokens {
    info: Rgb,
    success: Rgb,
    warning: Rgb,
    danger: Rgb,
    temp_freezing: Rgb,
    temp_cold: Rgb,
    temp_mild: Rgb,
    temp_warm: Rgb,
    temp_hot: Rgb,
    landmark_warm: Rgb,
    landmark_cool: Rgb,
    landmark_neutral: Rgb,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum MinimumToken {
    Text,
    Muted,
    Accent,
    PopupText,
    PopupMuted,
    Particle,
    Border,
    PopupBorder,
    Semantic,
    Temperature,
    Landmark,
    RangeTrack,
}

impl MinimumToken {
    const fn index(self) -> usize {
        self as usize
    }
}

const LIGHT_MINIMUMS: [f32; 12] = [4.8, 4.2, 3.8, 4.8, 4.3, 2.4, 2.8, 3.0, 4.4, 3.5, 3.4, 3.3];
const DARK_MINIMUMS: [f32; 12] = [4.5, 4.0, 3.2, 4.6, 4.0, 2.0, 2.6, 2.8, 4.2, 3.3, 3.2, 3.0];

pub(super) fn theme_for_palette(palette: ThemePalette, capability: ColorCapability) -> Theme {
    let ctx = palette_context(palette);
    let base = base_tokens(palette, ctx);
    let semantic = semantic_tokens(palette, ctx, base.accent);
    assemble_theme(palette, ctx, base, semantic, capability)
}

fn palette_context(palette: ThemePalette) -> PaletteContext {
    PaletteContext {
        is_light: palette.appearance.is_light(),
        backgrounds: [
            palette.top,
            palette.bottom,
            palette.surface,
            palette.surface_alt,
            palette.popup_surface,
        ],
        hero_backgrounds: [palette.top, palette.bottom, palette.surface],
    }
}

fn base_tokens(palette: ThemePalette, ctx: PaletteContext) -> BaseTokens {
    BaseTokens {
        accent: ensure_contrast_multi(
            palette.accent,
            &ctx.backgrounds,
            minimum(ctx, MinimumToken::Accent),
        ),
        text: ensure_contrast_multi(
            palette.text_hint,
            &ctx.backgrounds,
            minimum(ctx, MinimumToken::Text),
        ),
        muted: ensure_contrast_multi(
            palette.muted_hint,
            &ctx.backgrounds,
            minimum(ctx, MinimumToken::Muted),
        ),
        popup_text: ensure_contrast(
            palette.text_hint,
            palette.popup_surface,
            minimum(ctx, MinimumToken::PopupText),
        ),
        popup_muted: ensure_contrast(
            palette.muted_hint,
            palette.popup_surface,
            minimum(ctx, MinimumToken::PopupMuted),
        ),
        particle: ensure_contrast_multi(
            particle_seed(palette, ctx),
            &ctx.hero_backgrounds,
            minimum(ctx, MinimumToken::Particle),
        ),
        border: ensure_contrast_multi(
            border_seed(palette, ctx),
            &ctx.backgrounds[..4],
            minimum(ctx, MinimumToken::Border),
        ),
        popup_border: ensure_contrast(
            popup_border_seed(palette, ctx),
            palette.popup_surface,
            minimum(ctx, MinimumToken::PopupBorder),
        ),
        range_track: ensure_contrast_multi(
            palette.range_track.unwrap_or(palette.muted_hint),
            &[palette.surface_alt, palette.popup_surface],
            minimum(ctx, MinimumToken::RangeTrack),
        ),
    }
}

fn semantic_tokens(palette: ThemePalette, ctx: PaletteContext, accent: Rgb) -> SemanticTokens {
    SemanticTokens {
        info: semantic_color(palette.info, &ctx.backgrounds, ctx),
        success: semantic_color(palette.success, &ctx.backgrounds, ctx),
        warning: warning_color(palette.warning, accent, &ctx.backgrounds, ctx),
        danger: semantic_color(palette.danger, &ctx.backgrounds, ctx),
        temp_freezing: temperature_color(palette.temp_freezing, &ctx.backgrounds, ctx),
        temp_cold: temperature_color(palette.temp_cold, &ctx.backgrounds, ctx),
        temp_mild: temperature_color(palette.temp_mild, &ctx.backgrounds, ctx),
        temp_warm: temperature_color(palette.temp_warm, &ctx.backgrounds, ctx),
        temp_hot: temperature_color(palette.temp_hot, &ctx.backgrounds, ctx),
        landmark_warm: landmark_color(palette.landmark_warm, &ctx.hero_backgrounds, ctx),
        landmark_cool: landmark_color(palette.landmark_cool, &ctx.hero_backgrounds, ctx),
        landmark_neutral: landmark_neutral_color(
            palette.landmark_neutral,
            &ctx.hero_backgrounds,
            ctx,
        ),
    }
}

fn assemble_theme(
    palette: ThemePalette,
    _ctx: PaletteContext,
    base: BaseTokens,
    semantic: SemanticTokens,
    capability: ColorCapability,
) -> Theme {
    Theme {
        top: quantize_rgb(palette.top, capability),
        bottom: quantize_rgb(palette.bottom, capability),
        surface: quantize_rgb(palette.surface, capability),
        surface_alt: quantize_rgb(palette.surface_alt, capability),
        popup_surface: quantize_rgb(palette.popup_surface, capability),
        accent: quantize_rgb(base.accent, capability),
        text: quantize_rgb(base.text, capability),
        muted_text: quantize_rgb(base.muted, capability),
        popup_text: quantize_rgb(base.popup_text, capability),
        popup_muted_text: quantize_rgb(base.popup_muted, capability),
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
        range_track: quantize_rgb(base.range_track, capability),
        landmark_warm: quantize_rgb(semantic.landmark_warm, capability),
        landmark_cool: quantize_rgb(semantic.landmark_cool, capability),
        landmark_neutral: quantize_rgb(semantic.landmark_neutral, capability),
    }
}

pub(super) fn quantize_rgb(rgb: Rgb, capability: ColorCapability) -> Color {
    quantize(Color::Rgb(rgb.0, rgb.1, rgb.2), capability)
}

fn minimum(ctx: PaletteContext, token: MinimumToken) -> f32 {
    let values = if ctx.is_light {
        &LIGHT_MINIMUMS
    } else {
        &DARK_MINIMUMS
    };
    values
        .get(token.index())
        .copied()
        .unwrap_or_else(|| unreachable!("missing minimum for token"))
}

fn border_seed(palette: ThemePalette, ctx: PaletteContext) -> Rgb {
    palette.border.unwrap_or_else(|| {
        if ctx.is_light {
            mix_rgb(palette.surface_alt, (88, 104, 124), 0.48)
        } else {
            mix_rgb(palette.surface_alt, palette.accent, 0.28)
        }
    })
}

fn popup_border_seed(palette: ThemePalette, ctx: PaletteContext) -> Rgb {
    palette.popup_border.unwrap_or_else(|| {
        if ctx.is_light {
            mix_rgb(palette.popup_surface, (70, 88, 109), 0.42)
        } else {
            mix_rgb(palette.popup_surface, palette.accent, 0.34)
        }
    })
}

fn particle_seed(palette: ThemePalette, ctx: PaletteContext) -> Rgb {
    palette.particle.unwrap_or_else(|| {
        if ctx.is_light {
            mix_rgb(palette.muted_hint, palette.top, 0.35)
        } else {
            mix_rgb(palette.muted_hint, palette.accent, 0.18)
        }
    })
}

fn adjusted_color(seed: Rgb, backgrounds: &[Rgb], ctx: PaletteContext, token: MinimumToken) -> Rgb {
    ensure_contrast_multi(seed, backgrounds, minimum(ctx, token))
}

fn semantic_color(seed: Rgb, backgrounds: &[Rgb], ctx: PaletteContext) -> Rgb {
    adjusted_color(seed, backgrounds, ctx, MinimumToken::Semantic)
}

fn landmark_neutral_color(seed: Rgb, hero_backgrounds: &[Rgb], ctx: PaletteContext) -> Rgb {
    ensure_contrast_multi(seed, hero_backgrounds, if ctx.is_light { 3.2 } else { 3.0 })
}

fn warning_color(seed: Rgb, accent: Rgb, backgrounds: &[Rgb], ctx: PaletteContext) -> Rgb {
    let minimum = minimum(ctx, MinimumToken::Semantic);
    let initial = ensure_contrast_multi(separate_from_accent(seed, accent), backgrounds, minimum);
    if color_distance(initial, accent) >= 50.0 {
        return initial;
    }

    let fallback = if accent.0 > accent.1 {
        (154, 40, 86)
    } else {
        (224, 140, 70)
    };
    ensure_contrast_multi(fallback, backgrounds, minimum)
}

fn temperature_color(seed: Rgb, backgrounds: &[Rgb], ctx: PaletteContext) -> Rgb {
    adjusted_color(seed, backgrounds, ctx, MinimumToken::Temperature)
}

fn landmark_color(seed: Rgb, hero_backgrounds: &[Rgb], ctx: PaletteContext) -> Rgb {
    adjusted_color(seed, hero_backgrounds, ctx, MinimumToken::Landmark)
}

fn separate_from_accent(warning: Rgb, accent: Rgb) -> Rgb {
    if color_distance(warning, accent) >= 50.0 {
        return warning;
    }

    let warm_accent = accent.0 > 180 && accent.1 > 140 && accent.2 < 160;
    if warm_accent {
        (214, 92, 92)
    } else if luma(accent.0, accent.1, accent.2) > 150.0 {
        mix_rgb(warning, (176, 56, 56), 0.55)
    } else {
        mix_rgb(warning, (255, 196, 92), 0.50)
    }
}

fn color_distance(a: Rgb, b: Rgb) -> f32 {
    ((a.0 as f32 - b.0 as f32).powi(2)
        + (a.1 as f32 - b.1 as f32).powi(2)
        + (a.2 as f32 - b.2 as f32).powi(2))
    .sqrt()
}
