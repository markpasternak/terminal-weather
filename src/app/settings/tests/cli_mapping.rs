use super::*;

#[test]
fn from_cli_defaults_basic_mapping() {
    let mut cli = default_cli();
    cli.units = UnitsArg::Fahrenheit;
    cli.theme = ThemeArg::Nord;
    cli.no_flash = true;
    cli.hero_visual = HeroVisualArg::GaugeCluster;
    cli.refresh_interval = 300;

    let settings = RuntimeSettings::from_cli_defaults(&cli);

    assert_eq!(settings.units, crate::domain::weather::Units::Fahrenheit);
    assert_eq!(settings.theme, ThemeArg::Nord);
    assert!(settings.no_flash);
    assert_eq!(settings.hero_visual, HeroVisualArg::GaugeCluster);
    assert_eq!(settings.refresh_interval_secs, 300);

    let settings_default = RuntimeSettings::from_cli_defaults(&default_cli());
    assert_eq!(
        settings_default.units,
        crate::domain::weather::Units::Celsius
    );
}

#[test]
fn from_cli_defaults_ignores_animation_flags() {
    let baseline = RuntimeSettings::from_cli_defaults(&default_cli());

    let mut reduced = default_cli();
    reduced.reduced_motion = true;
    let reduced_settings = RuntimeSettings::from_cli_defaults(&reduced);
    assert_eq!(reduced_settings.units, baseline.units);
    assert_eq!(reduced_settings.theme, baseline.theme);
    assert_eq!(reduced_settings.icon_mode, baseline.icon_mode);
    assert_eq!(reduced_settings.hourly_view, baseline.hourly_view);

    let mut off = default_cli();
    off.no_animation = true;
    let off_settings = RuntimeSettings::from_cli_defaults(&off);
    assert_eq!(off_settings.units, baseline.units);
    assert_eq!(off_settings.theme, baseline.theme);
    assert_eq!(off_settings.icon_mode, baseline.icon_mode);
    assert_eq!(off_settings.hourly_view, baseline.hourly_view);
}

#[test]
fn from_cli_defaults_icon_mode_logic() {
    let cli = default_cli();
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&cli).icon_mode,
        IconMode::Unicode
    );

    let mut emoji = default_cli();
    emoji.emoji_icons = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&emoji).icon_mode,
        IconMode::Emoji
    );

    let mut ascii = default_cli();
    ascii.ascii_icons = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&ascii).icon_mode,
        IconMode::Ascii
    );

    let mut nerd = default_cli();
    nerd.nerd_font = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&nerd).icon_mode,
        IconMode::NerdFont
    );

    let mut precedence = default_cli();
    precedence.ascii_icons = true;
    precedence.emoji_icons = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&precedence).icon_mode,
        IconMode::Ascii
    );
}

#[test]
fn from_cli_defaults_hardcoded_fields() {
    let mut cli = default_cli();
    cli.hourly_view = Some(HourlyViewArg::Chart);

    let settings = RuntimeSettings::from_cli_defaults(&cli);

    assert_eq!(settings.hourly_view, HourlyViewMode::Table);
    assert!(settings.recent_locations.is_empty());
}

#[test]
fn override_theme_non_auto_updates_settings() {
    let mut cli = default_cli();
    cli.theme = ThemeArg::Nord;
    let settings = RuntimeSettings::from_cli_defaults(&cli);
    assert_eq!(settings.theme, ThemeArg::Nord);
}

#[test]
fn override_icon_mode_emoji_sets_emoji() {
    let mut cli = default_cli();
    cli.emoji_icons = true;
    let settings = RuntimeSettings::from_cli_defaults(&cli);
    assert_eq!(settings.icon_mode, IconMode::Emoji);
}

#[test]
fn override_refresh_interval_non_default_updates() {
    let mut cli = default_cli();
    cli.refresh_interval = 300;
    let settings = RuntimeSettings::from_cli_defaults(&cli);
    assert_eq!(settings.refresh_interval_secs, 300);
}
