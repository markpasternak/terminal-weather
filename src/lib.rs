#![allow(clippy::missing_errors_doc, clippy::wildcard_imports)]

pub mod app;
pub mod cli;
pub mod data;
pub mod domain;
pub mod resilience;
#[cfg(test)]
mod test_support;
pub mod ui;

use std::io::{self, IsTerminal, Stdout};

use anyhow::{Context, Result};
use app::events::{AppEvent, spawn_input_task};
use app::state::{AppMode, AppState};
use cli::Cli;
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

pub async fn run(cli: Cli) -> Result<()> {
    if cli.one_shot {
        return run_one_shot(&cli).await;
    }
    let mut terminal = setup_terminal()?;
    let result = run_inner(&mut terminal, cli).await;
    restore_terminal(&mut terminal)?;
    result
}

async fn run_one_shot(cli: &Cli) -> Result<()> {
    use crate::data::forecast::ForecastClient;
    use crate::data::geocode::GeocodeClient;

    let (units, unit_symbol) = one_shot_units(cli.units);
    let location = resolve_one_shot_location(cli, &GeocodeClient::new()).await?;
    let display_name = location.display_name();
    let bundle = ForecastClient::new().fetch(location).await?;

    print_one_shot_current(&bundle, &display_name, units, unit_symbol);
    print_one_shot_daily(&bundle, units, one_shot_icon_mode(cli));

    Ok(())
}

fn one_shot_units(
    units_arg: crate::cli::UnitsArg,
) -> (crate::domain::weather::Units, &'static str) {
    use crate::cli::UnitsArg;
    use crate::domain::weather::Units;

    match units_arg {
        UnitsArg::Celsius => (Units::Celsius, "C"),
        UnitsArg::Fahrenheit => (Units::Fahrenheit, "F"),
    }
}

async fn resolve_one_shot_location(
    cli: &Cli,
    geocoder: &crate::data::geocode::GeocodeClient,
) -> Result<crate::domain::weather::Location> {
    use crate::domain::weather::{GeocodeResolution, Location};

    if let (Some(lat), Some(lon)) = (cli.lat, cli.lon) {
        return Ok(match geocoder.reverse_resolve(lat, lon).await {
            Ok(Some(location)) => location,
            Ok(None) | Err(_) => Location::from_coords(lat, lon),
        });
    }

    let city = cli.default_city();
    match geocoder.resolve(city, cli.country_code.clone()).await? {
        GeocodeResolution::Selected(loc) => Ok(loc),
        GeocodeResolution::NeedsDisambiguation(locs) => {
            locs.into_iter().next().context("no locations found")
        }
        GeocodeResolution::NotFound(name) => anyhow::bail!("City not found: {name}"),
    }
}

fn one_shot_icon_mode(cli: &Cli) -> crate::cli::IconMode {
    use crate::cli::IconMode;

    if cli.ascii_icons {
        IconMode::Ascii
    } else if cli.emoji_icons {
        IconMode::Emoji
    } else {
        IconMode::Unicode
    }
}

fn print_one_shot_current(
    bundle: &crate::domain::weather::ForecastBundle,
    display_name: &str,
    units: crate::domain::weather::Units,
    unit_symbol: &str,
) {
    use crate::domain::weather::{convert_temp, round_temp, round_wind_speed, weather_label};

    let temp = round_temp(convert_temp(bundle.current.temperature_2m_c, units));
    let feels = round_temp(convert_temp(bundle.current.apparent_temperature_c, units));
    let condition = weather_label(bundle.current.weather_code);
    let wind = round_wind_speed(bundle.current.wind_speed_10m);
    let gust = round_wind_speed(bundle.current.wind_gusts_10m);
    let humidity = format!("{:.0}", bundle.current.relative_humidity_2m);
    let pressure = format!("{:.0}", bundle.current.pressure_msl_hpa);
    let vis_km = bundle.current.visibility_m / 1000.0;

    println!("  {display_name}");
    println!("  {temp}°{unit_symbol}  {condition}");
    println!("  Feels {feels}°{unit_symbol}  Humidity {humidity}%  Wind {wind}/{gust} m/s");
    println!("  Pressure {pressure}hPa  Visibility {vis_km:.1}km");
    println!();
}

