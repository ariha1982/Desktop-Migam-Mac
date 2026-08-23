use crate::domain::foreground::{
    ForegroundReadError, ForegroundWindowSource, WindowMinimizer, WindowSnapshot,
};

#[derive(Default)]
pub struct PlatformForegroundWindowSource;

#[derive(Default)]
pub struct PlatformWindowMinimizer;

impl PlatformForegroundWindowSource {
    pub const fn new() -> Self {
        Self
    }
}

impl ForegroundWindowSource for PlatformForegroundWindowSource {
    fn foreground_window(&self) -> Result<Option<WindowSnapshot>, ForegroundReadError> {
        Ok(None)
    }
}

impl WindowMinimizer for PlatformWindowMinimizer {
    fn minimize(&self, _window_id: isize) -> Result<(), ForegroundReadError> {
        Err(ForegroundReadError::InspectionFailed)
    }
}
