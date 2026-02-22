use clap::ValueEnum;

use crate::{
    cli::ThemeArg,
    domain::weather::{HourlyViewMode, Units},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyCommand {
    Quit,
    OpenSettings,
    OpenCityPicker,
    Refresh,
    SetFahrenheit,
    SetCelsius,
    CycleHourlyView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CommandAction {
    Refresh,
    Quit,
    Units(Units),
    View(HourlyViewMode),
    Theme(ThemeArg),
    City(String),
}

pub(super) fn command_from_char(cmd: char) -> Option<KeyCommand> {
    const KEY_COMMANDS: [(char, KeyCommand); 7] = [
        ('q', KeyCommand::Quit),
        ('s', KeyCommand::OpenSettings),
        ('l', KeyCommand::OpenCityPicker),
        ('r', KeyCommand::Refresh),
        ('f', KeyCommand::SetFahrenheit),
        ('c', KeyCommand::SetCelsius),
        ('v', KeyCommand::CycleHourlyView),
    ];

    KEY_COMMANDS
        .iter()
        .find_map(|(target, action)| (*target == cmd).then_some(*action))
}

pub(super) fn parse_units_command(value: &str) -> Option<Units> {
    if value.eq_ignore_ascii_case("c") || value.eq_ignore_ascii_case("celsius") {
        Some(Units::Celsius)
    } else if value.eq_ignore_ascii_case("f") || value.eq_ignore_ascii_case("fahrenheit") {
        Some(Units::Fahrenheit)
    } else {
        None
    }
}

pub(super) fn parse_hourly_view_command(value: &str) -> Option<HourlyViewMode> {
    if value.eq_ignore_ascii_case("table") {
        Some(HourlyViewMode::Table)
    } else if value.eq_ignore_ascii_case("hybrid") {
        Some(HourlyViewMode::Hybrid)
    } else if value.eq_ignore_ascii_case("chart") {
        Some(HourlyViewMode::Chart)
    } else {
        None
    }
}

pub(super) fn parse_command_action(command: &str) -> std::result::Result<CommandAction, String> {
    let mut parts = command.split_whitespace();
    let Some(verb) = parts.next().map(str::to_ascii_lowercase) else {
        return Ok(CommandAction::Refresh);
    };
    let rest: Vec<&str> = parts.collect();
    match verb.as_str() {
        "refresh" => Ok(CommandAction::Refresh),
        "quit" => Ok(CommandAction::Quit),
        "units" => cmd_units(&rest),
        "view" => cmd_view(&rest),
        "theme" => cmd_theme(&rest),
        "city" => cmd_city(&rest),
        _ => Err(format!("unknown command: {verb}")),
    }
}

fn cmd_units(args: &[&str]) -> std::result::Result<CommandAction, String> {
    let value = args
        .first()
        .ok_or_else(|| "usage: :units c|f".to_string())?;
    parse_units_command(value)
        .map(CommandAction::Units)
        .ok_or_else(|| "usage: :units c|f".to_string())
}

fn cmd_view(args: &[&str]) -> std::result::Result<CommandAction, String> {
    let value = args
        .first()
        .ok_or_else(|| "usage: :view table|hybrid|chart".to_string())?;
    parse_hourly_view_command(value)
        .map(CommandAction::View)
        .ok_or_else(|| "usage: :view table|hybrid|chart".to_string())
}

fn cmd_theme(args: &[&str]) -> std::result::Result<CommandAction, String> {
    let value = args
        .first()
        .ok_or_else(|| "usage: :theme <name>".to_string())?;
    ThemeArg::from_str(value, true)
        .map(CommandAction::Theme)
        .map_err(|_| format!("unknown theme: {value}"))
}

fn cmd_city(args: &[&str]) -> std::result::Result<CommandAction, String> {
    let query = args.join(" ");
    if query.trim().is_empty() {
        return Err("usage: :city <name>".to_string());
    }
    Ok(CommandAction::City(query))
}
