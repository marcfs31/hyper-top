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
    let layout_constraints = if app.expanded_process_view {
        [
            Constraint::Length(3),
            Constraint::Length(0),
            Constraint::Length(0),
            Constraint::Length(0),
            Constraint::Min(8),
            Constraint::Length(2),
        ]
    } else {
        [
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(2),
        ]
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(layout_constraints)
        .split(f.size());

    let palette_title = match app.input_mode {
        InputMode::Search => format!(" Search: {}_ ", app.query),
        _ if app.query.is_empty() => " Command Palette  (/) filter ".to_string(),
        _ => format!(" Filter: {} ", app.query),
    };
    let compact_suffix = if app.config.compact_mode {
        "  COMPACT "
    } else {
        ""
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
    .wrap(Wrap { trim: true })
    .scroll((app.help_scroll.min(16).min(u16::MAX as usize) as u16, 0))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(palette_title)
            .style(Style::default().fg(border_color)),
    );
    f.render_widget(palette_widget, chunks[0]);

    let metrics = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);

    let cpu_percent = app.system_state.cpu_usage.clamp(0.0, 100.0);
    let cpu_chart = format_core_usage_chart(&app.system_state.core_usage);
    let cpu_label = format!(
        "{:.1}% used  {} cores  {} MHz  {}",
        cpu_percent, app.system_state.cpu_count, app.system_state.cpu_frequency_mhz, cpu_chart
    );
    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" CPU Total "))
        .gauge_style(Style::default().fg(palette.success))
        .label(cpu_label)
        .percent(cpu_percent as u16);
    if !app.expanded_process_view {
        f.render_widget(cpu_gauge, metrics[0]);
    }

    let ram_percent = if app.system_state.ram_total_gb > 0.0 {
        ((app.system_state.ram_used_gb / app.system_state.ram_total_gb) * 100.0).clamp(0.0, 100.0)
            as u16
    } else {
        0
    };
    let ram_label = if app.system_state.swap_total_gb > 0.0 {
        format!(
            "RAM {:.2}/{:.2} GB {}  SWAP {:.2}/{:.2} GB",
            app.system_state.ram_used_gb,
            app.system_state.ram_total_gb,
            format_usage_bar(ram_percent, 10),
            app.system_state.swap_used_gb,
            app.system_state.swap_total_gb
        )
    } else {
        format!(
            "RAM {:.2}/{:.2} GB {}",
            app.system_state.ram_used_gb,
            app.system_state.ram_total_gb,
            format_usage_bar(ram_percent, 10)
        )
    };
    let ram_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory "))
        .gauge_style(Style::default().fg(palette.memory))
        .label(ram_label)
        .percent(ram_percent);
    if !app.expanded_process_view {
        f.render_widget(ram_gauge, metrics[1]);
    }

    let storage_percent = app.system_state.storage_percent();
    let storage_label = if app.system_state.storage_total_gb > 0.0 {
        format!(
            "{} {}  {:.1}% used  {:.2} GB free of {:.2} GB",
            app.system_state.storage_mount,
            format_usage_bar(storage_percent, 10),
            storage_percent as f64,
            app.system_state.storage_available_gb,
            app.system_state.storage_total_gb
        )
    } else {
        "No storage data".to_string()
    };
    let storage_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Storage "))
        .gauge_style(Style::default().fg(palette.warning))
        .label(storage_label)
        .percent(storage_percent);
    if !app.expanded_process_view {
        f.render_widget(storage_gauge, metrics[2]);
    }

    let chart_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[2]);
    let load_chart = Paragraph::new(vec![
        Line::from(format!(
            "1m {:.2}  5m {:.2} load",
            app.system_state.load_average[0], app.system_state.load_average[1]
        )),
        Line::from(format!("15m {:.2} load", app.system_state.load_average[2])),
        Line::from(format!(
            "avg {:.2} / core",
            app.system_state.load_average.iter().sum::<f64>()
                / 3.0
                / app.system_state.cpu_count.max(1) as f64
        )),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).title(" Load "))
    .style(Style::default().fg(palette.text));
    if !app.expanded_process_view {
        f.render_widget(load_chart, chart_chunks[0]);
    }

    let core_chart = Paragraph::new(vec![
        Line::from(format!(
            "Total {:.1}% {}",
            cpu_percent,
            format_usage_bar(cpu_percent as u16, 8)
        )),
        Line::from(format!(
            "{} logical cores",
            app.system_state.core_usage.len()
        )),
        Line::from(format_core_usage_chart(&app.system_state.core_usage).to_string()),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).title(" Cores "))
    .style(Style::default().fg(palette.text));
    if !app.expanded_process_view {
        f.render_widget(core_chart, chart_chunks[1]);
    }

    let ram_chart = Paragraph::new(vec![
        Line::from(format!(
            "RAM {:>3}% {}",
            ram_percent,
            format_usage_bar(ram_percent, 8)
        )),
        Line::from(format!(
            "{:.2} / {:.2} GB used",
            app.system_state.ram_used_gb, app.system_state.ram_total_gb
        )),
        Line::from(format!(
            "Swap {:.2} / {:.2} GB ({:.1}%)",
            app.system_state.swap_used_gb,
            app.system_state.swap_total_gb,
            usage_percent(
                app.system_state.swap_used_gb,
                app.system_state.swap_total_gb
            )
        )),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).title(" Memory "))
    .style(Style::default().fg(palette.text));
    if !app.expanded_process_view {
        f.render_widget(ram_chart, chart_chunks[2]);
    }

    let storage_chart = Paragraph::new(vec![
        Line::from(format!(
            "{} {:>3}% {}",
            app.system_state.storage_mount,
            storage_percent,
            format_usage_bar(storage_percent, 10)
        )),
        Line::from(format!(
            "{:.2} / {:.2} GB used",
            app.system_state.storage_used_gb, app.system_state.storage_total_gb
        )),
        Line::from(format!(
            "{} mounted volume(s)",
            app.system_state.storage_mounts.len()
        )),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).title(" Storage "))
    .style(Style::default().fg(palette.text));
    if !app.expanded_process_view {
        f.render_widget(storage_chart, chart_chunks[3]);
    }

    let info = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[3]);
    if !app.expanded_process_view {
        render_info(
            f,
            info[0],
            " Uptime ",
            format!(
                "{} ({})",
                format_duration(app.system_state.uptime),
                format_uptime_short(app.system_state.uptime)
            ),
            palette.text,
        );
        render_info(
            f,
            info[1],
            " Load Average ",
            format!(
                "1m {:.2}  5m {:.2}  15m {:.2} load{}",
                app.system_state.load_average[0],
                app.system_state.load_average[1],
                app.system_state.load_average[2],
                if app.system_state.cpu_count == 0 {
                    String::new()
                } else {
                    format!(
                        "  ({:.2} avg/core)",
                        app.system_state.load_average.iter().sum::<f64>()
                            / 3.0
                            / app.system_state.cpu_count as f64
                    )
                }
            ),
            palette.text,
        );
        render_info(
            f,
            info[2],
            " Overview ",
            format!(
                "{}  {}  {} total processes  {} total threads  {} visible  {}{}",
                sort_name(app.sort_mode),
                if app.paused {
                    "PAUSED (all telemetry)"
                } else {
                    "LIVE"
                },
                app.system_state.running_processes,
                app.system_state.total_threads,
                app.visible_processes().len(),
                app.focus_label(),
                if app.query.is_empty() {
                    ""
                } else {
                    "  FILTERED"
                }
            ),
            palette.text,
        );
        render_info(
            f,
            info[3],
            " Storage ",
            if app.system_state.storage_total_gb > 0.0 {
                format!(
                    "{} {}  {:.1}% used  {:.2} GB free",
                    app.system_state.storage_mount,
                    if app.system_state.storage_mount == "/" {
                        "root"
                    } else {
                        "disk"
                    },
                    app.system_state.storage_percent() as f64,
                    app.system_state.storage_available_gb
                )
            } else {
                "No storage data".to_string()
            },
            palette.text,
        );
    }

    let table_chunks = if app.config.compact_mode {
        vec![chunks[4]]
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(chunks[4])
            .to_vec()
    };

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
            DisplayColumn::User => Constraint::Length(12),
            DisplayColumn::Uid => Constraint::Length(8),
            DisplayColumn::Cpu => Constraint::Percentage(15),
            DisplayColumn::Memory => Constraint::Percentage(15),
            DisplayColumn::Threads => Constraint::Length(8),
            DisplayColumn::Status => Constraint::Length(10),
            DisplayColumn::Command => Constraint::Percentage(35),
            DisplayColumn::Parent => Constraint::Length(8),
            DisplayColumn::ParentUser => Constraint::Length(12),
            DisplayColumn::ParentUid => Constraint::Length(8),
            DisplayColumn::Runtime => Constraint::Length(14),
        })
        .collect();
    let total_memory_mb = (app.system_state.ram_total_gb * 1024.0) as u64;
    let rows: Vec<Row> = app
        .visible_processes()
        .iter()
        .enumerate()
        .map(|(index, process)| {
            let values = columns
                .iter()
                .map(|column| match column {
                    DisplayColumn::Pid => process.pid.clone(),
                    DisplayColumn::Name => format!("{}{}", app.tree_prefix(process), process.name),
                    DisplayColumn::User => process.user.clone(),
                    DisplayColumn::Uid => process.uid.clone(),
                    DisplayColumn::Cpu => format!("{:.1}%", process.cpu),
                    DisplayColumn::Memory => {
                        let memory_percent = if total_memory_mb == 0 {
                            0.0
                        } else {
                            process.mem_mb as f64 / total_memory_mb as f64 * 100.0
                        };
                        format!("{} MB ({:.1}%)", process.mem_mb, memory_percent)
                    }
                    DisplayColumn::Threads => process.threads.to_string(),
                    DisplayColumn::Status => process.status.clone(),
                    DisplayColumn::Parent => process.parent_pid.to_string(),
                    DisplayColumn::ParentUser => process.parent_user.clone(),
                    DisplayColumn::ParentUid => process.parent_uid.clone(),
                    DisplayColumn::Runtime => process.runtime_label(),
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
    let limit_label = if app.config.max_processes == 0 {
        "unlimited".to_string()
    } else {
        format!("limit:{}", app.config.max_processes)
    };
    let table_title = format!(
        " Processes ({})  {}{}{} sort:{}  {} ",
        app.visible_processes().len(),
        limit_label,
        if app.expanded_process_view {
            "  EXPANDED"
        } else {
            ""
        },
        if app.tree_view { "  TREE" } else { "" },
        sort_name(app.sort_mode),
        if app.query.is_empty() {
            "all"
        } else {
            "filtered"
        }
    );
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

    if !app.config.compact_mode {
        let detail_title = if app.focused_block == FocusedBlock::ProcessDetails {
            " Process details "
        } else {
            " Details "
        };
        let storage_mounts = app.system_state.storage_mounts_summary();
        render_process_details(
            f,
            table_chunks[1],
            app.selected_process_info(),
            &storage_mounts,
            (app.system_state.ram_total_gb * 1024.0) as u64,
            detail_title,
            app.focused_block == FocusedBlock::ProcessDetails,
            palette,
            &app.system_state,
        );
    }

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" focus  "),
        Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" select  "),
        Span::styled("PgUp/PgDn", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" page  "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" focus/unfocus  "),
        Span::styled("v/V", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(if app.expanded_process_view {
            " expanded view  "
        } else {
            " normal view  "
        }),
        Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(if app.tree_view {
            " process tree  "
        } else {
            " flat process list  "
        }),
        Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" sort / 0 clear  "),
        Span::styled("x", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" kill  "),
        Span::styled(
            "Esc/Delete/f",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" clear filter  "),
        Span::styled("Space", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" pause all telemetry  "),
        Span::styled("z", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" compact  "),
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
                "{}  {}s ago  {}  refresh:{}ms  limit:{} (u cycle){}",
                app.status(),
                app.system_state.refreshed_at.elapsed().as_secs(),
                app.config.theme_name(),
                app.config.refresh_interval_ms,
                if app.config.max_processes == 0 {
                    "unlimited".to_string()
                } else {
                    app.config.max_processes.to_string()
                },
                compact_suffix,
            ),
            Style::default().fg(palette.footer),
        ),
    ]))
    .wrap(Wrap { trim: true });
    f.render_widget(footer, chunks[5]);

    if app.input_mode == InputMode::Help {
        render_help(f, app);
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
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, area);
}

