use crate::app::{App, DisplayColumn, FocusedBlock, InputMode, SortMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let palette = app.config.theme.palette();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(f.size());

    let palette_title = match app.input_mode {
        InputMode::Search => format!(" Search: {}_ ", app.query),
        _ if app.query.is_empty() => " Command Palette  (/) filter ".to_string(),
        _ => format!(" Filter: {} ", app.query),
    };
    let border_color = if app.input_mode == InputMode::Search
        || app.focused_block == FocusedBlock::CommandPalette
    {
        palette.accent
    } else {
        palette.muted
    };
    let palette_widget = Paragraph::new(Line::from(vec![
        Span::styled(" > ", Style::default().fg(palette.accent)),
        Span::raw(if app.input_mode == InputMode::Search {
            "Type a process name or PID, then press Enter"
        } else {
            "Press / to filter processes"
        }),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(palette_title)
            .style(Style::default().fg(border_color)),
    );
    f.render_widget(palette_widget, chunks[0]);

    let metrics = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let cpu_label = format!(
        "{:.1}%  {} cores @ {} MHz",
        app.system_state.cpu_usage, app.system_state.cpu_count, app.system_state.cpu_frequency_mhz
    );
    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" CPU Total "))
        .gauge_style(Style::default().fg(palette.success))
        .label(cpu_label)
        .percent(app.system_state.cpu_usage.clamp(0.0, 100.0) as u16);
    f.render_widget(cpu_gauge, metrics[0]);

    let ram_percent = if app.system_state.ram_total_gb > 0.0 {
        ((app.system_state.ram_used_gb / app.system_state.ram_total_gb) * 100.0).clamp(0.0, 100.0)
            as u16
    } else {
        0
    };
    let ram_label = if app.system_state.swap_total_gb > 0.0 {
        format!(
            "RAM {:.2} / {:.2} GB  SWAP {:.2} / {:.2} GB",
            app.system_state.ram_used_gb,
            app.system_state.ram_total_gb,
            app.system_state.swap_used_gb,
            app.system_state.swap_total_gb
        )
    } else {
        format!(
            "RAM {:.2} / {:.2} GB",
            app.system_state.ram_used_gb, app.system_state.ram_total_gb
        )
    };
    let ram_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory "))
        .gauge_style(Style::default().fg(palette.memory))
        .label(ram_label)
        .percent(ram_percent);
    f.render_widget(ram_gauge, metrics[1]);

    let info = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(chunks[2]);
    render_info(
        f,
        info[0],
        " Uptime ",
        format_duration(app.system_state.uptime),
        palette.text,
    );
    render_info(
        f,
        info[1],
        " Load Average ",
        format!(
            "{:.2}  {:.2}  {:.2}",
            app.system_state.load_average[0],
            app.system_state.load_average[1],
            app.system_state.load_average[2]
        ),
        palette.text,
    );
    render_info(
        f,
        info[2],
        " Overview ",
        format!(
            "{}  {}  {} procs  {} threads{}",
            sort_name(app.sort_mode),
            if app.paused { "PAUSED" } else { "LIVE" },
            app.system_state.running_processes,
            app.system_state.total_threads,
            if app.query.is_empty() {
                ""
            } else {
                "  FILTERED"
            }
        ),
        palette.text,
    );

    let table_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[3]);

    let columns = if app.config.visible_columns.is_empty() {
        crate::app::AppConfig::default_visible_columns()
    } else {
        app.config.visible_columns.clone()
    };
    let header = Row::new(
        columns
            .iter()
            .map(|column| column.label().to_string())
            .collect::<Vec<_>>(),
    )
    .style(Style::default().add_modifier(Modifier::BOLD));
    let constraints: Vec<_> = columns
        .iter()
        .map(|column| match column {
            DisplayColumn::Pid => Constraint::Length(8),
            DisplayColumn::Name => Constraint::Percentage(40),
            DisplayColumn::Cpu => Constraint::Percentage(15),
            DisplayColumn::Memory => Constraint::Percentage(15),
            DisplayColumn::Threads => Constraint::Length(8),
            DisplayColumn::Status => Constraint::Length(10),
            DisplayColumn::Command => Constraint::Percentage(35),
        })
        .collect();
    let rows: Vec<Row> = app
        .visible_processes()
        .iter()
        .enumerate()
        .map(|(index, process)| {
            let values = columns
                .iter()
                .map(|column| match column {
                    DisplayColumn::Pid => process.pid.clone(),
                    DisplayColumn::Name => process.name.clone(),
                    DisplayColumn::Cpu => format!("{:.1}%", process.cpu),
                    DisplayColumn::Memory => format!("{} MB", process.mem_mb),
                    DisplayColumn::Threads => process.threads.to_string(),
                    DisplayColumn::Status => process.status.clone(),
                    DisplayColumn::Command => {
                        if app.config.show_full_command {
                            process.command.clone()
                        } else {
                            let max = 24usize;
                            let summary = process.command.trim();
                            if summary.len() <= max {
                                summary.to_string()
                            } else {
                                format!("{}...", &summary[..max])
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();
            let style = if index == app.selected_process {
                Style::default()
                    .fg(palette.selection_fg)
                    .bg(palette.selection_bg)
            } else {
                Style::default()
            };
            Row::new(values).style(style)
        })
        .collect();
    let table_title = format!(" Processes  ({}) ", app.visible_processes().len());
    let table = Table::new(rows, constraints)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(table_title)
                .style(
                    Style::default().fg(if app.focused_block == FocusedBlock::ProcessTable {
                        palette.accent
                    } else {
                        palette.muted
                    }),
                ),
        )
        .column_spacing(1);
    f.render_widget(table, table_chunks[0]);

    let detail_title = if app.focused_block == FocusedBlock::ProcessDetails {
        " Process details "
    } else {
        " Details "
    };
    render_process_details(
        f,
        table_chunks[1],
        app.selected_process_info(),
        detail_title,
        app.focused_block == FocusedBlock::ProcessDetails,
        palette,
    );

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" focus  "),
        Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" select  "),
        Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" sort  "),
        Span::styled("x", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" kill  "),
        Span::styled("Space", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" pause  "),
        Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" refresh  "),
        Span::styled("T", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" theme  "),
        Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" help  "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit  "),
        Span::styled(
            format!(
                "{}  {}s ago  {}  {}ms  {}",
                app.status(),
                app.system_state.refreshed_at.elapsed().as_secs(),
                app.config.theme_name(),
                app.config.refresh_interval_ms,
                app.config.max_processes,
            ),
            Style::default().fg(palette.footer),
        ),
    ]));
    f.render_widget(footer, chunks[4]);

    if app.input_mode == InputMode::Help {
        render_help(f, palette);
    } else if app.input_mode == InputMode::ConfirmKill {
        render_confirm(f, app.selected_pid(), palette);
    }
}

