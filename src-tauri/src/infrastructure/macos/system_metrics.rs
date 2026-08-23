use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemMetricsSnapshot {
    pub cpu_percent: u8,
    pub memory_percent: u8,
}

struct SamplingState {
    system: System,
    smoothed_cpu: f32,
    last_sample_at: Option<Instant>,
    snapshot: SystemMetricsSnapshot,
}

pub struct SystemMetricsMonitor {
    state: Mutex<SamplingState>,
}

impl Default for SystemMetricsMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMetricsMonitor {
    pub fn new() -> Self {
        let refreshes = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::nothing().with_ram());
        Self {
            state: Mutex::new(SamplingState {
                system: System::new_with_specifics(refreshes),
                smoothed_cpu: 0.0,
                last_sample_at: None,
                snapshot: SystemMetricsSnapshot::default(),
            }),
        }
    }

    pub fn poll(&self) -> Result<SystemMetricsSnapshot, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "system metrics state is unavailable".to_owned())?;
        let now = Instant::now();
        if state
            .last_sample_at
            .is_some_and(|sampled_at| now.duration_since(sampled_at) < Duration::from_millis(750))
        {
            return Ok(state.snapshot);
        }

        state.system.refresh_cpu_usage();
        state.system.refresh_memory();
        let measured_cpu = state.system.global_cpu_usage();
        state.smoothed_cpu = if state.last_sample_at.is_some() {
            state.smoothed_cpu * 0.65 + measured_cpu * 0.35
        } else {
            measured_cpu
        };
        let total_memory = state.system.total_memory();
        let memory_percent = if total_memory == 0 {
            0
        } else {
            ((state.system.used_memory() as f64 / total_memory as f64) * 100.0)
                .round()
                .clamp(0.0, 100.0) as u8
        };
        state.last_sample_at = Some(now);
        state.snapshot = SystemMetricsSnapshot {
            cpu_percent: state.smoothed_cpu.round().clamp(0.0, 100.0) as u8,
            memory_percent,
        };
        Ok(state.snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_metrics_are_available_and_bounded() {
        let snapshot = SystemMetricsMonitor::new().poll().unwrap();
        assert!(snapshot.cpu_percent <= 100);
        assert!(snapshot.memory_percent <= 100);
        assert!(snapshot.memory_percent > 0);
    }

    #[test]
    fn rapid_polls_reuse_the_latest_snapshot() {
        let monitor = SystemMetricsMonitor::new();
        let first = monitor.poll().unwrap();
        assert_eq!(monitor.poll().unwrap().memory_percent, first.memory_percent);
    }
}
