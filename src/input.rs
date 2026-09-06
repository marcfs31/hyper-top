use crate::{
    app::{App, DisplayColumn, InputMode, SortMode},
    telemetry::ProcessCommand,
};
use crossterm::event::{KeyCode, KeyModifiers};
use tokio::sync::mpsc;

const HELP_SCROLL_END: usize = 16;

pub async fn handle_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    command_tx: &mpsc::Sender<ProcessCommand>,
) {
    if app.input_mode == InputMode::Help {
        match code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Up => app.help_scroll = app.help_scroll.saturating_sub(1),
            KeyCode::Down => app.help_scroll = app.help_scroll.saturating_add(1),
            KeyCode::PageUp => app.help_scroll = app.help_scroll.saturating_sub(8),
            KeyCode::PageDown => app.help_scroll = app.help_scroll.saturating_add(8),
            KeyCode::Home => app.help_scroll = 0,
            KeyCode::End => app.help_scroll = HELP_SCROLL_END,
            _ => {}
        }
        return;
    }

    if app.input_mode == InputMode::Search {
        match code {
            KeyCode::Enter => app.input_mode = InputMode::Normal,
            KeyCode::Esc => {
                app.clear_filter();
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                app.query.pop();
                app.selected_process = 0;
            }
            KeyCode::Delete => app.clear_filter(),
            KeyCode::Char(character) if modifiers == KeyModifiers::CONTROL && character == 'c' => {
                app.clear_filter();
                app.input_mode = InputMode::Normal;
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

    match code {
        KeyCode::Esc => app.clear_filter(),
        KeyCode::Char(ch)
            if !modifiers.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'q') =>
        {
            app.is_running = false;
        }
        KeyCode::Char(ch)
            if !modifiers.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'f') =>
        {
            app.clear_filter();
        }
        KeyCode::Tab | KeyCode::BackTab => app.toggle_focus(),
        KeyCode::Char('?') => {
            app.help_scroll = 0;
            app.input_mode = InputMode::Help;
        }
        KeyCode::Char('/') => app.input_mode = InputMode::Search,
        KeyCode::Char(' ') => app.paused = !app.paused,
        KeyCode::Char(ch)
            if !modifiers.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'s') =>
        {
            app.cycle_sort();
        }
        KeyCode::Char(ch)
            if !modifiers.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'z') =>
        {
            app.toggle_compact_mode();
        }
        KeyCode::Char(ch)
            if !modifiers.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'r') =>
        {
            app.cycle_refresh_interval();
            let _ = command_tx
                .send(ProcessCommand::SetRefreshInterval(
                    app.config.refresh_interval_ms,
                ))
                .await;
        }
        KeyCode::Char(ch)
            if !modifiers.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'a') =>
        {
            app.config.refresh_interval_ms = 900;
            let _ = command_tx
                .send(ProcessCommand::SetRefreshInterval(
                    app.config.refresh_interval_ms,
                ))
                .await;
        }
        KeyCode::Char('T') => app.cycle_theme(),
        KeyCode::Char(ch)
            if !modifiers.contains(KeyModifiers::CONTROL) && matches!(ch, '[' | ']') =>
        {
            app.adjust_process_limit(if ch == '[' { -10 } else { 10 });
        }
        KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Char('{') => {
            app.adjust_process_limit(-10)
        }
        KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('}') => {
            app.adjust_process_limit(10)
        }
        KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'u') => app.cycle_process_limit(),
        KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'v') => app.toggle_expanded_process_view(),
        KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'e') => app.toggle_tree_view(),
        KeyCode::PageUp => app.move_selection(-10),
        KeyCode::PageDown => app.move_selection(10),
        KeyCode::Home => app.selected_process = 0,
        KeyCode::End => {
            app.selected_process = app.filtered_processes().len().saturating_sub(1);
        }
        KeyCode::Enter => app.toggle_process_focus(),
        KeyCode::Char('0') => {
            app.sort_mode = SortMode::None;
            app.selected_process = 0;
        }
        KeyCode::Char(ch) if !modifiers.contains(KeyModifiers::CONTROL) && ch.is_ascii_digit() => {
            match ch.to_digit(10).unwrap_or_default() as usize {
                1..=5 => {
                    let _ =
                        app.apply_quick_filter(ch.to_digit(10).unwrap_or_default() as usize - 1);
                }
                6 => {
                    let _ = app.apply_quick_filter(5);
                }
                7 => app.toggle_visible_column(DisplayColumn::Parent),
                8 => app.toggle_visible_column(DisplayColumn::Runtime),
                9 => app.toggle_visible_column(DisplayColumn::Status),
                _ => {}
            }
        }
        KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'x') && app.selected_pid().is_some() => {
            app.input_mode = InputMode::ConfirmKill;
        }
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
        KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
        KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'c') => app.sort_mode = SortMode::Cpu,
        KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'m') => app.sort_mode = SortMode::Memory,
        KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'n') => app.sort_mode = SortMode::Name,
        KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'p') => app.sort_mode = SortMode::Pid,
        KeyCode::Char('t') => app.sort_mode = SortMode::Threads,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppConfig;

    fn channel() -> mpsc::Sender<ProcessCommand> {
        let (tx, _rx) = mpsc::channel(1);
        tx
    }

    #[tokio::test]
    async fn shortcuts_update_core_modes() {
        let mut app = App::with_config(AppConfig::default());
        let tx = channel();

        handle_key(&mut app, KeyCode::Char('v'), KeyModifiers::NONE, &tx).await;
        assert!(app.expanded_process_view);
        handle_key(&mut app, KeyCode::Char('?'), KeyModifiers::NONE, &tx).await;
        assert_eq!(app.input_mode, InputMode::Help);
        handle_key(&mut app, KeyCode::Esc, KeyModifiers::NONE, &tx).await;
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[tokio::test]
    async fn refresh_and_limit_shortcuts_are_applied() {
        let mut app = App::with_config(AppConfig::default());
        app.config.max_processes = 80;
        let tx = channel();

        handle_key(&mut app, KeyCode::Char('a'), KeyModifiers::NONE, &tx).await;
        assert_eq!(app.config.refresh_interval_ms, 900);
        handle_key(&mut app, KeyCode::Char('+'), KeyModifiers::SHIFT, &tx).await;
        assert_eq!(app.config.max_processes, 90);
        handle_key(&mut app, KeyCode::Char('u'), KeyModifiers::NONE, &tx).await;
        assert_eq!(app.config.max_processes, 0);
    }

    #[tokio::test]
    async fn e_toggles_process_tree_view() {
        let mut app = App::with_config(AppConfig::default());
        let tx = channel();

        handle_key(&mut app, KeyCode::Char('e'), KeyModifiers::NONE, &tx).await;
        assert!(app.tree_view);
        handle_key(&mut app, KeyCode::Char('E'), KeyModifiers::SHIFT, &tx).await;
        assert!(!app.tree_view);
    }
}
