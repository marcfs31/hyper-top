mod app;
mod input;
mod telemetry;
mod terminal;
mod ui;

use app::App;
use crossterm::event::{self, Event};
use std::time::Duration;
use tokio::sync::mpsc;

const USAGE: &str = "Usage: hyper-top [--config PATH] [--theme default|solarized|midnight] [--refresh 900] [--limit 80] [--sort cpu|memory|name|pid|threads] [--filter QUERY] [--show-full-command|--hide-full-command] [--compact|--no-compact] [--import PATH] [--export PATH]";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{USAGE}");
        return Ok(());
    }

    let mut app = App::from_cli_args(args).map_err(|error| {
        eprintln!("Failed to parse CLI arguments: {error}");
        error
    })?;

    let mut terminal_session = terminal::TerminalSession::new()?;

    let (tx, mut rx) = mpsc::channel(10);
    let (command_tx, command_rx) = mpsc::channel(10);
    telemetry::spawn_telemetry_engine(
        tx,
        command_rx,
        Duration::from_millis(app.config.refresh_interval_ms),
    );

    while app.is_running {
        terminal_session
            .terminal_mut()
            .draw(|frame| ui::draw(frame, &app))?;
        if !app.paused {
            if let Ok(new_state) = rx.try_recv() {
                app.system_state = new_state;
                let count = app.filtered_processes().len();
                if let Some(pid) = app.focused_pid {
                    if let Some(index) = app
                        .visible_processes()
                        .iter()
                        .position(|process| process.pid.parse::<u32>().ok() == Some(pid))
                    {
                        app.selected_process = index;
                    } else {
                        app.focused_pid = None;
                        app.selected_process = app.selected_process.min(count.saturating_sub(1));
                    }
                } else {
                    app.selected_process = app.selected_process.min(count.saturating_sub(1));
                }
            }
        }
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                input::handle_key(&mut app, key.code, key.modifiers, &command_tx).await;
            }
        }
    }

    if let Some(path) = app.config_path.clone() {
        app.save_config(path)?;
    }

    Ok(())
}
