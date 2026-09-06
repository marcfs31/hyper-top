mod app;
mod telemetry;
mod ui;

use app::{App, InputMode, SortMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use telemetry::ProcessCommand;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut app = match App::from_cli_args(args) {
        Ok(app) => app,
        Err(error) => {
            eprintln!("Failed to parse CLI arguments: {error}");
            std::process::exit(1);
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::channel(10);
    let (command_tx, command_rx) = mpsc::channel(10);
    telemetry::spawn_telemetry_engine(
        tx,
        command_rx,
        std::time::Duration::from_millis(app.config.refresh_interval_ms),
    );

    while app.is_running {
        terminal.draw(|f| ui::draw(f, &app))?;

        if !app.paused {
            if let Ok(new_state) = rx.try_recv() {
                app.system_state = new_state;
                let count = app.filtered_processes().len();
                app.selected_process = app.selected_process.min(count.saturating_sub(1));
            }
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key.code, key.modifiers, &command_tx).await;
            }
        }
    }

    if let Some(path) = app.config_path.clone() {
        let _ = app.save_config(path);
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn handle_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    command_tx: &mpsc::Sender<ProcessCommand>,
) {
    if app.input_mode == InputMode::Help {
        if matches!(code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')) {
            app.input_mode = InputMode::Normal;
        }
        return;
    }

    if app.input_mode == InputMode::Search {
        match code {
            KeyCode::Esc | KeyCode::Enter => app.input_mode = InputMode::Normal,
            KeyCode::Backspace => {
                app.query.pop();
                app.selected_process = 0;
            }
            KeyCode::Char(character) if !modifiers.contains(KeyModifiers::CONTROL) => {
                app.query.push(character);
                app.selected_process = 0;
            }
            _ => {}
        }
        return;
    }

    if app.input_mode == InputMode::ConfirmKill {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(pid) = app.selected_pid() {
                    let _ = command_tx.send(ProcessCommand::Kill(pid)).await;
                }
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        return;
    }

    match (code, modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) => app.is_running = false,
        (KeyCode::Tab, KeyModifiers::NONE) => app.toggle_focus(),
        (KeyCode::BackTab, _) => app.toggle_focus(),
        (KeyCode::Char('?'), KeyModifiers::NONE) => app.input_mode = InputMode::Help,
        (KeyCode::Char('/'), KeyModifiers::NONE) => app.input_mode = InputMode::Search,
        (KeyCode::Char(' '), KeyModifiers::NONE) => app.paused = !app.paused,
        (KeyCode::Char('s'), KeyModifiers::NONE) => app.cycle_sort(),
        (KeyCode::Char('z'), KeyModifiers::NONE) => app.toggle_compact_mode(),
        (KeyCode::Char('r'), KeyModifiers::NONE) => {
            app.cycle_refresh_interval();
            let _ = command_tx
                .send(ProcessCommand::SetRefreshInterval(
                    app.config.refresh_interval_ms,
                ))
                .await;
        }
        (KeyCode::Char('T'), KeyModifiers::NONE) => app.cycle_theme(),
        (KeyCode::Char('['), KeyModifiers::NONE) => app.adjust_process_limit(-10),
        (KeyCode::Char(']'), KeyModifiers::NONE) => app.adjust_process_limit(10),
        (KeyCode::Char('1'), KeyModifiers::NONE) => {
            let _ = app.apply_quick_filter(0);
        }
        (KeyCode::Char('2'), KeyModifiers::NONE) => {
            let _ = app.apply_quick_filter(1);
        }
        (KeyCode::Char('3'), KeyModifiers::NONE) => {
            let _ = app.apply_quick_filter(2);
        }
        (KeyCode::Char('4'), KeyModifiers::NONE) => {
            let _ = app.apply_quick_filter(3);
        }
        (KeyCode::Char('5'), KeyModifiers::NONE) => {
            let _ = app.apply_quick_filter(4);
        }
        (KeyCode::Char('6'), KeyModifiers::NONE) => {
            let _ = app.apply_quick_filter(5);
        }
        (KeyCode::Char('7'), KeyModifiers::NONE) => {
            app.toggle_visible_column_by_index(0);
        }
        (KeyCode::Char('8'), KeyModifiers::NONE) => {
            app.toggle_visible_column_by_index(1);
        }
        (KeyCode::Char('9'), KeyModifiers::NONE) => {
            app.toggle_visible_column_by_index(2);
        }
        (KeyCode::Char('x'), KeyModifiers::NONE) if app.selected_pid().is_some() => {
            app.input_mode = InputMode::ConfirmKill;
        }
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => app.move_selection(1),
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => app.move_selection(-1),
        (KeyCode::Char('c'), KeyModifiers::NONE) => app.sort_mode = SortMode::Cpu,
        (KeyCode::Char('m'), KeyModifiers::NONE) => app.sort_mode = SortMode::Memory,
        (KeyCode::Char('n'), KeyModifiers::NONE) => app.sort_mode = SortMode::Name,
        (KeyCode::Char('p'), KeyModifiers::NONE) => app.sort_mode = SortMode::Pid,
        (KeyCode::Char('t'), KeyModifiers::NONE) => app.sort_mode = SortMode::Threads,
        _ => {}
    }
}