fn render_info(
    f: &mut Frame,
    area: Rect,
    title: &str,
    value: String,
    text_color: ratatui::style::Color,
) {
    let widget = Paragraph::new(value)
        .style(Style::default().fg(text_color))
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, area);
}

fn render_process_details(
    f: &mut Frame,
    area: Rect,
    process: Option<&crate::app::ProcessItem>,
    title: &str,
    focused: bool,
    palette: crate::app::ThemePalette,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(if focused {
            palette.accent
        } else {
            palette.muted
        }));

    let content = if let Some(process) = process {
        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(process.name.clone()),
            ]),
            Line::from(vec![
                Span::styled("PID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(process.pid.clone()),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(process.status.clone()),
            ]),
            Line::from(vec![
                Span::styled("CPU: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:.1}%", process.cpu)),
            ]),
            Line::from(vec![
                Span::styled("Memory: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} MB", process.mem_mb)),
            ]),
            Line::from(vec![
                Span::styled("Threads: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(process.threads.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Parent: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(process.parent_pid.to_string()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(process.command.clone()),
            ]),
        ]
    } else {
        vec![
            Line::from("No process selected"),
            Line::from(""),
            Line::from("Use the table to choose a running task."),
        ]
    };

    let widget = Paragraph::new(content)
        .wrap(Wrap { trim: true })
        .block(block);
    f.render_widget(widget, area);
}

fn render_help(f: &mut Frame, palette: crate::app::ThemePalette) {
    let area = centered_rect(72, 70, f.size());
    f.render_widget(Clear, area);
    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            "Hyper Top Controls",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Tab / Shift-Tab   Move focus between panels"),
        Line::from("j / Down          Select the next process"),
        Line::from("k / Up            Select the previous process"),
        Line::from("/                 Search by process name or PID"),
        Line::from("s                 Cycle sort: CPU, memory, name, PID, threads"),
        Line::from("c / m / n / p / t Sort directly by CPU, memory, name, PID, threads"),
        Line::from("r                 Cycle refresh rate for telemetry"),
        Line::from("T                 Cycle the UI theme"),
        Line::from("[ / ]             Adjust tracked process display count"),
        Line::from("x                 Request termination of selected process"),
        Line::from("Space             Pause or resume telemetry updates"),
        Line::from("q                 Quit; Esc closes dialogs"),
    ])
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .style(Style::default().fg(palette.accent)),
    );
    f.render_widget(help, area);
}

fn render_confirm(f: &mut Frame, pid: Option<u32>, palette: crate::app::ThemePalette) {
    let area = centered_rect(60, 25, f.size());
    f.render_widget(Clear, area);
    let text = format!(
        "Terminate PID {}?\n\n[y] confirm    [n] cancel",
        pid.map_or("?".to_string(), |value| value.to_string())
    );
    let confirm = Paragraph::new(text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm action ")
                .style(Style::default().fg(palette.warning)),
        );
    f.render_widget(confirm, area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width) / 2),
            Constraint::Percentage(width),
            Constraint::Percentage((100 - width) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height) / 2),
            Constraint::Percentage(height),
            Constraint::Percentage((100 - height) / 2),
        ])
        .split(horizontal[1])[1]
}

fn sort_name(sort: SortMode) -> &'static str {
    match sort {
        SortMode::Cpu => "CPU",
        SortMode::Memory => "MEM",
        SortMode::Name => "NAME",
        SortMode::Pid => "PID",
        SortMode::Threads => "THREADS",
    }
}

fn format_duration(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    format!("{}d {:02}h {:02}m {:02}s", days, hours, minutes, seconds)
}
