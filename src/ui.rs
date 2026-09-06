use crate::app::{App, FocusedBlock, InputMode, SortMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Row, Table, Wrap},
    Frame,
};

const ACCENT: Color = Color::Cyan;
const MUTED: Color = Color::DarkGray;

pub fn draw(f: &mut Frame, app: &App) {
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
    let palette_border = if app.input_mode == InputMode::Search
        || app.focused_block == FocusedBlock::CommandPalette
    {
        ACCENT
    } else {
        MUTED
    };
    let palette = Paragraph::new(Line::from(vec![
        Span::styled(" > ", Style::default().fg(ACCENT)),
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
            .style(Style::default().fg(palette_border)),
    );
    f.render_widget(palette, chunks[0]);

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
        .gauge_style(Style::default().fg(Color::LightGreen))
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
        .gauge_style(Style::default().fg(Color::LightMagenta))
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
    );

    let table_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[3]);

    let rows: Vec<Row> = app
        .visible_processes()
        .iter()
        .enumerate()
        .map(|(index, process)| {
            let style = if index == app.selected_process {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
            };
            Row::new(vec![
                process.pid.clone(),
                process.name.clone(),
                format!("{:.1}%", process.cpu),
                format!("{} MB", process.mem_mb),
            ])
            .style(style)
        })
        .collect();
    let table_title = format!(" Processes  ({}) ", app.visible_processes().len());
    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["PID", "NAME", "CPU", "MEMORY"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(table_title)
            .style(
                Style::default().fg(if app.focused_block == FocusedBlock::ProcessTable {
                    ACCENT
                } else {
                    MUTED
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
        Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" help  "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit  "),
        Span::styled(
            format!(
                "{}  {}s ago",
                app.status(),
                app.system_state.refreshed_at.elapsed().as_secs()
            ),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    f.render_widget(footer, chunks[4]);

    if app.input_mode == InputMode::Help {
        render_help(f);
    } else if app.input_mode == InputMode::ConfirmKill {
        render_confirm(f, app.selected_pid());
    }
}

fn render_info(f: &mut Frame, area: Rect, title: &str, value: String) {
    let widget = Paragraph::new(value)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, area);
}

fn render_process_details(
    f: &mut Frame,
    area: Rect,
    process: Option<&crate::app::ProcessItem>,
    title: &str,
    focused: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(if focused { ACCENT } else { MUTED }));

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

fn render_help(f: &mut Frame) {
    let area = centered_rect(72, 70, f.size());
    f.render_widget(Clear, area);
    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            "Hyper Top Controls",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Tab / Shift-Tab   Move focus between panels"),
        Line::from("j / Down          Select the next process"),
        Line::from("k / Up            Select the previous process"),
        Line::from("/                 Search by process name or PID"),
        Line::from("s                 Cycle sort: CPU, memory, name, PID, threads"),
        Line::from("c / m / n / p / t Sort directly by CPU, memory, name, PID, threads"),
        Line::from("x                 Request termination of selected process"),
        Line::from("Space             Pause or resume telemetry updates"),
        Line::from("q                 Quit; Esc closes dialogs"),
    ])
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .style(Style::default().fg(ACCENT)),
    );
    f.render_widget(help, area);
}

fn render_confirm(f: &mut Frame, pid: Option<u32>) {
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
                .style(Style::default().fg(Color::Yellow)),
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
