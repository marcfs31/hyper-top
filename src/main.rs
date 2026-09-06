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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::channel(10);
    let (command_tx, command_rx) = mpsc::channel(10);
    telemetry::spawn_telemetry_engine(tx, command_rx);

    let mut app = App::new();

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
