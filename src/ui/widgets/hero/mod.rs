pub mod background;
pub mod landmark_panel;
mod shared;
#[cfg(test)]
mod test_support;
pub mod weather;
pub mod weather_expanded;

use crate::{
    app::state::AppState,
    cli::Cli,
    domain::weather::{WeatherCategory, weather_code_to_category},
    ui::{
        animation::{CloudDensity, UiMotionContext, VisibilityBand},
        motion_context,
        theme::{detect_color_capability, theme_for},
    },
};
use background::GradientBackground;
use landmark_panel::render_landmark;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
};
use weather::render_weather_info;

fn inset_rect(r: Rect, dw: u16, dh: u16) -> Rect {
    let w = r.width.saturating_sub(dw * 2);
    let h = r.height.saturating_sub(dh * 2);
    Rect {
        x: r.x.saturating_add(dw),
        y: r.y.saturating_add(dh),
        width: w,
        height: h,
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    let (is_day, code, theme) = hero_context(state);
    render_hero_background(frame, area, state, theme);

    let title_prefix = if state.panel_focus == crate::app::state::PanelFocus::Hero {
        "▶ "
    } else {
        ""
    };
    let title = Line::from(vec![
        Span::styled(
            format!("{title_prefix}Current · "),
            Style::default().fg(theme.text),
        ),
        Span::styled(
            "L",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" cities · ", Style::default().fg(theme.muted_text)),
        Span::styled(
            "S",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" settings · ", Style::default().fg(theme.muted_text)),
        Span::styled(
            "?",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" help", Style::default().fg(theme.muted_text)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(theme.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = split_hero_columns(inner);
    let left_area = left_weather_area(columns[0]);
    render_weather_info(frame, left_area, state, theme, code);

    if columns.len() > 1 {
        render_right_landmark(frame, columns[1], state, is_day, theme);
    }
}

fn hero_context(state: &AppState) -> (bool, u8, crate::ui::theme::Theme) {
    let (category, is_day, code) = state.weather.as_ref().map_or(
        // Loading/no-data should default to a dark palette to avoid bright blank panels.
        (WeatherCategory::Unknown, false, 0),
        |w| {
            (
                weather_code_to_category(w.current.weather_code),
                w.current.is_day,
                w.current.weather_code,
            )
        },
    );
    let capability = detect_color_capability(state.color_mode);
    let theme = theme_for(category, is_day, capability, state.settings.theme);
    (is_day, code, theme)
}

fn render_hero_background(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: crate::ui::theme::Theme,
) {
    let motion = motion_context(state, "hero-background");
    let (top, bottom) = animated_background_colors(theme, motion);
    let bg = GradientBackground {
        top,
        bottom,
        text: theme.text,
        particle: theme.particle,
        particles: &state.particles.particles,
        flash: state.particles.flash_active(),
        flash_bg: theme.accent,
        flash_fg: theme.text,
    };
    frame.render_widget(bg, area);
}

fn animated_background_colors(
    theme: crate::ui::theme::Theme,
    motion: UiMotionContext,
) -> (ratatui::style::Color, ratatui::style::Color) {
    let Some(profile) = motion.weather_profile else {
        return (theme.top, theme.bottom);
    };

    let pulse = motion.lane("gradient").pulse(
        motion.elapsed_seconds,
        0.18 + profile.wind_strength * 0.20,
        0,
    );
    let settle = motion.transition_mix();
    let transition_boost = (1.0 - settle) * 0.12;

    match profile.category {
        WeatherCategory::Clear => clear_background(theme, pulse, transition_boost),
        WeatherCategory::Cloudy => cloudy_background(theme, profile),
        WeatherCategory::Rain => rain_background(theme, profile, pulse),
        WeatherCategory::Snow => snow_background(theme, profile, pulse),
        WeatherCategory::Fog => fog_background(theme, profile.visibility_band),
        WeatherCategory::Thunder => thunder_background(theme, profile, pulse),
        WeatherCategory::Unknown => (theme.top, theme.bottom),
    }
}

fn clear_background(
    theme: crate::ui::theme::Theme,
    pulse: f32,
    transition_boost: f32,
) -> (ratatui::style::Color, ratatui::style::Color) {
    (
        blend_color(
            theme.top,
            theme.accent,
            0.05 + pulse * 0.06 + transition_boost,
        ),
        blend_color(theme.bottom, theme.info, 0.04 + pulse * 0.04),
    )
}

fn cloudy_background(
    theme: crate::ui::theme::Theme,
    profile: crate::ui::animation::WeatherMotionProfile,
) -> (ratatui::style::Color, ratatui::style::Color) {
    (
        blend_color(
            theme.top,
            theme.border,
            0.05 + cloud_mix(profile.cloud_density) * 0.10,
        ),
        blend_color(
            theme.bottom,
            theme.surface_alt,
            0.06 + profile.wind_strength * 0.08,
        ),
    )
}

fn rain_background(
    theme: crate::ui::theme::Theme,
    profile: crate::ui::animation::WeatherMotionProfile,
    pulse: f32,
) -> (ratatui::style::Color, ratatui::style::Color) {
    (
        blend_color(theme.top, theme.surface, 0.10 + pulse * 0.08),
        blend_color(theme.bottom, theme.info, 0.05 + profile.intensity * 0.08),
    )
}

fn snow_background(
    theme: crate::ui::theme::Theme,
    profile: crate::ui::animation::WeatherMotionProfile,
    pulse: f32,
) -> (ratatui::style::Color, ratatui::style::Color) {
    (
        blend_color(theme.top, theme.info, 0.06 + pulse * 0.05),
        blend_color(theme.bottom, theme.text, 0.04 + profile.intensity * 0.05),
    )
}

fn fog_background(
    theme: crate::ui::theme::Theme,
    visibility_band: VisibilityBand,
) -> (ratatui::style::Color, ratatui::style::Color) {
    let fog_mix = match visibility_band {
        VisibilityBand::Open => 0.12,
        VisibilityBand::Muted => 0.20,
        VisibilityBand::Obscured => 0.28,
    };
    let midpoint = blend_color(theme.top, theme.bottom, 0.50);
    (
        blend_color(theme.top, midpoint, fog_mix),
        blend_color(theme.bottom, midpoint, fog_mix + 0.06),
    )
}

fn thunder_background(
    theme: crate::ui::theme::Theme,
    profile: crate::ui::animation::WeatherMotionProfile,
    pulse: f32,
) -> (ratatui::style::Color, ratatui::style::Color) {
    (
        blend_color(theme.top, theme.surface_alt, 0.16 + pulse * 0.12),
        blend_color(theme.bottom, theme.accent, 0.03 + profile.intensity * 0.07),
    )
}

fn cloud_mix(density: CloudDensity) -> f32 {
    match density {
        CloudDensity::Sparse => 0.25,
        CloudDensity::Layered => 0.55,
        CloudDensity::Dense => 0.85,
    }
}

fn blend_color(
    a: ratatui::style::Color,
    b: ratatui::style::Color,
    mix: f32,
) -> ratatui::style::Color {
    let mix = mix.clamp(0.0, 1.0);
    let (ar, ag, ab) = to_rgb(a);
    let (br, bg, bb) = to_rgb(b);
    ratatui::style::Color::Rgb(
        lerp_u8(ar, br, mix),
        lerp_u8(ag, bg, mix),
        lerp_u8(ab, bb, mix),
    )
}

#[allow(clippy::cast_sign_loss)]
fn lerp_u8(a: u8, b: u8, mix: f32) -> u8 {
    (f32::from(a) + (f32::from(b) - f32::from(a)) * mix)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn to_rgb(color: ratatui::style::Color) -> (u8, u8, u8) {
    use ratatui::style::Color;

    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::DarkGray | Color::Gray | Color::White => neutral_color_rgb(color),
        Color::Red | Color::Green | Color::Yellow | Color::Blue | Color::Magenta | Color::Cyan => {
            primary_color_rgb(color)
        }
        Color::LightRed
        | Color::LightGreen
        | Color::LightYellow
        | Color::LightBlue
        | Color::LightMagenta
        | Color::LightCyan => light_color_rgb(color),
        _ => (0, 0, 0),
    }
}

fn neutral_color_rgb(color: ratatui::style::Color) -> (u8, u8, u8) {
    use ratatui::style::Color;

    match color {
        Color::DarkGray => (85, 85, 85),
        Color::Gray => (170, 170, 170),
        Color::White => (255, 255, 255),
        _ => (0, 0, 0),
    }
}

fn primary_color_rgb(color: ratatui::style::Color) -> (u8, u8, u8) {
    use ratatui::style::Color;

    match color {
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Yellow => (255, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Magenta => (255, 0, 255),
        Color::Cyan => (0, 255, 255),
        _ => (0, 0, 0),
    }
}

fn light_color_rgb(color: ratatui::style::Color) -> (u8, u8, u8) {
    use ratatui::style::Color;

    match color {
        Color::LightRed => (255, 128, 128),
        Color::LightGreen => (128, 255, 128),
        Color::LightYellow => (255, 255, 128),
        Color::LightBlue => (128, 128, 255),
        Color::LightMagenta => (255, 128, 255),
        Color::LightCyan => (128, 255, 255),
        _ => (0, 0, 0),
    }
}

fn split_hero_columns(inner: Rect) -> Vec<Rect> {
    if inner.width < 58 || inner.height < 8 {
        return vec![inner];
    }

    let (left_pct, right_pct) = if inner.width >= 170 {
        (38, 62)
    } else if inner.width >= 140 {
        (40, 60)
    } else if inner.width >= 120 {
        (42, 58)
    } else if inner.width >= 96 {
        (48, 52)
    } else {
        (58, 42)
    };
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_pct),
            Constraint::Percentage(right_pct),
        ])
        .split(inner)
        .to_vec()
}

fn left_weather_area(column: Rect) -> Rect {
    if column.width >= 56 {
        inset_rect(column, 1, 0)
    } else {
        column
    }
}

fn render_right_landmark(
    frame: &mut Frame,
    right_column: Rect,
    state: &AppState,
    is_day: bool,
    theme: crate::ui::theme::Theme,
) {
    let separator = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(theme.border));
    let right_inner = separator.inner(right_column);
    frame.render_widget(separator, right_column);

    let right_content = if right_inner.width >= 78 && right_inner.height >= 14 {
        inset_rect(right_inner, 3, 1)
    } else if right_inner.width >= 60 && right_inner.height >= 10 {
        inset_rect(right_inner, 2, 1)
    } else if right_inner.width >= 48 {
        inset_rect(right_inner, 2, 0)
    } else {
        right_inner
    };
    render_landmark(frame, right_content, state, is_day, theme);
}
