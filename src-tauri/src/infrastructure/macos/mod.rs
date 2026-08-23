mod foreground_window;
mod system_metrics;

pub use foreground_window::{
    focus_guard_permissions, PlatformForegroundWindowSource, PlatformWindowMinimizer,
};
pub use system_metrics::{SystemMetricsMonitor, SystemMetricsSnapshot};
