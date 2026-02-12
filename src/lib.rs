pub mod app;
pub mod cli;
pub mod data;
pub mod domain;
pub mod resilience;
pub mod ui;

use std::io::{self, Stdout};

use anyhow::Result;
use app::events::{AppEvent, spawn_input_task};
use app::state::{AppMode, AppState};
use cli::{Cli, IconMode};
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

pub async fn run(cli: Cli) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_inner(&mut terminal, cli).await;
    restore_terminal(&mut terminal)?;
    result
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
                    app.handle_event(event, &tx, &cli).await?;
                }
            }
        }

        terminal.draw(|frame| ui::render(frame, &app, &cli))?;

        if app.mode == AppMode::Quit {
            app.running = false;
        }
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
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

pub fn icon_mode(cli: &Cli) -> IconMode {
    if cli.ascii_icons {
        IconMode::Ascii
    } else if cli.emoji_icons {
        IconMode::Emoji
    } else {
        IconMode::Unicode
    }
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
