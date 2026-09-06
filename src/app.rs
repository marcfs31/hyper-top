use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FocusedBlock {
    CommandPalette,
    CpuRam,
    ProcessTable,
    ProcessDetails,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortMode {
    Cpu,
    Memory,
    Name,
    Pid,
    Threads,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputMode {
    Normal,
    Search,
    ConfirmKill,
    Help,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Default,
    Solarized,
    Midnight,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AppConfig {
    pub refresh_interval_ms: u64,
    pub max_processes: usize,
    pub theme: Theme,
    pub show_full_command: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_interval_ms: 900,
            max_processes: 80,
            theme: Theme::Default,
            show_full_command: true,
        }
    }
}

impl AppConfig {
    pub fn theme_name(&self) -> &'static str {
        match self.theme {
            Theme::Default => "DEFAULT",
            Theme::Solarized => "SOLARIZED",
            Theme::Midnight => "MIDNIGHT",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ProcessItem {
    pub pid: String,
    pub name: String,
    pub command: String,
    pub cpu: f32,
    pub mem_mb: u64,
    pub status: String,
    pub threads: usize,
    pub parent_pid: u32,
}

pub struct SystemState {
    pub cpu_usage: f32,
    pub cpu_count: usize,
    pub cpu_frequency_mhz: u64,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub swap_used_gb: f64,
    pub swap_total_gb: f64,
    pub uptime: Duration,
    pub load_average: [f64; 3],
    pub running_processes: usize,
    pub total_threads: usize,
    pub processes: Vec<ProcessItem>,
    pub refreshed_at: Instant,
    pub message: String,
}

pub struct App {
    pub focused_block: FocusedBlock,
    pub system_state: SystemState,
    pub is_running: bool,
    pub input_mode: InputMode,
    pub query: String,
    pub sort_mode: SortMode,
    pub selected_process: usize,
    pub paused: bool,
    pub config: AppConfig,
}

impl App {
    pub fn new() -> Self {
        Self::with_config(AppConfig::default())
    }

    pub fn with_config(config: AppConfig) -> Self {
        Self {
            focused_block: FocusedBlock::ProcessTable,
            system_state: SystemState {
                cpu_usage: 0.0,
                cpu_count: 0,
                cpu_frequency_mhz: 0,
                ram_used_gb: 0.0,
                ram_total_gb: 0.0,
                swap_used_gb: 0.0,
                swap_total_gb: 0.0,
                uptime: Duration::ZERO,
                load_average: [0.0; 3],
                running_processes: 0,
                total_threads: 0,
                processes: Vec::new(),
                refreshed_at: Instant::now(),
                message: "Starting telemetry...".to_string(),
            },
            is_running: true,
            input_mode: InputMode::Normal,
            query: String::new(),
            sort_mode: SortMode::Cpu,
            selected_process: 0,
            paused: false,
            config,
        }
    }

    pub fn filtered_processes(&self) -> Vec<&ProcessItem> {
        let query = self.query.to_lowercase();
        self.system_state
            .processes
            .iter()
            .filter(|process| {
                query.is_empty()
                    || process.name.to_lowercase().contains(&query)
                    || process.command.to_lowercase().contains(&query)
                    || process.pid.contains(&query)
            })
            .collect()
    }

    pub fn visible_processes(&self) -> Vec<&ProcessItem> {
        let mut processes = self.filtered_processes();
        match self.sort_mode {
            SortMode::Cpu => processes.sort_by(|a, b| {
                b.cpu
                    .partial_cmp(&a.cpu)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortMode::Memory => processes.sort_by_key(|process| std::cmp::Reverse(process.mem_mb)),
            SortMode::Name => processes.sort_by_key(|process| process.name.to_lowercase()),
            SortMode::Pid => {
                processes.sort_by_key(|process| process.pid.parse::<u32>().unwrap_or_default())
            }
            SortMode::Threads => {
                processes.sort_by_key(|process| std::cmp::Reverse(process.threads))
            }
        }
        processes.truncate(self.config.max_processes.max(1));
        processes
    }

    pub fn selected_process_info(&self) -> Option<&ProcessItem> {
        self.visible_processes().get(self.selected_process).copied()
    }

    pub fn selected_pid(&self) -> Option<u32> {
        self.visible_processes()
            .get(self.selected_process)
            .and_then(|process| process.pid.parse().ok())
    }

    pub fn move_selection(&mut self, delta: i32) {
        let count = self.filtered_processes().len();
        if count == 0 {
            self.selected_process = 0;
            return;
        }
        let next = self.selected_process as i32 + delta;
        self.selected_process = next.clamp(0, count as i32 - 1) as usize;
    }

    pub fn cycle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Cpu => SortMode::Memory,
            SortMode::Memory => SortMode::Name,
            SortMode::Name => SortMode::Pid,
            SortMode::Pid => SortMode::Threads,
            SortMode::Threads => SortMode::Cpu,
        };
        self.selected_process = 0;
    }

    pub fn toggle_focus(&mut self) {
        self.focused_block = match self.focused_block {
            FocusedBlock::CommandPalette => FocusedBlock::CpuRam,
            FocusedBlock::CpuRam => FocusedBlock::ProcessTable,
            FocusedBlock::ProcessTable => FocusedBlock::ProcessDetails,
            FocusedBlock::ProcessDetails => FocusedBlock::CommandPalette,
        };
    }

    pub fn cycle_refresh_interval(&mut self) {
        let next = match self.config.refresh_interval_ms {
            250 => 500,
            500 => 900,
            900 => 1500,
            1500 => 2500,
            _ => 250,
        };
        self.config.refresh_interval_ms = next;
    }

    pub fn adjust_process_limit(&mut self, delta: i32) {
        let next = self.config.max_processes as i32 + delta;
        self.config.max_processes = next.max(10).min(250) as usize;
    }

    pub fn cycle_theme(&mut self) {
        self.config.theme = match self.config.theme {
            Theme::Default => Theme::Solarized,
            Theme::Solarized => Theme::Midnight,
            Theme::Midnight => Theme::Default,
        };
    }

    pub fn status(&self) -> &str {
        &self.system_state.message
    }
}

#[cfg(test)]
mod tests {
    use super::{App, AppConfig, FocusedBlock, ProcessItem, SortMode, Theme};

    fn process(pid: &str, name: &str, cpu: f32, mem_mb: u64, threads: usize) -> ProcessItem {
        ProcessItem {
            pid: pid.to_string(),
            name: name.to_string(),
            command: name.to_string(),
            cpu,
            mem_mb,
            status: "Run".to_string(),
            threads,
            parent_pid: 1,
        }
    }

    #[test]
    fn filters_processes_by_name_pid_and_command_case_insensitively() {
        let mut app = App::new();
        app.system_state.processes = vec![
            process("12", "terminal", 2.0, 100, 4),
            process("34", "database", 1.0, 200, 8),
        ];
        app.system_state.processes[1].command = "Postgres --cluster main".to_string();

        app.query = "TERM".to_string();
        assert_eq!(app.visible_processes().len(), 1);
        assert_eq!(app.visible_processes()[0].pid, "12");

        app.query = "postgres".to_string();
        assert_eq!(app.visible_processes()[0].name, "database");

        app.query = "34".to_string();
        assert_eq!(app.visible_processes()[0].pid, "34");

        app.query.clear();
        assert_eq!(app.visible_processes().len(), 2);
    }

    #[test]
    fn selection_moves_within_bounds_across_list() {
        let mut app = App::new();
        app.system_state.processes = vec![
            process("10", "alpha", 20.0, 200, 4),
            process("20", "beta", 15.0, 400, 13),
            process("30", "gamma", 10.0, 800, 9),
        ];

        app.selected_process = 0;
        app.move_selection(1);
        assert_eq!(app.selected_process, 1);

        app.move_selection(10);
        assert_eq!(app.selected_process, 2);

        app.move_selection(-99);
        assert_eq!(app.selected_process, 0);

        app.selected_process = 5;
        app.move_selection(1);
        assert_eq!(app.selected_process, 2);
    }

    #[test]
    fn cycle_sort_advances_modes_and_resets_selection() {
        let mut app = App::new();
        app.sort_mode = SortMode::Cpu;
        app.selected_process = 2;

        app.cycle_sort();
        assert_eq!(app.sort_mode, SortMode::Memory);
        assert_eq!(app.selected_process, 0);

        app.cycle_sort();
        assert_eq!(app.sort_mode, SortMode::Name);

        app.cycle_sort();
        assert_eq!(app.sort_mode, SortMode::Pid);

        app.cycle_sort();
        assert_eq!(app.sort_mode, SortMode::Threads);

        app.cycle_sort();
        assert_eq!(app.sort_mode, SortMode::Cpu);
    }

    #[test]
    fn toggle_focus_cycles_through_blocks() {
        let mut app = App::new();

        assert_eq!(app.focused_block, FocusedBlock::ProcessTable);

        app.toggle_focus();
        assert_eq!(app.focused_block, FocusedBlock::ProcessDetails);

        app.toggle_focus();
        assert_eq!(app.focused_block, FocusedBlock::CommandPalette);

        app.toggle_focus();
        assert_eq!(app.focused_block, FocusedBlock::CpuRam);

        app.toggle_focus();
        assert_eq!(app.focused_block, FocusedBlock::ProcessTable);
    }

    #[test]
    fn sorts_by_memory_and_threads_and_keeps_selection_in_view() {
        let mut app = App::new();
        app.system_state.processes = vec![
            process("12", "small", 90.0, 100, 2),
            process("34", "large", 1.0, 900, 32),
        ];
        app.sort_mode = SortMode::Memory;
        app.selected_process = 0;

        assert_eq!(app.visible_processes()[0].name, "large");
        assert_eq!(app.selected_pid(), Some(34));

        app.sort_mode = SortMode::Threads;
        app.selected_process = 0;
        assert_eq!(app.visible_processes()[0].name, "large");
    }

    #[test]
    fn custom_config_controls_refresh_and_process_limit() {
        let mut app = App::with_config(AppConfig {
            refresh_interval_ms: 500,
            max_processes: 12,
            theme: Theme::Solarized,
            show_full_command: false,
        });

        assert_eq!(app.config.refresh_interval_ms, 500);
        assert_eq!(app.config.max_processes, 12);
        assert_eq!(app.config.theme, Theme::Solarized);

        app.cycle_refresh_interval();
        assert_eq!(app.config.refresh_interval_ms, 900);

        app.adjust_process_limit(20);
        assert_eq!(app.config.max_processes, 32);

        app.cycle_theme();
        assert_eq!(app.config.theme, Theme::Midnight);
    }

    #[test]
    fn visible_process_count_uses_config_limit() {
        let mut app = App::new();
        app.config.max_processes = 2;
        app.system_state.processes = vec![
            process("10", "first", 90.0, 1, 1),
            process("20", "second", 80.0, 2, 2),
            process("30", "third", 70.0, 3, 3),
        ];

        assert_eq!(app.visible_processes().len(), 2);
    }

    #[test]
    fn sorts_by_cpu_name_and_pid() {
        let mut app = App::new();
        app.system_state.processes = vec![
            process("20", "zebra", 30.0, 1000, 3),
            process("10", "alpha", 80.0, 150, 7),
            process("30", "beta", 50.0, 400, 10),
        ];

        app.sort_mode = SortMode::Cpu;
        let cpu_order: Vec<_> = app
            .visible_processes()
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(cpu_order, vec!["alpha", "beta", "zebra"]);

        app.sort_mode = SortMode::Name;
        let name_order: Vec<_> = app
            .visible_processes()
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(name_order, vec!["alpha", "beta", "zebra"]);

        app.sort_mode = SortMode::Pid;
        let pid_order: Vec<_> = app
            .visible_processes()
            .iter()
            .map(|p| p.pid.as_str())
            .collect();
        assert_eq!(pid_order, vec!["10", "20", "30"]);
    }

    #[test]
    fn selected_process_info_exposes_full_metadata() {
        let mut app = App::new();
        app.system_state.processes = vec![ProcessItem {
            pid: "4301".to_string(),
            name: "gnome-shell".to_string(),
            command: "gnome-shell --replace".to_string(),
            cpu: 12.5,
            mem_mb: 2048,
            status: "run".to_string(),
            threads: 32,
            parent_pid: 1,
        }];
        app.selected_process = 0;

        let info = app.selected_process_info().unwrap();
        assert_eq!(info.command, "gnome-shell --replace");
        assert_eq!(info.status, "run");
        assert_eq!(info.threads, 32);
        assert_eq!(info.parent_pid, 1);
        assert_eq!(app.selected_pid(), Some(4301));
    }
}
