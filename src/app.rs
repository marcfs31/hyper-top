use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FocusedBlock {
    CommandPalette,
    CpuRam,
    ProcessTable,
    ProcessDetails,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Cpu,
    Memory,
    Name,
    Pid,
    Threads,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    ConfirmKill,
    Help,
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
}

impl App {
    pub fn new() -> Self {
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

    pub fn status(&self) -> &str {
        &self.system_state.message
    }
}

#[cfg(test)]
mod tests {
    use super::{App, ProcessItem, SortMode};

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
    fn filters_processes_by_name_pid_and_command() {
        let mut app = App::new();
        app.system_state.processes = vec![
            process("12", "terminal", 2.0, 100, 4),
            process("34", "database", 1.0, 200, 8),
        ];
        app.system_state.processes[1].command = "postgres --cluster main".to_string();

        app.query = "term".to_string();
        assert_eq!(app.visible_processes().len(), 1);
        assert_eq!(app.visible_processes()[0].pid, "12");

        app.query = "postgres".to_string();
        assert_eq!(app.visible_processes()[0].name, "database");

        app.query = "34".to_string();
        assert_eq!(app.visible_processes()[0].pid, "34");
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
}