#[allow(clippy::too_many_arguments)]
fn render_process_details(
    f: &mut Frame,
    area: Rect,
    process: Option<&crate::app::ProcessItem>,
    storage_mounts: &[crate::app::StorageMount],
    total_memory_mb: u64,
    title: &str,
    focused: bool,
    palette: crate::app::ThemePalette,
    system: &crate::app::SystemState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(if focused {
            palette.accent
        } else {
            palette.muted
        }));

    let mut content = if let Some(process) = process {
        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(process.name.clone()),
            ]),
            Line::from(vec![
                Span::styled("User: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} (UID {})", process.user, process.uid)),
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
                Span::raw(format!(
                    "{} MB ({:.1}% of RAM)",
                    process.mem_mb,
                    if total_memory_mb == 0 {
                        0.0
                    } else {
                        process.mem_mb as f64 / total_memory_mb as f64 * 100.0
                    }
                )),
            ]),
            Line::from(vec![
                Span::styled("Threads: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(process.threads.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Parent: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "{}  {} (UID {})",
                    process.parent_pid, process.parent_user, process.parent_uid
                )),
            ]),
            Line::from(vec![
                Span::styled("Runtime: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(process.runtime_label()),
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

    if !storage_mounts.is_empty() {
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled("Storage: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("mounted volumes"),
        ]));
        for mount in storage_mounts.iter().take(4) {
            let percent = mount.percent_used();
            content.push(Line::from(format!(
                "  {} {} {}% used ({:.2} GB free / {:.2} GB total)",
                mount.mount,
                format_usage_bar(percent, 12),
                percent,
                mount.available_gb,
                mount.total_gb
            )));
        }
        content.push(Line::from(""));
        content.push(Line::from(format!(
            "System: CPU {:.1}% | RAM {:.1}% | load {:.2}",
            system.cpu_usage,
            usage_percent(system.ram_used_gb, system.ram_total_gb),
            system.load_average[0]
        )));
        content.push(Line::from(format!(
            "Uptime: {} | Processes: {} | Threads: {}",
            format_duration(system.uptime),
            system.running_processes,
            system.total_threads
        )));
    }

    let widget = Paragraph::new(content)
        .wrap(Wrap { trim: true })
        .block(block);
    f.render_widget(widget, area);
}

