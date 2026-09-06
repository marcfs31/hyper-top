use crate::app::{ProcessItem, SystemState};
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, ProcessRefreshKind, ProcessStatus, RefreshKind, System};
use tokio::sync::mpsc;

pub enum ProcessCommand {
    Kill(u32),
}

pub fn spawn_telemetry_engine(
    tx: mpsc::Sender<SystemState>,
    mut commands: mpsc::Receiver<ProcessCommand>,
) {
    tokio::spawn(async move {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );

        let mut message = "Telemetry online".to_string();
        loop {
            tokio::time::sleep(Duration::from_millis(900)).await;
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
                }
            }

            sys.refresh_cpu();
            sys.refresh_memory();
            sys.refresh_processes();

            let global_cpu = sys.global_cpu_info().cpu_usage();
            let cpu_count = sys.cpus().len();
            let cpu_frequency_mhz = sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0);
            let load = System::load_average();
            let ram_used = sys.used_memory() as f64 / 1_073_741_824.0;
            let ram_total = sys.total_memory() as f64 / 1_073_741_824.0;
            let swap_used = sys.used_swap() as f64 / 1_073_741_824.0;
            let swap_total = sys.total_swap() as f64 / 1_073_741_824.0;

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
                    ProcessItem {
                        pid: pid.to_string(),
                        name: proc.name().to_string(),
                        command,
                        cpu: proc.cpu_usage(),
                        mem_mb: proc.memory() / 1024 / 1024,
                        status: format_process_status(proc.status()),
                        threads: proc.tasks().map_or(0, |tasks| tasks.len()),
                        parent_pid: proc.parent().map_or(0, |parent| parent.as_u32()),
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
                ram_used_gb: ram_used,
                ram_total_gb: ram_total,
                swap_used_gb: swap_used,
                swap_total_gb: swap_total,
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
    use super::format_process_status;
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
}
