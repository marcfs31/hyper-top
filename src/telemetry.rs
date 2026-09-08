use crate::app::{ProcessItem, SystemState};
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, ProcessRefreshKind, ProcessStatus, RefreshKind, System, Users};
use tokio::sync::mpsc;

pub enum ProcessCommand {
    Kill(u32),
    SetRefreshInterval(u64),
}

pub fn spawn_telemetry_engine(
    tx: mpsc::Sender<SystemState>,
    mut commands: mpsc::Receiver<ProcessCommand>,
    mut refresh_interval: Duration,
) {
    tokio::spawn(async move {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );

        let mut message = "Telemetry online".to_string();
        let users = Users::new_with_refreshed_list();
        let mut networks = sysinfo::Networks::new_with_refreshed_list();
        let mut last_network_sample = Instant::now();
        loop {
            tokio::time::sleep(refresh_interval).await;
            while let Ok(command) = commands.try_recv() {
                match command {
                    ProcessCommand::Kill(pid) => {
                        let process_id = sysinfo::Pid::from_u32(pid);
                        message = match sys.process(process_id) {
                            Some(process) if process.kill() => {
                                format!("Sent termination signal to PID {pid}")
                            }
                            Some(_) => format!("Could not terminate PID {pid}"),
                            None => format!("PID {pid} is no longer running"),
                        };
                    }
                    ProcessCommand::SetRefreshInterval(interval_ms) => {
                        refresh_interval = Duration::from_millis(interval_ms.clamp(250, 2500));
                    }
                }
            }

            sys.refresh_cpu();
            sys.refresh_memory();
            sys.refresh_processes();

            let disks = sysinfo::Disks::new_with_refreshed_list();
            let global_cpu = sys.global_cpu_info().cpu_usage();
            let cpu_count = sys.cpus().len();
            let core_usage = sys
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .collect::<Vec<_>>();
            let cpu_frequency_mhz = sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0);
            let load = System::load_average();
            let ram_used = sys.used_memory() as f64 / 1_073_741_824.0;
            let ram_total = sys.total_memory() as f64 / 1_073_741_824.0;
            let swap_used = sys.used_swap() as f64 / 1_073_741_824.0;
            let swap_total = sys.total_swap() as f64 / 1_073_741_824.0;
            let storage_mounts = disks
                .iter()
                .map(|disk| {
                    let total = disk.total_space() as f64 / 1_073_741_824.0;
                    let available = disk.available_space() as f64 / 1_073_741_824.0;
                    let used =
                        (disk.total_space() - disk.available_space()) as f64 / 1_073_741_824.0;
                    crate::app::StorageMount {
                        mount: disk.mount_point().to_string_lossy().to_string(),
                        total_gb: total,
                        used_gb: used,
                        available_gb: available,
                    }
                })
                .collect::<Vec<_>>();
            let storage_disk = storage_mounts
                .iter()
                .find(|disk| disk.mount == "/" || disk.mount == "/private/var/root")
                .or_else(|| storage_mounts.first());
            let storage_total = storage_disk.map(|disk| disk.total_gb).unwrap_or(0.0);
            let storage_used = storage_disk.map(|disk| disk.used_gb).unwrap_or(0.0);
            let storage_available = storage_disk.map(|disk| disk.available_gb).unwrap_or(0.0);
            let storage_mount = storage_disk
                .map(|disk| disk.mount.clone())
                .unwrap_or_else(|| "/".to_string());

            networks.refresh_list();
            let network_now = Instant::now();
            let network_elapsed = network_now.duration_since(last_network_sample);
            last_network_sample = network_now;

            let raw_network_interfaces: Vec<crate::app::NetworkInterface> = networks
                .iter()
                .map(|(name, data)| crate::app::NetworkInterface {
                    name: name.clone(),
                    rx_bytes_per_sec: bytes_per_second(data.received(), network_elapsed),
                    tx_bytes_per_sec: bytes_per_second(data.transmitted(), network_elapsed),
                    total_rx_bytes: data.total_received(),
                    total_tx_bytes: data.total_transmitted(),
                })
                .collect();
            let network_interfaces = select_network_interfaces(raw_network_interfaces);

            let mut procs: Vec<ProcessItem> = sys
                .processes()
                .iter()
                .map(|(pid, proc)| {
                    let command = proc.cmd().join(" ");
                    let command = if command.is_empty() {
                        proc.name().to_string()
                    } else {
                        command
                    };
                    let parent = proc.parent().and_then(|parent| sys.process(parent));
                    let parent_uid = parent
                        .and_then(|process| process.user_id())
                        .map(|user_id| user_id.to_string())
                        .unwrap_or_else(|| "N/A".to_string());
                    let parent_user = parent
                        .and_then(|process| process.user_id())
                        .and_then(|user_id| users.get_user_by_id(user_id))
                        .map(|user| user.name().to_string())
                        .unwrap_or_else(|| "?".to_string());
                    ProcessItem {
                        pid: pid.to_string(),
                        name: proc.name().to_string(),
                        user: proc
                            .user_id()
                            .and_then(|user_id| users.get_user_by_id(user_id))
                            .map(|user| user.name().to_string())
                            .unwrap_or_else(|| "?".to_string()),
                        uid: proc
                            .user_id()
                            .map(|user_id| user_id.to_string())
                            .unwrap_or_else(|| "?".to_string()),
                        command,
                        cpu: proc.cpu_usage(),
                        mem_mb: proc.memory() / 1024 / 1024,
                        status: format_process_status(proc.status()),
                        threads: proc.tasks().map_or(0, |tasks| tasks.len()),
                        parent_pid: proc.parent().map_or(0, |parent| parent.as_u32()),
                        parent_user,
                        parent_uid,
                        runtime: Duration::from_secs(proc.run_time()),
                    }
                })
                .collect();

            procs.sort_by(|a, b| {
                b.cpu
                    .partial_cmp(&a.cpu)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            procs.truncate(120);

            let total_threads = sys
                .processes()
                .values()
                .map(|proc| proc.tasks().map_or(0, |tasks| tasks.len()))
                .sum::<usize>();

            let snapshot = SystemState {
                cpu_usage: global_cpu,
                cpu_count,
                cpu_frequency_mhz,
                core_usage,
                ram_used_gb: ram_used,
                ram_total_gb: ram_total,
                swap_used_gb: swap_used,
                swap_total_gb: swap_total,
                storage_total_gb: storage_total,
                storage_used_gb: storage_used,
                storage_available_gb: storage_available,
                storage_mount,
                storage_mounts,
                network_interfaces,
                uptime: Duration::from_secs(System::uptime()),
                load_average: [load.one, load.five, load.fifteen],
                running_processes: sys.processes().len(),
                total_threads,
                processes: procs,
                refreshed_at: Instant::now(),
                message: message.clone(),
            };

            if tx.send(snapshot).await.is_err() {
                break;
            }
        }
    });
}

