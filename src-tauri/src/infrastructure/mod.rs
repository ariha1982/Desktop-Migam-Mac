#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
pub use macos::{
    PlatformForegroundWindowSource, PlatformWindowMinimizer, SystemMetricsMonitor,
    SystemMetricsSnapshot,
};
#[cfg(windows)]
pub use windows::{
    PlatformForegroundWindowSource, PlatformWindowMinimizer, SystemMetricsMonitor,
    SystemMetricsSnapshot,
};
