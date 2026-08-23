use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PomodoroPhase {
    Stopped,
    Focus,
    ShortBreak,
    LongBreak,
    Paused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PomodoroEvent {
    Start,
    Pause,
    Resume,
    Skip,
    Stop,
    Tick,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PomodoroEffect {
    NotifyPhase(PomodoroPhase),
    SetFocusGuardEnabled(bool),
    Persist,
}

#[derive(Clone, Copy, Debug)]
struct PausedState {
    phase: PomodoroPhase,
    remaining: Duration,
}

#[derive(Debug)]
pub struct PomodoroMachine {
    phase: PomodoroPhase,
    deadline: Option<Instant>,
    paused: Option<PausedState>,
    focus_duration: Duration,
    short_break_duration: Duration,
    long_break_duration: Duration,
    sessions_before_long_break: u32,
    completed_focus_sessions: u32,
}

impl PomodoroMachine {
    pub fn new(
        focus_duration: Duration,
        short_break_duration: Duration,
        long_break_duration: Duration,
        sessions_before_long_break: u32,
    ) -> Self {
        Self {
            phase: PomodoroPhase::Stopped,
            deadline: None,
            paused: None,
            focus_duration,
            short_break_duration,
            long_break_duration,
            sessions_before_long_break,
            completed_focus_sessions: 0,
        }
    }

    pub fn phase(&self) -> PomodoroPhase {
        self.phase
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub fn completed_focus_sessions(&self) -> u32 {
        self.completed_focus_sessions
    }

    pub fn remaining(&self, now: Instant) -> Option<Duration> {
        match self.phase {
            PomodoroPhase::Stopped => None,
            PomodoroPhase::Paused => self.paused.map(|paused| paused.remaining),
            _ => self
                .deadline
                .map(|deadline| deadline.saturating_duration_since(now)),
        }
    }

    pub fn reconfigure(
        &mut self,
        focus_duration: Duration,
        short_break_duration: Duration,
        long_break_duration: Duration,
        sessions_before_long_break: u32,
    ) -> bool {
        if self.phase != PomodoroPhase::Stopped {
            return false;
        }
        self.focus_duration = focus_duration;
        self.short_break_duration = short_break_duration;
        self.long_break_duration = long_break_duration;
        self.sessions_before_long_break = sessions_before_long_break;
        true
    }

    pub fn reduce(&mut self, event: PomodoroEvent, now: Instant) -> Vec<PomodoroEffect> {
        match event {
            PomodoroEvent::Start if self.phase == PomodoroPhase::Stopped => {
                self.enter_phase(PomodoroPhase::Focus, self.focus_duration, now)
            }
            PomodoroEvent::Pause if self.is_running() => {
                let remaining = self
                    .deadline
                    .take()
                    .map(|deadline| deadline.saturating_duration_since(now))
                    .unwrap_or_default();
                self.paused = Some(PausedState {
                    phase: self.phase,
                    remaining,
                });
                self.phase = PomodoroPhase::Paused;
                Self::phase_effects(PomodoroPhase::Paused)
            }
            PomodoroEvent::Resume if self.phase == PomodoroPhase::Paused => {
                let Some(paused) = self.paused.take() else {
                    return Vec::new();
                };
                self.enter_phase(paused.phase, paused.remaining, now)
            }
            PomodoroEvent::Skip if self.is_running() => self.advance(now),
            PomodoroEvent::Stop if self.phase != PomodoroPhase::Stopped => {
                self.phase = PomodoroPhase::Stopped;
                self.deadline = None;
                self.paused = None;
                Self::phase_effects(PomodoroPhase::Stopped)
            }
            PomodoroEvent::Tick
                if self.is_running() && self.deadline.is_some_and(|deadline| now >= deadline) =>
            {
                self.advance(now)
            }
            _ => Vec::new(),
        }
    }

    fn is_running(&self) -> bool {
        matches!(
            self.phase,
            PomodoroPhase::Focus | PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak
        )
    }

    fn advance(&mut self, now: Instant) -> Vec<PomodoroEffect> {
        match self.phase {
            PomodoroPhase::Focus => {
                self.completed_focus_sessions += 1;
                let long_break_due = self.sessions_before_long_break > 0
                    && self
                        .completed_focus_sessions
                        .is_multiple_of(self.sessions_before_long_break);
                if long_break_due {
                    self.enter_phase(PomodoroPhase::LongBreak, self.long_break_duration, now)
                } else {
                    self.enter_phase(PomodoroPhase::ShortBreak, self.short_break_duration, now)
                }
            }
            PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak => {
                self.enter_phase(PomodoroPhase::Focus, self.focus_duration, now)
            }
            _ => Vec::new(),
        }
    }

    fn enter_phase(
        &mut self,
        phase: PomodoroPhase,
        duration: Duration,
        now: Instant,
    ) -> Vec<PomodoroEffect> {
        self.phase = phase;
        self.deadline = Some(now + duration);
        self.paused = None;
        Self::phase_effects(phase)
    }

    fn phase_effects(phase: PomodoroPhase) -> Vec<PomodoroEffect> {
        vec![
            PomodoroEffect::NotifyPhase(phase),
            PomodoroEffect::SetFocusGuardEnabled(phase == PomodoroPhase::Focus),
            PomodoroEffect::Persist,
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    fn machine() -> PomodoroMachine {
        PomodoroMachine::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
    }

    #[test]
    fn start_and_elapsed_focus_transition_to_a_timed_short_break() {
        let now = Instant::now();
        let mut machine = machine();

        assert_eq!(machine.phase(), PomodoroPhase::Stopped);
        assert_eq!(
            machine.reduce(PomodoroEvent::Start, now),
            vec![
                PomodoroEffect::NotifyPhase(PomodoroPhase::Focus),
                PomodoroEffect::SetFocusGuardEnabled(true),
                PomodoroEffect::Persist,
            ]
        );
        assert_eq!(machine.phase(), PomodoroPhase::Focus);
        assert_eq!(machine.deadline(), Some(now + Duration::from_secs(25 * 60)));

        let focus_deadline = machine.deadline().unwrap();
        assert_eq!(
            machine.reduce(PomodoroEvent::Tick, focus_deadline),
            vec![
                PomodoroEffect::NotifyPhase(PomodoroPhase::ShortBreak),
                PomodoroEffect::SetFocusGuardEnabled(false),
                PomodoroEffect::Persist,
            ]
        );
        assert_eq!(machine.phase(), PomodoroPhase::ShortBreak);
        assert_eq!(
            machine.deadline(),
            Some(focus_deadline + Duration::from_secs(5 * 60))
        );
    }

    #[test]
    fn fourth_completed_focus_transitions_to_long_break() {
        let now = Instant::now();
        let mut machine = machine();

        machine.reduce(PomodoroEvent::Start, now);
        for _ in 0..3 {
            machine.reduce(PomodoroEvent::Skip, now);
            assert_eq!(machine.phase(), PomodoroPhase::ShortBreak);
            machine.reduce(PomodoroEvent::Skip, now);
            assert_eq!(machine.phase(), PomodoroPhase::Focus);
        }

        machine.reduce(PomodoroEvent::Skip, now);

        assert_eq!(machine.phase(), PomodoroPhase::LongBreak);
        assert_eq!(machine.completed_focus_sessions(), 4);
        assert_eq!(machine.deadline(), Some(now + Duration::from_secs(15 * 60)));
    }

    #[test]
    fn pause_and_resume_preserve_focus_remaining_duration() {
        let now = Instant::now();
        let mut machine = machine();
        machine.reduce(PomodoroEvent::Start, now);

        let paused_at = now + Duration::from_secs(5 * 60);
        assert_eq!(
            machine.reduce(PomodoroEvent::Pause, paused_at),
            vec![
                PomodoroEffect::NotifyPhase(PomodoroPhase::Paused),
                PomodoroEffect::SetFocusGuardEnabled(false),
                PomodoroEffect::Persist,
            ]
        );
        assert_eq!(machine.phase(), PomodoroPhase::Paused);
        assert_eq!(
            machine.remaining(paused_at),
            Some(Duration::from_secs(20 * 60))
        );
        assert_eq!(machine.deadline(), None);

        let resumed_at = now + Duration::from_secs(10 * 60);
        assert_eq!(
            machine.reduce(PomodoroEvent::Resume, resumed_at),
            vec![
                PomodoroEffect::NotifyPhase(PomodoroPhase::Focus),
                PomodoroEffect::SetFocusGuardEnabled(true),
                PomodoroEffect::Persist,
            ]
        );
        assert_eq!(machine.phase(), PomodoroPhase::Focus);
        assert_eq!(
            machine.deadline(),
            Some(resumed_at + Duration::from_secs(20 * 60))
        );
    }

    #[test]
    fn skip_advances_the_current_phase_and_sets_a_new_deadline() {
        let now = Instant::now();
        let mut machine = machine();
        machine.reduce(PomodoroEvent::Start, now);

        let skipped_at = now + Duration::from_secs(60);
        machine.reduce(PomodoroEvent::Skip, skipped_at);
        assert_eq!(machine.phase(), PomodoroPhase::ShortBreak);
        assert_eq!(
            machine.deadline(),
            Some(skipped_at + Duration::from_secs(5 * 60))
        );

        machine.reduce(PomodoroEvent::Skip, skipped_at);
        assert_eq!(machine.phase(), PomodoroPhase::Focus);
        assert_eq!(
            machine.deadline(),
            Some(skipped_at + Duration::from_secs(25 * 60))
        );
    }

    #[test]
    fn stop_clears_timing_and_disables_focus_guard() {
        let now = Instant::now();
        let mut machine = machine();
        machine.reduce(PomodoroEvent::Start, now);

        assert_eq!(
            machine.reduce(PomodoroEvent::Stop, now),
            vec![
                PomodoroEffect::NotifyPhase(PomodoroPhase::Stopped),
                PomodoroEffect::SetFocusGuardEnabled(false),
                PomodoroEffect::Persist,
            ]
        );
        assert_eq!(machine.phase(), PomodoroPhase::Stopped);
        assert_eq!(machine.deadline(), None);
        assert_eq!(machine.remaining(now), None);
        assert!(machine
            .reduce(PomodoroEvent::Tick, now + Duration::from_secs(60 * 60))
            .is_empty());
    }

    #[test]
    fn tick_after_sleep_completes_from_stored_deadline_only_once() {
        let now = Instant::now();
        let mut machine = machine();
        machine.reduce(PomodoroEvent::Start, now);

        let woke_at = now + Duration::from_secs(8 * 60 * 60);
        let effects = machine.reduce(PomodoroEvent::Tick, woke_at);

        assert_eq!(machine.phase(), PomodoroPhase::ShortBreak);
        assert_eq!(
            machine.deadline(),
            Some(woke_at + Duration::from_secs(5 * 60))
        );
        assert_eq!(
            effects,
            vec![
                PomodoroEffect::NotifyPhase(PomodoroPhase::ShortBreak),
                PomodoroEffect::SetFocusGuardEnabled(false),
                PomodoroEffect::Persist,
            ]
        );
        assert!(machine.reduce(PomodoroEvent::Tick, woke_at).is_empty());
    }

    #[test]
    fn settings_can_only_reconfigure_a_stopped_timer() {
        let now = Instant::now();
        let mut machine = machine();
        assert!(machine.reconfigure(
            Duration::from_secs(10 * 60),
            Duration::from_secs(2 * 60),
            Duration::from_secs(20 * 60),
            3,
        ));
        machine.reduce(PomodoroEvent::Start, now);
        assert_eq!(machine.deadline(), Some(now + Duration::from_secs(10 * 60)));
        assert!(!machine.reconfigure(
            Duration::from_secs(99 * 60),
            Duration::from_secs(2 * 60),
            Duration::from_secs(20 * 60),
            3,
        ));
    }
}
