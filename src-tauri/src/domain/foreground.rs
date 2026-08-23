#[derive(Clone, Eq, PartialEq)]
pub struct WindowSnapshot {
    pub window_id: isize,
    pub process_id: u32,
    pub process_name: Option<String>,
    pub title: Option<String>,
    pub is_visible: bool,
    pub is_minimized: bool,
    pub is_fullscreen: bool,
    pub monitor_left: i32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub trait ForegroundWindowSource: Send + Sync {
    fn foreground_window(&self) -> Result<Option<WindowSnapshot>, ForegroundReadError>;

    fn foreground_window_excluding(
        &self,
        process_id: u32,
    ) -> Result<Option<WindowSnapshot>, ForegroundReadError> {
        Ok(self
            .foreground_window()?
            .filter(|snapshot| snapshot.process_id != process_id))
    }

    fn monitored_foreground_window_excluding(
        &self,
        process_id: u32,
    ) -> Result<Option<WindowSnapshot>, ForegroundReadError> {
        self.foreground_window_excluding(process_id)
    }
}

pub trait WindowMinimizer: Send + Sync {
    fn minimize(&self, window_id: isize) -> Result<(), ForegroundReadError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForegroundReadError {
    AccessDenied,
    InspectionFailed,
}
