use ratatui::style::Color;

use crate::{cli::ThemeArg, domain::weather::WeatherCategory};

use super::{Basic16Palette, ThemeSeed};

pub(super) const AUTO_THEME_SEEDS: &[((WeatherCategory, bool), ThemeSeed)] = &[
    (
        (WeatherCategory::Clear, true),
        ((13, 53, 102), (30, 102, 158), (255, 215, 117)),
    ),
    (
        (WeatherCategory::Clear, false),
        ((9, 18, 44), (21, 43, 79), (173, 216, 255)),
    ),
    (
        (WeatherCategory::Cloudy, true),
        ((25, 36, 51), (48, 63, 84), (210, 223, 235)),
    ),
    (
        (WeatherCategory::Cloudy, false),
        ((20, 26, 40), (34, 42, 62), (194, 207, 224)),
    ),
    (
        (WeatherCategory::Rain, true),
        ((17, 47, 88), (32, 73, 126), (153, 214, 255)),
    ),
    (
        (WeatherCategory::Rain, false),
        ((12, 25, 52), (25, 44, 78), (143, 196, 255)),
    ),
    (
        (WeatherCategory::Snow, true),
        ((27, 51, 77), (43, 74, 106), (237, 247, 255)),
    ),
    (
        (WeatherCategory::Snow, false),
        ((19, 35, 55), (34, 55, 80), (226, 241, 255)),
    ),
    (
        (WeatherCategory::Fog, true),
        ((30, 34, 40), (50, 55, 62), (216, 220, 224)),
    ),
    (
        (WeatherCategory::Fog, false),
        ((21, 24, 30), (33, 37, 43), (201, 207, 211)),
    ),
    (
        (WeatherCategory::Thunder, true),
        ((28, 25, 66), (42, 40, 97), (255, 223, 112)),
    ),
    (
        (WeatherCategory::Thunder, false),
        ((18, 15, 44), (28, 24, 63), (255, 208, 95)),
    ),
    (
        (WeatherCategory::Unknown, true),
        ((28, 36, 51), (42, 53, 73), (205, 219, 234)),
    ),
    (
        (WeatherCategory::Unknown, false),
        ((19, 24, 35), (31, 39, 53), (195, 205, 215)),
    ),
];

pub(super) const PRESET_THEME_SEEDS: &[(ThemeArg, ThemeSeed)] = &[
    (
        ThemeArg::Aurora,
        ((9, 31, 65), (16, 70, 105), (102, 232, 242)),
    ),
    (
        ThemeArg::MidnightCyan,
        ((10, 14, 42), (28, 22, 84), (122, 230, 255)),
    ),
    (
        ThemeArg::Aubergine,
        ((46, 24, 73), (82, 36, 114), (106, 212, 243)),
    ),
    (
        ThemeArg::Hoth,
        ((229, 236, 247), (204, 218, 236), (32, 109, 167)),
    ),
    (
        ThemeArg::Monument,
        ((17, 33, 33), (33, 58, 57), (242, 176, 68)),
    ),
    (
        ThemeArg::Nord,
        ((46, 52, 64), (59, 66, 82), (136, 192, 208)),
    ),
    (
        ThemeArg::CatppuccinMocha,
        ((30, 30, 46), (49, 50, 68), (203, 166, 247)),
    ),
    (
        ThemeArg::Mono,
        ((17, 17, 24), (32, 35, 44), (196, 201, 214)),
    ),
    (
        ThemeArg::HighContrast,
        ((0, 0, 0), (10, 10, 16), (255, 210, 0)),
    ),
    (
        ThemeArg::Dracula,
        ((40, 42, 54), (68, 71, 90), (189, 147, 249)),
    ),
    (
        ThemeArg::GruvboxMaterialDark,
        ((40, 40, 40), (60, 56, 54), (216, 166, 87)),
    ),
    (
        ThemeArg::KanagawaWave,
        ((31, 31, 40), (43, 46, 58), (152, 187, 108)),
    ),
    (
        ThemeArg::AyuMirage,
        ((31, 36, 48), (46, 53, 71), (255, 173, 102)),
    ),
    (
        ThemeArg::AyuLight,
        ((248, 249, 250), (232, 236, 242), (255, 148, 64)),
    ),
    (
        ThemeArg::PoimandresStorm,
        ((37, 43, 55), (56, 65, 84), (209, 159, 232)),
    ),
    (
        ThemeArg::SelenizedDark,
        ((16, 60, 72), (24, 73, 86), (90, 170, 255)),
    ),
    (
        ThemeArg::NoClownFiesta,
        ((16, 16, 16), (33, 37, 45), (179, 168, 241)),
    ),
];

