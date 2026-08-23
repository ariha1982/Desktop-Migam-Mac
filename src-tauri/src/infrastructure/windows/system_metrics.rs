use std::{
    mem::size_of,
    sync::Mutex,
    time::{Duration, Instant},
};

use windows_sys::Win32::{
    Foundation::FILETIME,
    System::{
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        Threading::GetSystemTimes,
    },
};

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemMetricsSnapshot {
    pub cpu_percent: u8,
    pub memory_percent: u8,
}

#[derive(Clone, Copy)]
struct CpuTimes {
    idle: u64,
    kernel: u64,
    user: u64,
}

#[derive(Default)]
struct SamplingState {
    previous: Option<CpuTimes>,
    smoothed_cpu: f64,
    last_sample_at: Option<Instant>,
    snapshot: SystemMetricsSnapshot,
}

#[derive(Default)]
pub struct SystemMetricsMonitor {
    state: Mutex<SamplingState>,
}

impl SystemMetricsMonitor {
    pub fn new() -> Self {
        Self::default()
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
        let times = cpu_times()?;
        let memory_percent = memory_load()?;
        let measured_cpu = state.previous.map_or(0.0, |previous| {
            let idle = times.idle.saturating_sub(previous.idle);
            let total = times
                .kernel
                .saturating_sub(previous.kernel)
                .saturating_add(times.user.saturating_sub(previous.user));
            if total == 0 {
                0.0
            } else {
                (1.0 - idle as f64 / total as f64) * 100.0
            }
        });
        state.smoothed_cpu = if state.previous.is_some() {
            state.smoothed_cpu * 0.65 + measured_cpu * 0.35
        } else {
            measured_cpu
        };
        state.previous = Some(times);
        state.last_sample_at = Some(now);
        state.snapshot = SystemMetricsSnapshot {
            cpu_percent: state.smoothed_cpu.round().clamp(0.0, 100.0) as u8,
            memory_percent,
        };
        Ok(state.snapshot)
    }
}

fn file_time_value(value: FILETIME) -> u64 {
    (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
}

fn cpu_times() -> Result<CpuTimes, String> {
    let mut idle = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut kernel = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut user = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let success = unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) };
    if success == 0 {
        return Err("CPU usage is unavailable".to_owned());
    }
    Ok(CpuTimes {
        idle: file_time_value(idle),
        kernel: file_time_value(kernel),
        user: file_time_value(user),
    })
}

fn memory_load() -> Result<u8, String> {
    let mut status = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return Err("memory usage is unavailable".to_owned());
    }
    Ok(status.dwMemoryLoad.min(100) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_metrics_are_available_and_bounded() {
        let snapshot = SystemMetricsMonitor::new().poll().unwrap();
        assert!(snapshot.cpu_percent <= 100);
        assert!(snapshot.memory_percent <= 100);
    }
}
