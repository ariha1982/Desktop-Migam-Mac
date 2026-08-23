use std::sync::{atomic::AtomicBool, RwLock};

use crate::{
    application::{
        foreground_monitor::ForegroundMonitor, gamcha_service::GamchaService,
        pomodoro_service::PomodoroService, settings_service::SettingsService,
        todo_service::TodoService,
    },
    domain::settings::Settings,
    infrastructure::{
        PlatformForegroundWindowSource, PlatformWindowMinimizer, SystemMetricsMonitor,
    },
};

pub struct AppState {
    pub settings: RwLock<Settings>,
    pub settings_service: SettingsService,
    pub pomodoro_service: PomodoroService,
    pub gamcha_service: GamchaService,
    pub todo_service: TodoService,
    pub foreground_monitor: ForegroundMonitor,
    pub system_metrics_monitor: SystemMetricsMonitor,
    pub emergency_stopped: AtomicBool,
    pub emergency_shortcut_available: AtomicBool,
    pub tray_available: AtomicBool,
}

impl AppState {
    pub fn new(
        settings: Settings,
        settings_service: SettingsService,
        gamcha_service: GamchaService,
        todo_service: TodoService,
    ) -> Self {
        let pomodoro_service = PomodoroService::new(&settings.pomodoro);
        Self {
            settings: RwLock::new(settings),
            settings_service,
            pomodoro_service,
            gamcha_service,
            todo_service,
            foreground_monitor: ForegroundMonitor::new(
                Box::new(PlatformForegroundWindowSource::new()),
                Box::new(PlatformWindowMinimizer),
                std::process::id(),
            ),
            system_metrics_monitor: SystemMetricsMonitor::new(),
            emergency_stopped: AtomicBool::new(false),
            emergency_shortcut_available: AtomicBool::new(false),
            tray_available: AtomicBool::new(false),
        }
    }
}
