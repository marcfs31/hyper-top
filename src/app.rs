use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FocusedBlock {
    CommandPalette,
    CpuRam,
    ProcessTable,
    ProcessDetails,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortMode {
    Cpu,
    Memory,
    Name,
    Pid,
    Threads,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum InputMode {
    Normal,
    Search,
    ConfirmKill,
    Help,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Default,
    Solarized,
    Midnight,
}

impl Theme {
    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::Solarized => "SOLARIZED",
            Self::Midnight => "MIDNIGHT",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "default" => Some(Self::Default),
            "solarized" => Some(Self::Solarized),
            "midnight" => Some(Self::Midnight),
            _ => None,
        }
    }

    pub fn palette(self) -> ThemePalette {
        match self {
            Self::Default => ThemePalette {
                accent: Color::Cyan,
                muted: Color::DarkGray,
                success: Color::LightGreen,
                warning: Color::Yellow,
                memory: Color::LightMagenta,
                text: Color::White,
                selection_fg: Color::Black,
                selection_bg: Color::Cyan,
                footer: Color::Yellow,
            },
            Self::Solarized => ThemePalette {
                accent: Color::LightBlue,
                muted: Color::DarkGray,
                success: Color::Green,
                warning: Color::Yellow,
                memory: Color::Magenta,
                text: Color::White,
                selection_fg: Color::Black,
                selection_bg: Color::LightBlue,
                footer: Color::LightYellow,
            },
            Self::Midnight => ThemePalette {
                accent: Color::LightCyan,
                muted: Color::Gray,
                success: Color::Green,
                warning: Color::LightRed,
                memory: Color::Magenta,
                text: Color::White,
                selection_fg: Color::Black,
                selection_bg: Color::LightCyan,
                footer: Color::LightRed,
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ThemePalette {
    pub accent: Color,
    pub muted: Color,
    pub success: Color,
    pub warning: Color,
    pub memory: Color,
    pub text: Color,
    pub selection_fg: Color,
    pub selection_bg: Color,
    pub footer: Color,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayColumn {
    Pid,
    Name,
    Cpu,
    Memory,
    Threads,
    Status,
    Command,
}

impl DisplayColumn {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pid => "PID",
            Self::Name => "NAME",
            Self::Cpu => "CPU",
            Self::Memory => "MEMORY",
            Self::Threads => "THREADS",
            Self::Status => "STATUS",
            Self::Command => "COMMAND",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "pid" => Some(Self::Pid),
            "name" => Some(Self::Name),
            "cpu" => Some(Self::Cpu),
            "memory" => Some(Self::Memory),
            "threads" => Some(Self::Threads),
            "status" => Some(Self::Status),
            "command" => Some(Self::Command),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct FilterPreset {
    pub name: String,
    pub query: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SortProfile {
    pub name: String,
    pub sort_mode: SortMode,
}
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub warning_cpu: f32,
    pub critical_cpu: f32,
    pub warning_memory_pct: f32,
    pub critical_memory_pct: f32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            warning_cpu: 70.0,
            critical_cpu: 90.0,
            warning_memory_pct: 70.0,
            critical_memory_pct: 90.0,
        }
    }
}

impl AlertThresholds {
    pub fn clamp(&self) -> Self {
        let warning_cpu = self.warning_cpu.clamp(10.0, 95.0);
        let mut critical_cpu = self.critical_cpu.clamp(20.0, 99.0);
        let warning_memory_pct = self.warning_memory_pct.clamp(10.0, 95.0);
        let mut critical_memory_pct = self.critical_memory_pct.clamp(20.0, 99.0);

        if critical_cpu < warning_cpu {
            critical_cpu = warning_cpu;
        }
        if critical_memory_pct < warning_memory_pct {
            critical_memory_pct = warning_memory_pct;
        }

        Self {
            warning_cpu,
            critical_cpu,
            warning_memory_pct,
            critical_memory_pct,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessSeverity {
    Normal,
    Warning,
    Critical,
}

impl ProcessSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub refresh_interval_ms: u64,
    pub max_processes: usize,
    pub theme: Theme,
    pub show_full_command: bool,
    pub compact_mode: bool,
    #[serde(default)]
    pub alert_thresholds: AlertThresholds,
    #[serde(default)]
    pub visible_columns: Vec<DisplayColumn>,
    #[serde(default)]
    pub filter_presets: Vec<FilterPreset>,
    #[serde(default)]
    pub sort_profiles: Vec<SortProfile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_interval_ms: 900,
            max_processes: 80,
            theme: Theme::Default,
            show_full_command: true,
            compact_mode: false,
            alert_thresholds: AlertThresholds::default(),
            visible_columns: Self::default_visible_columns(),
            filter_presets: Vec::new(),
            sort_profiles: Self::default_sort_profiles(),
        }
    }
}

impl AppConfig {
    pub fn default_visible_columns() -> Vec<DisplayColumn> {
        vec![
            DisplayColumn::Pid,
            DisplayColumn::Name,
            DisplayColumn::Cpu,
            DisplayColumn::Memory,
        ]
    }

    pub fn default_sort_profiles() -> Vec<SortProfile> {
        vec![
            SortProfile {
                name: "CPU".to_string(),
                sort_mode: SortMode::Cpu,
            },
            SortProfile {
                name: "Memory".to_string(),
                sort_mode: SortMode::Memory,
            },
            SortProfile {
                name: "Name".to_string(),
                sort_mode: SortMode::Name,
            },
            SortProfile {
                name: "PID".to_string(),
                sort_mode: SortMode::Pid,
            },
            SortProfile {
                name: "Threads".to_string(),
                sort_mode: SortMode::Threads,
            },
        ]
    }

    fn apply_defaults(&mut self) {
        if self.visible_columns.is_empty() {
            self.visible_columns = Self::default_visible_columns();
        }
        if self.sort_profiles.is_empty() {
            self.sort_profiles = Self::default_sort_profiles();
        }
        self.alert_thresholds = self.alert_thresholds.clamp();
        self.refresh_interval_ms = self.refresh_interval_ms.clamp(250, 2500).max(250).min(2500);
        self.max_processes = self.max_processes.max(10).min(250);
    }

    pub fn theme_name(&self) -> &'static str {
        self.theme.name()
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        match fs::read_to_string(path) {
            Ok(contents) => {
                let mut config: Self = serde_json::from_str(&contents)
                    .or_else(|_| toml::from_str(&contents))
                    .map_err(|error| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("invalid config: {error}"),
                        )
                    })?;
                config.apply_defaults();
                Ok(config)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::load_from_file(path)
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialize = |value: &Self| -> io::Result<String> {
            let ext = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("json")
                .to_ascii_lowercase();
            match ext.as_str() {
                "toml" => toml::to_string_pretty(value)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string())),
                _ => serde_json::to_string_pretty(value)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string())),
            }
        };
        let data = serialize(self)?;
        fs::write(path, data)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        self.save_to_file(path)
    }

    #[allow(dead_code)]
    pub fn from_cli_args<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let options = CliOptions::parse(args)?;
        let mut config = match options.config_path.as_ref() {
            Some(path) => Self::load_from_file(path).unwrap_or_else(|_| Self::default()),
            None => Self::default(),
        };
        if let Some(theme) = options.theme {
            config.theme = theme;
        }
        if let Some(refresh_interval_ms) = options.refresh_interval_ms {
            config.refresh_interval_ms = refresh_interval_ms;
        }
        if let Some(max_processes) = options.max_processes {
            config.max_processes = max_processes;
        }
        if let Some(show_full_command) = options.show_full_command {
            config.show_full_command = show_full_command;
        }
        config.apply_defaults();
        Ok(config)
    }

    pub fn set_visible_columns(&mut self, columns: Vec<DisplayColumn>) {
        let mut unique = Vec::new();
        for column in columns {
            if !unique.contains(&column) {
                unique.push(column);
            }
        }
        self.visible_columns = if unique.is_empty() {
            Self::default_visible_columns()
        } else {
            unique
        };
    }

    pub fn toggle_column(&mut self, column: DisplayColumn) {
        if self.visible_columns.contains(&column) {
            self.visible_columns.retain(|value| *value != column);
            if self.visible_columns.is_empty() {
                self.visible_columns = Self::default_visible_columns();
            }
        } else {
            self.visible_columns.push(column);
        }
    }
}