fn format_usage_bar(percent: u16, width: usize) -> String {
    let filled = ((percent as usize).clamp(0, 100) * width) / 100;
    let bar = "█".repeat(filled).to_string();
    let blank = "░".repeat(width.saturating_sub(filled));
    format!("[{}{}]", bar, blank)
}

fn format_core_usage_chart(values: &[f32]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let symbols = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let mut chart = String::new();
    for value in values.iter().take(8) {
        let normalized = (value.clamp(0.0, 100.0) / 100.0 * (symbols.len() as f32 - 1.0)) as usize;
        chart.push_str(symbols[normalized]);
    }
    format!("cores:{}", chart)
}

fn render_help(f: &mut Frame, app: &App) {
    let palette = app.config.theme.palette();
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
        Line::from("Delete / Esc      Clear the active filter quickly"),
        Line::from("s                 Cycle sort: CPU, memory, name, PID, threads, none"),
        Line::from("0                 Clear sorting (restore telemetry order)"),
        Line::from("c / m / n / p / t Sort directly by CPU, memory, name, PID, threads"),
        Line::from("r                 Cycle refresh rate (250/500/900/1500/2500 ms)"),
        Line::from("a                 Reset refresh rate to automatic default (900 ms)"),
        Line::from("z                 Toggle compact mode"),
        Line::from("T                 Cycle the UI theme"),
        Line::from("[ / ], -/+, or { }  Decrease/increase process limit by 10"),
        Line::from("u                 Cycle limit: unlimited / 50 / 80 / 150"),
        Line::from("v / V             Toggle full-screen Processes + Details telemetry view"),
        Line::from("e                 Toggle hierarchical process tree view"),
        Line::from("Home / End         Jump to first / last process"),
        Line::from("PageUp / PageDown   Move by one page of processes"),
        Line::from("1-6               Apply quick filter presets"),
        Line::from("7 / 8 / 9         Toggle PPID / runtime / status columns"),
        Line::from("Enter             Focus/unfocus selected PID while all telemetry refreshes"),
        Line::from("x                 Request termination of selected process"),
        Line::from("Space             Pause/resume all telemetry updates (not one process)"),
        Line::from("q                 Quit; Esc closes dialogs"),
    ])
    .scroll((app.help_scroll.min(u16::MAX as usize) as u16, 0))
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help - Up/Down scroll, PgUp/PgDn page, Home/End jump, ?/Esc close ")
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
        SortMode::None => "NONE",
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

fn format_uptime_short(duration: std::time::Duration) -> String {
    let hours = duration.as_secs() / 3_600;
    format!("{hours} h total")
}

fn usage_percent(used: f64, total: f64) -> f64 {
    if total <= 0.0 {
        0.0
    } else {
        (used / total * 100.0).clamp(0.0, 100.0)
    }
}
