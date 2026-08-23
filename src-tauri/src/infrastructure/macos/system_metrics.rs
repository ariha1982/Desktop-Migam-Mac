#[derive(Clone, Copy, Debug, Default)]
pub struct SystemMetricsSnapshot {
    pub cpu_percent: u8,
    pub memory_percent: u8,
}

#[derive(Default)]
pub struct SystemMetricsMonitor;

impl SystemMetricsMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn poll(&self) -> Result<SystemMetricsSnapshot, String> {
        Ok(SystemMetricsSnapshot::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_metrics_are_bounded() {
        let snapshot = SystemMetricsMonitor::new().poll().unwrap();
        assert_eq!(snapshot.cpu_percent, 0);
        assert_eq!(snapshot.memory_percent, 0);
    }
}