#[derive(Default, Clone, Debug)]
struct CliOptions {
    config_path: Option<PathBuf>,
    theme: Option<Theme>,
    refresh_interval_ms: Option<u64>,
    max_processes: Option<usize>,
    show_full_command: Option<bool>,
    compact_mode: Option<bool>,
    filter: Option<String>,
    sort_mode: Option<SortMode>,
    export_path: Option<PathBuf>,
    import_path: Option<PathBuf>,
}

impl CliOptions {
    fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut options = Self::default();
        let mut args = args.into_iter().peekable();

        while let Some(raw_arg) = args.next() {
            let arg = raw_arg.as_ref();
            match arg {
                "--config" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--config requires a path".to_string())?;
                    options.config_path = Some(PathBuf::from(value.as_ref()));
                }
                "--theme" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--theme requires a value".to_string())?;
                    options.theme = Some(
                        Theme::from_name(value.as_ref())
                            .ok_or_else(|| format!("unknown theme '{}'", value.as_ref()))?,
                    );
                }
                "--refresh" | "--refresh-interval" => {
                    let value = args
                        .next()
                        .ok_or_else(|| format!("{arg} requires a millisecond value"))?;
                    options.refresh_interval_ms = Some(
                        value
                            .as_ref()
                            .parse::<u64>()
                            .map_err(|_| format!("invalid refresh value '{}': expected u64", value.as_ref()))?,
                    );
                }
                "--limit" | "--max-processes" => {
                    let value = args
                        .next()
                        .ok_or_else(|| format!("{arg} requires a numeric value"))?;
                    options.max_processes = Some(
                        value
                            .as_ref()
                            .parse::<usize>()
                            .map_err(|_| format!("invalid process limit '{}': expected usize", value.as_ref()))?,
                    );
                }
                "--show-full-command" => options.show_full_command = Some(true),
                "--hide-full-command" => options.show_full_command = Some(false),
                "--compact" => options.compact_mode = Some(true),
                "--no-compact" => options.compact_mode = Some(false),
                "--filter" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--filter requires a query".to_string())?;
                    options.filter = Some(value.as_ref().to_string());
                }
                "--import" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--import requires a path".to_string())?;
                    options.import_path = Some(PathBuf::from(value.as_ref()));
                }
                "--export" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--export requires a path".to_string())?;
                    options.export_path = Some(PathBuf::from(value.as_ref()));
                }
                "--sort" | "--sort-mode" => {
                    let value = args
                        .next()
                        .ok_or_else(|| format!("{arg} requires a value"))?;
                    options.sort_mode = Some(
                        match value.as_ref().to_ascii_lowercase().as_str() {
                            "cpu" => SortMode::Cpu,
                            "memory" => SortMode::Memory,
                            "name" => SortMode::Name,
                            "pid" => SortMode::Pid,
                            "threads" => SortMode::Threads,
                            _ => return Err(format!("unknown sort mode '{}'", value.as_ref())),
                        },
                    );
                }
                "--help" | "-h" => {
                    return Err(
                        "Usage: hyper-top [--config PATH] [--theme default|solarized|midnight] [--refresh 900] [--limit 80] [--sort cpu|memory|name|pid|threads] [--filter QUERY] [--show-full-command|--hide-full-command] [--compact|--no-compact] [--import PATH] [--export PATH]".to_string(),
                    )
                }
                _ if arg.starts_with("--") => {
                    return Err(format!("unknown option '{arg}'"));
                }
                _ => {
                    return Err(format!("unexpected positional argument '{arg}'"));
                }
            }
        }

        Ok(options)
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
    pub runtime: Duration,
}