fn print_one_shot_daily(
    bundle: &crate::domain::weather::ForecastBundle,
    units: crate::domain::weather::Units,
    icon_mode: crate::cli::IconMode,
) {
    use crate::domain::weather::{convert_temp, round_temp, weather_icon};

    println!("  7-Day Forecast");
    for day in &bundle.daily {
        let day_name = day.date.format("%a %d").to_string();
        let icon = day
            .weather_code
            .map_or("--", |c| weather_icon(c, icon_mode, true));
        let min = day
            .temperature_min_c
            .map(|t| round_temp(convert_temp(t, units)));
        let max = day
            .temperature_max_c
            .map(|t| round_temp(convert_temp(t, units)));
        print_daily_line(&day_name, icon, min, max, day.precipitation_sum_mm);
    }
}

fn print_daily_line(
    day_name: &str,
    icon: &str,
    min: Option<i32>,
    max: Option<i32>,
    precip_mm: Option<f32>,
) {
    let min_str = min.map_or_else(|| "--".to_string(), |v| format!("{v}°"));
    let max_str = max.map_or_else(|| "--".to_string(), |v| format!("{v}°"));
    let precip = precip_mm.map_or_else(|| "--".to_string(), |p| format!("{p:.1}mm"));
    println!("  {day_name:<8} {icon:<4} {min_str:>4} / {max_str:<4}  {precip}");
}

async fn run_inner(terminal: &mut Terminal<CrosstermBackend<Stdout>>, cli: Cli) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<AppEvent>(256);
    let input_stream = spawn_input_task();
    tokio::pin!(input_stream);
    let mut app = AppState::new(&cli);

    tx.send(AppEvent::Bootstrap).await?;

    while app.running {
        tokio::select! {
            maybe_input = input_stream.next() => {
                handle_input_event(&mut app, maybe_input, &tx, &cli).await?;
            }
            maybe_event = rx.recv() => {
                handle_app_event(terminal, &mut app, maybe_event, &tx, &cli).await?;
            }
        }

        app.viewport_width = terminal.size()?.width;
        terminal.draw(|frame| ui::render(frame, &app, &cli))?;

        if app.mode == AppMode::Quit {
            app.running = false;
        }
    }

    Ok(())
}

async fn handle_input_event(
    app: &mut AppState,
    maybe_input: Option<crossterm::event::Event>,
    tx: &mpsc::Sender<AppEvent>,
    cli: &Cli,
) -> Result<()> {
    if let Some(input) = maybe_input {
        app.handle_event(AppEvent::Input(input), tx, cli).await?;
    }
    Ok(())
}