pub(super) const BASIC16_MODE_PALETTES: &[(ThemeArg, Basic16Palette)] = &[
    (
        ThemeArg::Aurora,
        (
            Color::Blue,
            Color::Cyan,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightCyan,
            Color::White,
        ),
    ),
    (
        ThemeArg::MidnightCyan,
        (
            Color::Blue,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightCyan,
            Color::Yellow,
        ),
    ),
    (
        ThemeArg::Aubergine,
        (
            Color::Magenta,
            Color::Blue,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightMagenta,
            Color::Yellow,
        ),
    ),
    (
        ThemeArg::Hoth,
        (
            Color::Gray,
            Color::White,
            Color::DarkGray,
            Color::Blue,
            Color::Blue,
            Color::Black,
        ),
    ),
    (
        ThemeArg::Monument,
        (
            Color::Black,
            Color::DarkGray,
            Color::DarkGray,
            Color::Yellow,
            Color::LightGreen,
            Color::White,
        ),
    ),
    (
        ThemeArg::Nord,
        (
            Color::Blue,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightBlue,
            Color::White,
        ),
    ),
    (
        ThemeArg::CatppuccinMocha,
        (
            Color::Magenta,
            Color::Blue,
            Color::DarkGray,
            Color::LightMagenta,
            Color::LightCyan,
            Color::White,
        ),
    ),
    (
        ThemeArg::Mono,
        (
            Color::Black,
            Color::DarkGray,
            Color::DarkGray,
            Color::White,
            Color::Gray,
            Color::White,
        ),
    ),
    (
        ThemeArg::HighContrast,
        (
            Color::Black,
            Color::Black,
            Color::Black,
            Color::Yellow,
            Color::White,
            Color::Yellow,
        ),
    ),
    (
        ThemeArg::Dracula,
        (
            Color::Magenta,
            Color::Blue,
            Color::DarkGray,
            Color::LightMagenta,
            Color::LightMagenta,
            Color::White,
        ),
    ),
    (
        ThemeArg::GruvboxMaterialDark,
        (
            Color::Black,
            Color::DarkGray,
            Color::DarkGray,
            Color::Yellow,
            Color::Yellow,
            Color::White,
        ),
    ),
    (
        ThemeArg::KanagawaWave,
        (
            Color::Green,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightGreen,
            Color::LightGreen,
            Color::White,
        ),
    ),
    (
        ThemeArg::AyuMirage,
        (
            Color::Yellow,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightYellow,
            Color::Yellow,
            Color::White,
        ),
    ),
    (
        ThemeArg::AyuLight,
        (
            Color::Gray,
            Color::White,
            Color::DarkGray,
            Color::Red,
            Color::Yellow,
            Color::Black,
        ),
    ),
    (
        ThemeArg::PoimandresStorm,
        (
            Color::Magenta,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightMagenta,
            Color::LightMagenta,
            Color::White,
        ),
    ),
    (
        ThemeArg::SelenizedDark,
        (
            Color::Cyan,
            Color::Blue,
            Color::DarkGray,
            Color::LightCyan,
            Color::LightBlue,
            Color::White,
        ),
    ),
    (
        ThemeArg::NoClownFiesta,
        (
            Color::Black,
            Color::DarkGray,
            Color::DarkGray,
            Color::LightMagenta,
            Color::Magenta,
            Color::White,
        ),
    ),
];

#[cfg(test)]
pub(super) const ALL_NON_AUTO_THEMES: &[ThemeArg] = &[
    ThemeArg::Aurora,
    ThemeArg::MidnightCyan,
    ThemeArg::Aubergine,
    ThemeArg::Hoth,
    ThemeArg::Monument,
    ThemeArg::Nord,
    ThemeArg::CatppuccinMocha,
    ThemeArg::Mono,
    ThemeArg::HighContrast,
    ThemeArg::Dracula,
    ThemeArg::GruvboxMaterialDark,
    ThemeArg::KanagawaWave,
    ThemeArg::AyuMirage,
    ThemeArg::AyuLight,
    ThemeArg::PoimandresStorm,
    ThemeArg::SelenizedDark,
    ThemeArg::NoClownFiesta,
];