impl ProcessItem {
    pub fn runtime_label(&self) -> String {
        let seconds = self.runtime.as_secs();
        let days = seconds / 86_400;
        let hours = (seconds % 86_400) / 3_600;
        let minutes = (seconds % 3_600) / 60;
        let secs = seconds % 60;
        format!("{}d {:02}h {:02}m {:02}s", days, hours, minutes, secs)
    }

    pub fn severity_with_thresholds(
        &self,
        total_memory_mb: u64,
        thresholds: AlertThresholds,
    ) -> ProcessSeverity {
        let memory_pct = if total_memory_mb == 0 {
            0.0
        } else {
            (self.mem_mb as f64 / total_memory_mb as f64) * 100.0
        };

        if self.cpu >= thresholds.critical_cpu
            || memory_pct >= thresholds.critical_memory_pct as f64
        {
            ProcessSeverity::Critical
        } else if self.cpu >= thresholds.warning_cpu
            || memory_pct >= thresholds.warning_memory_pct as f64
        {
            ProcessSeverity::Warning
        } else {
            ProcessSeverity::Normal
        }
    }
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
    pub config_path: Option<PathBuf>,
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
            config_path: None,
        }
    }

    pub fn from_cli_args<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let options = CliOptions::parse(args)?;
        let load_path = options.config_path.clone();
        let mut app = match load_path.as_ref() {
            Some(path) => Self::load_from_file(path)
                .unwrap_or_else(|_| Self::with_config(AppConfig::default())),
            None => Self::with_config(AppConfig::default()),
        };
        app.config_path = load_path;
        if let Some(theme) = options.theme {
            app.config.theme = theme;
        }
        if let Some(refresh_interval_ms) = options.refresh_interval_ms {
            app.config.refresh_interval_ms = refresh_interval_ms;
        }
        if let Some(max_processes) = options.max_processes {
            app.config.max_processes = max_processes;
        }
        if let Some(show_full_command) = options.show_full_command {
            app.config.show_full_command = show_full_command;
        }
        if let Some(compact_mode) = options.compact_mode {
            app.config.compact_mode = compact_mode;
        }
        if let Some(filter) = options.filter {
            app.query = filter;
        }
        if let Some(sort_mode) = options.sort_mode {
            app.sort_mode = sort_mode;
        }
        if let Some(export_path) = options.export_path {
            app.export_config(export_path.clone())
                .map_err(|error| format!("failed to export config: {error}"))?;
        }
        if let Some(import_path) = options.import_path {
            app.import_config(import_path)
                .map_err(|error| format!("failed to import config: {error}"))?;
        }
        app.config.apply_defaults();
        Ok(app)
    }
    pub fn load_from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let config = AppConfig::load_from_file(&path)?;
        let mut app = Self::with_config(config);
        app.config_path = Some(path);
        Ok(app)
    }

    pub fn save_config(&self, path: impl AsRef<Path>) -> io::Result<()> {
        self.config.save_to_file(path)
    }

    #[allow(dead_code)]
    pub fn save_current_config(&self) -> io::Result<()> {
        match &self.config_path {
            Some(path) => self.save_config(path),
            None => Ok(()),
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

    pub fn add_filter_preset(&mut self, name: impl Into<String>, query: impl Into<String>) {
        let name = name.into();
        let query = query.into();
        match self
            .config
            .filter_presets
            .iter_mut()
            .find(|preset| preset.name == name)
        {
            Some(preset) => preset.query = query,
            None => self
                .config
                .filter_presets
                .push(FilterPreset { name, query }),
        }
    }

    pub fn apply_filter_preset(&mut self, name: &str) -> Option<String> {
        let query = self
            .config
            .filter_presets
            .iter()
            .find(|preset| preset.name == name)?
            .query
            .clone();
        self.query = query.clone();
        Some(query)
    }

    pub fn remove_filter_preset(&mut self, name: &str) {
        self.config
            .filter_presets
            .retain(|preset| preset.name != name);
    }

    pub fn add_sort_profile(&mut self, name: impl Into<String>, sort_mode: SortMode) {
        let name = name.into();
        match self
            .config
            .sort_profiles
            .iter_mut()
            .find(|profile| profile.name == name)
        {
            Some(profile) => profile.sort_mode = sort_mode,
            None => self
                .config
                .sort_profiles
                .push(SortProfile { name, sort_mode }),
        }
    }

    pub fn apply_sort_profile(&mut self, name: &str) -> Option<SortMode> {
        let sort_mode = self
            .config
            .sort_profiles
            .iter()
            .find(|profile| profile.name == name)?
            .sort_mode;
        self.sort_mode = sort_mode;
        Some(sort_mode)
    }

    pub fn remove_sort_profile(&mut self, name: &str) {
        self.config
            .sort_profiles
            .retain(|profile| profile.name != name);
    }

    pub fn set_visible_columns(&mut self, columns: Vec<DisplayColumn>) {
        self.config.set_visible_columns(columns);
    }

    pub fn toggle_visible_column(&mut self, column: DisplayColumn) {
        self.config.toggle_column(column);
    }

    pub fn toggle_visible_column_by_index(&mut self, index: usize) {
        let columns = self.config.visible_columns.clone();
        let column = match columns.get(index).copied() {
            Some(column) => column,
            None => return,
        };
        self.config.toggle_column(column);
    }

    pub fn apply_quick_filter(&mut self, index: usize) -> bool {
        let preset = self
            .config
            .filter_presets
            .get(index)
            .map(|preset| preset.query.clone());
        match preset {
            Some(query) => {
                self.query = query;
                true
            }
            None => false,
        }
    }

    pub fn toggle_compact_mode(&mut self) {
        self.config.compact_mode = !self.config.compact_mode;
    }

    pub fn export_config(&self, path: impl AsRef<Path>) -> io::Result<()> {
        self.config.save_to_file(path)
    }

    pub fn import_config(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let config = AppConfig::load_from_file(path)?;
        self.config = config;
        Ok(())
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
    use super::{
        AlertThresholds, App, AppConfig, DisplayColumn, FilterPreset, FocusedBlock, ProcessItem,
        SortMode, SortProfile, Theme,
    };
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

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
            runtime: std::time::Duration::from_secs(120),
        }
    }

    fn unique_temp_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{nanos}.json"))
    }

    #[test]
    fn alert_thresholds_keep_critical_values_above_warning_values() {
        let thresholds = AlertThresholds {
            warning_cpu: 80.0,
            critical_cpu: 40.0,
            warning_memory_pct: 75.0,
            critical_memory_pct: 55.0,
        }
        .clamp();

        assert_eq!(thresholds.warning_cpu, 80.0);
        assert_eq!(thresholds.critical_cpu, 80.0);
        assert_eq!(thresholds.warning_memory_pct, 75.0);
        assert_eq!(thresholds.critical_memory_pct, 75.0);
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
            compact_mode: false,
            alert_thresholds: AlertThresholds::default(),
            visible_columns: AppConfig::default_visible_columns(),
            filter_presets: Vec::new(),
            sort_profiles: AppConfig::default_sort_profiles(),
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
            runtime: std::time::Duration::from_secs(3_600),
        }];
        app.selected_process = 0;

        let info = app.selected_process_info().unwrap();
        assert_eq!(info.command, "gnome-shell --replace");
        assert_eq!(info.status, "run");
        assert_eq!(info.threads, 32);
        assert_eq!(info.parent_pid, 1);
        assert_eq!(info.runtime_label(), "0d 01h 00m 00s");
        assert_eq!(app.selected_pid(), Some(4301));
    }

    #[test]
    fn config_file_round_trip_persists_user_prefs() {
        let path = unique_temp_path("hyper-top-config");
        let mut original = AppConfig::default();
        original.refresh_interval_ms = 1500;
        original.max_processes = 24;
        original.theme = Theme::Solarized;
        original.show_full_command = false;
        original.compact_mode = true;
        original.visible_columns =
            vec![DisplayColumn::Pid, DisplayColumn::Name, DisplayColumn::Cpu];
        original.filter_presets = vec![FilterPreset {
            name: "postgres".to_string(),
            query: "postgres".to_string(),
        }];
        original.sort_profiles = vec![SortProfile {
            name: "Memory".to_string(),
            sort_mode: SortMode::Memory,
        }];

        original.save(&path).unwrap();
        let loaded = AppConfig::load(&path).unwrap();
        assert_eq!(loaded.refresh_interval_ms, 1500);
        assert_eq!(loaded.max_processes, 24);
        assert_eq!(loaded.theme, Theme::Solarized);
        assert!(!loaded.show_full_command);
        assert!(loaded.compact_mode);
        assert_eq!(
            loaded.visible_columns,
            vec![DisplayColumn::Pid, DisplayColumn::Name, DisplayColumn::Cpu]
        );
        assert_eq!(loaded.filter_presets[0].name, "postgres");
        assert_eq!(loaded.sort_profiles[0].sort_mode, SortMode::Memory);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn compact_mode_and_quick_filters_are_applied_from_config() {
        let mut app = App::new();
        app.config.filter_presets = vec![
            FilterPreset {
                name: "db".to_string(),
                query: "postgres".to_string(),
            },
            FilterPreset {
                name: "web".to_string(),
                query: "nginx".to_string(),
            },
        ];

        assert!(!app.config.compact_mode);
        app.toggle_compact_mode();
        assert!(app.config.compact_mode);

        assert!(app.apply_quick_filter(0));
        assert_eq!(app.query, "postgres");
        assert!(app.apply_quick_filter(1));
        assert_eq!(app.query, "nginx");
        assert!(!app.apply_quick_filter(99));
    }

    #[test]
    fn theme_palette_and_config_support_filter_and_sort_profiles() {
        let mut app = App::new();
        app.add_filter_preset("postgres", "postgres");
        app.add_filter_preset("all", "");
        assert_eq!(
            app.apply_filter_preset("postgres"),
            Some("postgres".to_string())
        );
        assert_eq!(app.query, "postgres");
        app.remove_filter_preset("all");
        assert!(!app
            .config
            .filter_presets
            .iter()
            .any(|preset| preset.name == "all"));

        app.add_sort_profile("low-memory", SortMode::Memory);
        assert_eq!(app.apply_sort_profile("low-memory"), Some(SortMode::Memory));
        assert_eq!(app.sort_mode, SortMode::Memory);
        app.remove_sort_profile("low-memory");
        assert!(!app
            .config
            .sort_profiles
            .iter()
            .any(|profile| profile.name == "low-memory"));

        app.config.theme = Theme::Midnight;
        let palette = app.config.theme.palette();
        assert_eq!(palette.accent, ratatui::style::Color::LightCyan);
        assert_eq!(palette.footer, ratatui::style::Color::LightRed);
        assert_eq!(Theme::from_name("midnight"), Some(Theme::Midnight));

        app.set_visible_columns(vec![DisplayColumn::Pid, DisplayColumn::Name]);
        assert_eq!(
            app.config.visible_columns,
            vec![DisplayColumn::Pid, DisplayColumn::Name]
        );
        app.toggle_visible_column(DisplayColumn::Threads);
        assert!(app.config.visible_columns.contains(&DisplayColumn::Threads));
        app.toggle_visible_column(DisplayColumn::Threads);
        assert!(!app.config.visible_columns.contains(&DisplayColumn::Threads));
        assert_eq!(
            DisplayColumn::from_name("threads"),
            Some(DisplayColumn::Threads)
        );
    }

    #[test]
    fn cli_args_override_config_defaults_and_load_files() {
        let path = unique_temp_path("hyper-top-cli");
        let mut config = AppConfig::default();
        config.theme = Theme::Solarized;
        config.refresh_interval_ms = 1200;
        config.max_processes = 30;
        config.save_to_file(&path).unwrap();

        let app = App::from_cli_args([
            "--config",
            path.to_str().unwrap(),
            "--theme",
            "midnight",
            "--refresh",
            "500",
            "--limit",
            "12",
            "--sort",
            "memory",
            "--filter",
            "database",
        ])
        .unwrap();

        assert_eq!(app.config.theme, Theme::Midnight);
        assert_eq!(app.config.refresh_interval_ms, 500);
        assert_eq!(app.config.max_processes, 12);
        assert_eq!(app.sort_mode, SortMode::Memory);
        assert_eq!(app.query, "database");
        assert_eq!(app.config_path, Some(path.clone()));

        let _ = fs::remove_file(path);
    }
}
