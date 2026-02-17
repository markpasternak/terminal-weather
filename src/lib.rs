pub mod app;
pub mod cli;
pub mod data;
pub mod domain;
pub mod resilience;
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
    use crate::cli::{IconMode, UnitsArg};
    use crate::data::forecast::ForecastClient;
    use crate::data::geocode::GeocodeClient;
    use crate::domain::weather::{
        GeocodeResolution, Location, Units, convert_temp, round_temp, weather_icon, weather_label,
    };

    let units = match cli.units {
        UnitsArg::Celsius => Units::Celsius,
        UnitsArg::Fahrenheit => Units::Fahrenheit,
    };
    let unit_symbol = match units {
        Units::Celsius => "C",
        Units::Fahrenheit => "F",
    };

    // Resolve location
    let location = if let (Some(lat), Some(lon)) = (cli.lat, cli.lon) {
        Location::from_coords(lat, lon)
    } else {
        let geocoder = GeocodeClient::new();
        let city = cli.default_city();
        match geocoder
            .resolve(city.clone(), cli.country_code.clone())
            .await?
        {
            GeocodeResolution::Selected(loc) => loc,
            GeocodeResolution::NeedsDisambiguation(locs) => {
                locs.into_iter().next().context("no locations found")?
            }
            GeocodeResolution::NotFound(name) => {
                anyhow::bail!("City not found: {name}");
            }
        }
    };

    let display_name = location.display_name();

    // Fetch weather
    let client = ForecastClient::new();
    let bundle = client.fetch(location).await?;

    // Print current conditions
    let temp = round_temp(convert_temp(bundle.current.temperature_2m_c, units));
    let feels = round_temp(convert_temp(bundle.current.apparent_temperature_c, units));
    let condition = weather_label(bundle.current.weather_code);
    let wind = bundle.current.wind_speed_10m.round() as i32;
    let gust = bundle.current.wind_gusts_10m.round() as i32;
    let humidity = bundle.current.relative_humidity_2m.round() as i32;
    let pressure = bundle.current.pressure_msl_hpa.round() as i32;
    let vis_km = bundle.current.visibility_m / 1000.0;

    println!("  {display_name}");
    println!("  {temp}°{unit_symbol}  {condition}");
    println!("  Feels {feels}°{unit_symbol}  Humidity {humidity}%  Wind {wind}/{gust} km/h");
    println!("  Pressure {pressure}hPa  Visibility {vis_km:.1}km");
    println!();

    // Print daily forecast
    println!("  7-Day Forecast");
    let icon_mode = if cli.ascii_icons {
        IconMode::Ascii
    } else if cli.emoji_icons {
        IconMode::Emoji
    } else {
        IconMode::Unicode
    };
    for day in &bundle.daily {
        let day_name = day.date.format("%a %d").to_string();
        let icon = day
            .weather_code
            .map(|c| weather_icon(c, icon_mode, true))
            .unwrap_or("--");
        let min = day
            .temperature_min_c
            .map(|t| round_temp(convert_temp(t, units)));
        let max = day
            .temperature_max_c
            .map(|t| round_temp(convert_temp(t, units)));
        let min_str = min
            .map(|v| format!("{v}°"))
            .unwrap_or_else(|| "--".to_string());
        let max_str = max
            .map(|v| format!("{v}°"))
            .unwrap_or_else(|| "--".to_string());
        let precip = day
            .precipitation_sum_mm
            .map(|p| format!("{p:.1}mm"))
            .unwrap_or_else(|| "--".to_string());
        println!("  {day_name:<8} {icon:<4} {min_str:>4} / {max_str:<4}  {precip}");
    }

    Ok(())
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
                if let Some(input) = maybe_input {
                    app.handle_event(AppEvent::Input(input), &tx, &cli).await?;
                }
            }
            maybe_event = rx.recv() => {
                if let Some(event) = maybe_event {
                    if matches!(event, AppEvent::ForceRedraw) {
                        terminal.clear()?;
                    }
                    app.handle_event(event, &tx, &cli).await?;
                }
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
