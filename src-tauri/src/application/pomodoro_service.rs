use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::Serialize;

use crate::domain::{
    pomodoro::{PomodoroEvent, PomodoroMachine, PomodoroPhase},
    settings::PomodoroSettings,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerState {
    pub phase: PomodoroPhase,
    pub remaining_seconds: u64,
    pub completed_focus_sessions: u32,
}

pub struct PomodoroService {
    machine: Mutex<PomodoroMachine>,
    configuration: Mutex<PomodoroConfiguration>,
}

#[derive(Clone, Copy)]
struct PomodoroConfiguration {
    focus_duration: Duration,
    short_break_duration: Duration,
    long_break_duration: Duration,
    sessions_before_long_break: u32,
}

impl PomodoroService {
    pub fn new(settings: &PomodoroSettings) -> Self {
        let configuration = Self::configuration(settings);
        Self {
            machine: Mutex::new(Self::machine(configuration)),
            configuration: Mutex::new(configuration),
        }
    }

    pub fn dispatch(
        &self,
        event: PomodoroEvent,
        now: Instant,
    ) -> Result<(TimerState, bool), String> {
        let (snapshot, changed, _) = self.dispatch_internal(event, now)?;
        Ok((snapshot, changed))
    }

    pub fn tick(&self, now: Instant) -> Result<(TimerState, bool, bool), String> {
        self.dispatch_internal(PomodoroEvent::Tick, now)
    }

    fn dispatch_internal(
        &self,
        event: PomodoroEvent,
        now: Instant,
    ) -> Result<(TimerState, bool, bool), String> {
        let mut machine = self
            .machine
            .lock()
            .map_err(|_| "timer state is unavailable".to_owned())?;
        let previous_phase = machine.phase();
        let previous_completed = machine.completed_focus_sessions();
        let changed = !machine.reduce(event, now).is_empty();
        let focus_completed_naturally = event == PomodoroEvent::Tick
            && previous_phase == PomodoroPhase::Focus
            && machine.completed_focus_sessions() > previous_completed;
        let configuration = *self
            .configuration
            .lock()
            .map_err(|_| "timer settings are unavailable".to_owned())?;
        if machine.phase() == PomodoroPhase::Stopped {
            machine.reconfigure(
                configuration.focus_duration,
                configuration.short_break_duration,
                configuration.long_break_duration,
                configuration.sessions_before_long_break,
            );
        }
        Ok((
            Self::snapshot(&machine, now, configuration.focus_duration),
            changed,
            focus_completed_naturally,
        ))
    }

    pub fn update_settings(&self, settings: &PomodoroSettings) -> Result<(), String> {
        let configuration = Self::configuration(settings);
        let mut machine = self
            .machine
            .lock()
            .map_err(|_| "timer state is unavailable".to_owned())?;
        *self
            .configuration
            .lock()
            .map_err(|_| "timer settings are unavailable".to_owned())? = configuration;
        machine.reconfigure(
            configuration.focus_duration,
            configuration.short_break_duration,
            configuration.long_break_duration,
            configuration.sessions_before_long_break,
        );
        Ok(())
    }

    fn configuration(settings: &PomodoroSettings) -> PomodoroConfiguration {
        PomodoroConfiguration {
            focus_duration: Duration::from_secs(u64::from(settings.focus_minutes) * 60),
            short_break_duration: Duration::from_secs(u64::from(settings.short_break_minutes) * 60),
            long_break_duration: Duration::from_secs(u64::from(settings.long_break_minutes) * 60),
            sessions_before_long_break: u32::from(settings.sessions_before_long_break),
        }
    }

    fn machine(configuration: PomodoroConfiguration) -> PomodoroMachine {
        PomodoroMachine::new(
            configuration.focus_duration,
            configuration.short_break_duration,
            configuration.long_break_duration,
            configuration.sessions_before_long_break,
        )
    }

    fn snapshot(machine: &PomodoroMachine, now: Instant, stopped_duration: Duration) -> TimerState {
        let remaining = machine.remaining(now).unwrap_or(stopped_duration);
        let remaining_seconds = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
        TimerState {
            phase: machine.phase(),
            remaining_seconds,
            completed_focus_sessions: machine.completed_focus_sessions(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> PomodoroSettings {
        PomodoroSettings {
            focus_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            sessions_before_long_break: 4,
        }
    }

    #[test]
    fn stopped_snapshot_uses_the_configured_focus_duration() {
        let service = PomodoroService::new(&settings());
        let (state, changed) = service
            .dispatch(PomodoroEvent::Tick, Instant::now())
            .unwrap();

        assert!(!changed);
        assert_eq!(state.phase, PomodoroPhase::Stopped);
        assert_eq!(state.remaining_seconds, 25 * 60);
    }

    #[test]
    fn start_and_pause_return_serializable_user_facing_state() {
        let service = PomodoroService::new(&settings());
        let now = Instant::now();
        let (started, changed) = service.dispatch(PomodoroEvent::Start, now).unwrap();
        assert!(changed);
        assert_eq!(started.phase, PomodoroPhase::Focus);

        let (paused, changed) = service
            .dispatch(PomodoroEvent::Pause, now + Duration::from_secs(60))
            .unwrap();
        assert!(changed);
        assert_eq!(paused.phase, PomodoroPhase::Paused);
        assert_eq!(paused.remaining_seconds, 24 * 60);
    }

    #[test]
    fn settings_changed_while_running_apply_after_stop() {
        let service = PomodoroService::new(&settings());
        let now = Instant::now();
        service.dispatch(PomodoroEvent::Start, now).unwrap();
        let mut changed = settings();
        changed.focus_minutes = 10;
        service.update_settings(&changed).unwrap();

        service.dispatch(PomodoroEvent::Stop, now).unwrap();
        let (stopped, _) = service.dispatch(PomodoroEvent::Tick, now).unwrap();
        assert_eq!(stopped.remaining_seconds, 10 * 60);
        let (started, _) = service.dispatch(PomodoroEvent::Start, now).unwrap();
        assert_eq!(started.remaining_seconds, 10 * 60);
    }

    #[test]
    fn elapsed_focus_reports_a_natural_completion() {
        let service = PomodoroService::new(&settings());
        let now = Instant::now();
        service.dispatch(PomodoroEvent::Start, now).unwrap();

        let (state, changed, completed) = service.tick(now + Duration::from_secs(25 * 60)).unwrap();

        assert!(changed);
        assert!(completed);
        assert_eq!(state.completed_focus_sessions, 1);
    }

    #[test]
    fn skipped_focus_is_not_a_natural_completion() {
        let service = PomodoroService::new(&settings());
        let now = Instant::now();
        service.dispatch(PomodoroEvent::Start, now).unwrap();

        let (_, changed, completed) = service.dispatch_internal(PomodoroEvent::Skip, now).unwrap();

        assert!(changed);
        assert!(!completed);
    }
}