async fn handle_app_event(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut AppState,
    maybe_event: Option<AppEvent>,
    tx: &mpsc::Sender<AppEvent>,
    cli: &Cli,
) -> Result<()> {
    if let Some(event) = maybe_event {
        if matches!(event, AppEvent::ForceRedraw) {
            terminal.clear()?;
        }
        app.handle_event(event, tx, cli).await?;
    }
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    if !io::stdout().is_terminal() {
        anyhow::bail!(
            "terminal-weather requires an interactive TTY. Run it in a terminal, or use --help for CLI usage."
        );
    }
    install_panic_hook();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn install_panic_hook() {
    let existing = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        existing(panic);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{IconMode, UnitsArg};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    fn one_shot_cli() -> Cli {
        let mut cli = crate::test_support::settings_default_test_cli();
        cli.one_shot = true;
        cli
    }

    #[test]
    fn one_shot_units_maps_cli_arg() {
        assert_eq!(
            one_shot_units(UnitsArg::Celsius),
            (crate::domain::weather::Units::Celsius, "C")
        );
        assert_eq!(
            one_shot_units(UnitsArg::Fahrenheit),
            (crate::domain::weather::Units::Fahrenheit, "F")
        );
    }

    #[test]
    fn one_shot_icon_mode_has_expected_precedence() {
        let mut cli = one_shot_cli();
        assert_eq!(one_shot_icon_mode(&cli), IconMode::Unicode);

        cli.emoji_icons = true;
        assert_eq!(one_shot_icon_mode(&cli), IconMode::Emoji);

        cli.ascii_icons = true;
        assert_eq!(one_shot_icon_mode(&cli), IconMode::Ascii);
    }

    #[tokio::test]
    async fn resolve_one_shot_location_uses_reverse_geocode_for_coordinates() {
        let mut cli = one_shot_cli();
        cli.lat = Some(59.3293);
        cli.lon = Some(18.0686);
        let server = MockServer::start().await;
        let payload = serde_json::json!({
            "address": {
                "city": "Stockholm",
                "state": "Stockholm County",
                "country": "Sweden"
            }
        });
        Mock::given(method("GET"))
            .and(path("/v1/reverse"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;
        let geocoder = crate::data::geocode::GeocodeClient::with_base_url(format!(
            "{}/v1/search",
            server.uri()
        ));
        let location = resolve_one_shot_location(&cli, &geocoder)
            .await
            .expect("coords resolve");
        assert_eq!(location.name, "Stockholm");
    }

    #[tokio::test]
    async fn resolve_one_shot_location_falls_back_to_raw_coordinates_on_reverse_failure() {
        let mut cli = one_shot_cli();
        cli.lat = Some(59.3293);
        cli.lon = Some(18.0686);
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/reverse"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;
        let geocoder = crate::data::geocode::GeocodeClient::with_base_url(format!(
            "{}/v1/search",
            server.uri()
        ));
        let location = resolve_one_shot_location(&cli, &geocoder)
            .await
            .expect("coords fallback resolve");
        assert_eq!(location.name, "59.3293, 18.0686");
    }

    #[tokio::test]
    async fn resolve_one_shot_location_uses_geocoder_selection() {
        let server = MockServer::start().await;
        let payload = serde_json::json!({
            "results": [{
                "name": "Stockholm",
                "latitude": 59.3293,
                "longitude": 18.0686,
                "country": "Sweden",
                "country_code": "SE",
                "admin1": "Stockholm",
                "timezone": "Europe/Stockholm",
                "population": 975_000
            }]
        });
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;

        let mut cli = one_shot_cli();
        cli.city = Some("Stockholm".to_string());
        let geocoder = crate::data::geocode::GeocodeClient::with_base_url(server.uri());
        let location = resolve_one_shot_location(&cli, &geocoder)
            .await
            .expect("selection resolve");
        assert_eq!(location.name, "Stockholm");
    }

    #[tokio::test]
    async fn resolve_one_shot_location_errors_on_not_found() {
        let server = MockServer::start().await;
        let payload = serde_json::json!({ "results": [] });
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;

        let mut cli = one_shot_cli();
        cli.city = Some("Missing".to_string());
        let geocoder = crate::data::geocode::GeocodeClient::with_base_url(server.uri());
        let err = resolve_one_shot_location(&cli, &geocoder)
            .await
            .expect_err("not found should error");
        assert!(format!("{err}").contains("City not found"));
    }

    #[tokio::test]
    async fn resolve_one_shot_location_disambiguation_picks_first() {
        let server = MockServer::start().await;
        let payload = serde_json::json!({
            "results": [
                {
                    "name": "Springfield",
                    "latitude": 39.78,
                    "longitude": -89.65,
                    "country": "United States",
                    "country_code": "US",
                    "admin1": "Illinois",
                    "timezone": "America/Chicago",
                    "population": 100_000
                },
                {
                    "name": "Springfield",
                    "latitude": 37.21,
                    "longitude": -93.29,
                    "country": "United States",
                    "country_code": "US",
                    "admin1": "Missouri",
                    "timezone": "America/Chicago",
                    "population": 105_000
                }
            ]
        });
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;

        let mut cli = one_shot_cli();
        cli.city = Some("Springfield".to_string());
        let geocoder = crate::data::geocode::GeocodeClient::with_base_url(server.uri());
        let location = resolve_one_shot_location(&cli, &geocoder)
            .await
            .expect("disambiguation should pick first");
        assert_eq!(location.name, "Springfield");
        assert!((location.latitude - 37.21).abs() < 0.01);
    }

    #[test]
    fn print_helpers_execute_for_sample_bundle() {
        let bundle = crate::test_support::sample_bundle();
        print_one_shot_current(
            &bundle,
            "Stockholm, Sweden",
            crate::domain::weather::Units::Celsius,
            "C",
        );
        print_daily_line("Thu 12", "☀", Some(1), Some(8), Some(2.4));
        print_daily_line("Fri 13", "--", None, None, None);
        print_one_shot_daily(
            &bundle,
            crate::domain::weather::Units::Celsius,
            IconMode::Unicode,
        );
    }

    #[test]
    fn one_shot_icon_mode_ascii_takes_precedence() {
        let mut cli = one_shot_cli();
        cli.ascii_icons = true;
        cli.emoji_icons = true;
        assert_eq!(one_shot_icon_mode(&cli), IconMode::Ascii);
    }
}