fn bytes_per_second(delta_bytes: u64, elapsed: Duration) -> f64 {
    let seconds = elapsed.as_secs_f64();
    if seconds <= 0.0 {
        0.0
    } else {
        delta_bytes as f64 / seconds
    }
}

fn is_loopback_interface(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower == "lo" || lower.contains("loopback") {
        return true;
    }
    match lower.strip_prefix("lo") {
        Some(rest) if !rest.is_empty() => rest.chars().all(|c| c.is_ascii_digit()),
        _ => false,
    }
}

fn select_network_interfaces(
    mut interfaces: Vec<crate::app::NetworkInterface>,
) -> Vec<crate::app::NetworkInterface> {
    interfaces.retain(|iface| {
        !is_loopback_interface(&iface.name)
            && (iface.total_rx_bytes > 0 || iface.total_tx_bytes > 0)
    });
    interfaces.sort_by(|a, b| {
        let a_rate = a.rx_bytes_per_sec + a.tx_bytes_per_sec;
        let b_rate = b.rx_bytes_per_sec + b.tx_bytes_per_sec;
        b_rate
            .partial_cmp(&a_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    interfaces
}

fn format_process_status(status: ProcessStatus) -> String {
    match status {
        ProcessStatus::Idle => "idle".to_string(),
        ProcessStatus::Run => "run".to_string(),
        ProcessStatus::Sleep => "sleep".to_string(),
        ProcessStatus::Stop => "stop".to_string(),
        ProcessStatus::Zombie => "zombie".to_string(),
        ProcessStatus::Tracing => "tracing".to_string(),
        ProcessStatus::Dead => "dead".to_string(),
        ProcessStatus::Wakekill => "wakekill".to_string(),
        ProcessStatus::Waking => "waking".to_string(),
        ProcessStatus::Parked => "parked".to_string(),
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        bytes_per_second, format_process_status, is_loopback_interface, select_network_interfaces,
    };
    use crate::app::NetworkInterface;
    use std::time::Duration;
    use sysinfo::ProcessStatus;

    #[test]
    fn formats_process_status_labels_for_common_states() {
        assert_eq!(format_process_status(ProcessStatus::Idle), "idle");
        assert_eq!(format_process_status(ProcessStatus::Run), "run");
        assert_eq!(format_process_status(ProcessStatus::Sleep), "sleep");
        assert_eq!(format_process_status(ProcessStatus::Stop), "stop");
        assert_eq!(format_process_status(ProcessStatus::Zombie), "zombie");
        assert_eq!(format_process_status(ProcessStatus::Tracing), "tracing");
        assert_eq!(format_process_status(ProcessStatus::Dead), "dead");
        assert_eq!(format_process_status(ProcessStatus::Wakekill), "wakekill");
        assert_eq!(format_process_status(ProcessStatus::Waking), "waking");
        assert_eq!(format_process_status(ProcessStatus::Parked), "parked");
    }

    #[test]
    fn bytes_per_second_returns_zero_for_zero_elapsed_time() {
        assert_eq!(bytes_per_second(1_000, Duration::ZERO), 0.0);
    }

    #[test]
    fn bytes_per_second_divides_delta_by_elapsed_seconds() {
        assert_eq!(bytes_per_second(2_000, Duration::from_secs(2)), 1_000.0);
        assert_eq!(bytes_per_second(1_000, Duration::from_millis(500)), 2_000.0);
    }

    #[test]
    fn is_loopback_interface_matches_common_platform_names_only() {
        assert!(is_loopback_interface("lo"));
        assert!(is_loopback_interface("lo0"));
        assert!(is_loopback_interface("LO0"));
        assert!(is_loopback_interface("Loopback Pseudo-Interface 1"));

        assert!(!is_loopback_interface("en0"));
        assert!(!is_loopback_interface("eth0"));
        assert!(!is_loopback_interface("utun4"));
        assert!(!is_loopback_interface("lockdown0"));
        assert!(!is_loopback_interface("local0"));
    }

    fn interface(
        name: &str,
        rx_rate: f64,
        tx_rate: f64,
        total_rx: u64,
        total_tx: u64,
    ) -> NetworkInterface {
        NetworkInterface {
            name: name.to_string(),
            rx_bytes_per_sec: rx_rate,
            tx_bytes_per_sec: tx_rate,
            total_rx_bytes: total_rx,
            total_tx_bytes: total_tx,
        }
    }

    #[test]
    fn select_network_interfaces_excludes_loopback_and_idle_interfaces_and_sorts_by_rate() {
        let interfaces = vec![
            interface("lo0", 0.0, 0.0, 500, 500),
            interface("en5", 0.0, 0.0, 0, 0),
            interface("en0", 100.0, 50.0, 10_000, 5_000),
            interface("utun4", 5_000.0, 1_000.0, 20_000, 2_000),
        ];

        let selected = select_network_interfaces(interfaces);
        let names: Vec<&str> = selected.iter().map(|iface| iface.name.as_str()).collect();

        assert_eq!(names, vec!["utun4", "en0"]);
    }
}
