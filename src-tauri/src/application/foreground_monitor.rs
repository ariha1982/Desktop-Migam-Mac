use std::{collections::HashMap, sync::Mutex, time::Instant};

use serde::Serialize;

use crate::domain::{
    foreground::{ForegroundWindowSource, WindowMinimizer, WindowSnapshot},
    settings::FocusGuardSettings,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionState {
    pub matched: bool,
    pub rule_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterventionRequest {
    pub intervention_id: u64,
    pub start_x: i32,
    pub impact_x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForegroundEffect {
    Detection(DetectionState),
    Start(InterventionRequest),
    Cancel(u64),
}

#[derive(Clone)]
struct Candidate {
    window_id: isize,
    rule_id: String,
    since: Instant,
}

#[derive(Clone)]
struct PendingIntervention {
    id: u64,
    window_id: isize,
    rule_id: String,
    cooldown_seconds: u32,
}

#[derive(Default)]
struct MonitorRuntime {
    detection: DetectionState,
    candidate: Option<Candidate>,
    pending: Option<PendingIntervention>,
    cooldowns: HashMap<(isize, String), Instant>,
    next_id: u64,
}

pub struct ForegroundMonitor {
    source: Box<dyn ForegroundWindowSource>,
    minimizer: Box<dyn WindowMinimizer>,
    runtime: Mutex<MonitorRuntime>,
    application_process_id: u32,
}

impl ForegroundMonitor {
    pub fn new(
        source: Box<dyn ForegroundWindowSource>,
        minimizer: Box<dyn WindowMinimizer>,
        application_process_id: u32,
    ) -> Self {
        Self {
            source,
            minimizer,
            runtime: Mutex::new(MonitorRuntime::default()),
            application_process_id,
        }
    }

    pub fn state(&self) -> Result<DetectionState, String> {
        self.runtime
            .lock()
            .map(|runtime| runtime.detection.clone())
            .map_err(|_| "foreground detection state is unavailable".to_owned())
    }

    pub fn poll(
        &self,
        now: Instant,
        focus_running: bool,
        emergency_stopped: bool,
        settings: &FocusGuardSettings,
    ) -> Result<Vec<ForegroundEffect>, String> {
        let active = focus_running && !emergency_stopped && settings.intervention_enabled;
        let snapshot = if active {
            self.source.foreground_window().ok().flatten()
        } else {
            None
        };
        let matched = snapshot.as_ref().and_then(|snapshot| {
            if self.is_protected(snapshot) {
                return None;
            }
            let process_name = snapshot.process_name.as_deref().unwrap_or_default();
            let title = snapshot.title.as_deref().unwrap_or_default();
            settings
                .rules
                .iter()
                .find(|rule| rule.enabled && rule.matches(process_name, title))
                .map(|rule| (rule.id.clone(), rule.grace_seconds, rule.cooldown_seconds))
        });

        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "foreground detection state is unavailable".to_owned())?;
        let mut effects = Vec::new();
        let next_detection = DetectionState {
            matched: matched.is_some(),
            rule_id: matched.as_ref().map(|(id, _, _)| id.clone()),
        };
        if runtime.detection != next_detection {
            runtime.detection = next_detection.clone();
            effects.push(ForegroundEffect::Detection(next_detection));
        }

        let Some(snapshot) = snapshot else {
            Self::cancel_pending(&mut runtime, &mut effects);
            runtime.candidate = None;
            return Ok(effects);
        };
        let Some((rule_id, grace_seconds, cooldown_seconds)) = matched else {
            Self::cancel_pending(&mut runtime, &mut effects);
            runtime.candidate = None;
            return Ok(effects);
        };

        if let Some(pending) = runtime.pending.as_ref() {
            if pending.window_id == snapshot.window_id && pending.rule_id == rule_id {
                return Ok(effects);
            }
            Self::cancel_pending(&mut runtime, &mut effects);
        }

        let cooldown_key = (snapshot.window_id, rule_id.clone());
        if runtime
            .cooldowns
            .get(&cooldown_key)
            .is_some_and(|until| *until > now)
        {
            runtime.candidate = None;
            return Ok(effects);
        }
        runtime.cooldowns.retain(|_, until| *until > now);

        let candidate_ready = runtime.candidate.as_ref().is_some_and(|candidate| {
            candidate.window_id == snapshot.window_id
                && candidate.rule_id == rule_id
                && now.duration_since(candidate.since).as_secs() >= u64::from(grace_seconds)
        });
        if !candidate_ready {
            let same_candidate = runtime.candidate.as_ref().is_some_and(|candidate| {
                candidate.window_id == snapshot.window_id && candidate.rule_id == rule_id
            });
            if !same_candidate {
                runtime.candidate = Some(Candidate {
                    window_id: snapshot.window_id,
                    rule_id,
                    since: now,
                });
            }
            return Ok(effects);
        }

        runtime.next_id = runtime.next_id.wrapping_add(1).max(1);
        let intervention_id = runtime.next_id;
        runtime.pending = Some(PendingIntervention {
            id: intervention_id,
            window_id: snapshot.window_id,
            rule_id,
            cooldown_seconds,
        });
        runtime.candidate = None;
        let impact_x = snapshot.x + (snapshot.width.saturating_sub(220) / 2) as i32;
        let y = snapshot.y + (snapshot.height.saturating_sub(180) / 2) as i32;
        effects.push(ForegroundEffect::Start(InterventionRequest {
            intervention_id,
            start_x: snapshot.monitor_left - 200,
            impact_x,
            y,
        }));
        Ok(effects)
    }

    pub fn complete(
        &self,
        intervention_id: u64,
        now: Instant,
        focus_running: bool,
        emergency_stopped: bool,
        settings: &FocusGuardSettings,
    ) -> Result<bool, String> {
        if !focus_running || emergency_stopped || !settings.intervention_enabled {
            self.cancel(intervention_id)?;
            return Ok(false);
        }
        let pending = self
            .runtime
            .lock()
            .map_err(|_| "foreground intervention state is unavailable".to_owned())?
            .pending
            .clone();
        let Some(pending) = pending.filter(|pending| pending.id == intervention_id) else {
            return Ok(false);
        };
        let fresh = self.source.foreground_window().ok().flatten();
        let still_matches = fresh.as_ref().is_some_and(|snapshot| {
            if snapshot.window_id != pending.window_id || self.is_protected(snapshot) {
                return false;
            }
            let process_name = snapshot.process_name.as_deref().unwrap_or_default();
            let title = snapshot.title.as_deref().unwrap_or_default();
            settings.rules.iter().any(|rule| {
                rule.enabled && rule.id == pending.rule_id && rule.matches(process_name, title)
            })
        });
        if !still_matches {
            self.cancel(intervention_id)?;
            return Ok(false);
        }

        let minimized = self.minimizer.minimize(pending.window_id).is_ok();
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "foreground intervention state is unavailable".to_owned())?;
        if runtime
            .pending
            .as_ref()
            .is_some_and(|value| value.id == intervention_id)
        {
            runtime.cooldowns.insert(
                (pending.window_id, pending.rule_id),
                now + std::time::Duration::from_secs(u64::from(pending.cooldown_seconds)),
            );
            runtime.pending = None;
            runtime.candidate = None;
        }
        Ok(minimized)
    }

    pub fn cancel(&self, intervention_id: u64) -> Result<bool, String> {
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "foreground intervention state is unavailable".to_owned())?;
        if runtime
            .pending
            .as_ref()
            .is_some_and(|pending| pending.id == intervention_id)
        {
            runtime.pending = None;
            runtime.candidate = None;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn cancel_all(&self) -> Result<Option<u64>, String> {
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "foreground intervention state is unavailable".to_owned())?;
        runtime.candidate = None;
        runtime.detection = DetectionState::default();
        Ok(runtime.pending.take().map(|pending| pending.id))
    }

    fn cancel_pending(runtime: &mut MonitorRuntime, effects: &mut Vec<ForegroundEffect>) {
        if let Some(pending) = runtime.pending.take() {
            effects.push(ForegroundEffect::Cancel(pending.id));
        }
    }

    fn is_protected(&self, snapshot: &WindowSnapshot) -> bool {
        const PROTECTED: &[&str] = &[
            "desktop-pet-mvp.exe",
            "Desktop Migam Mac",
            "Dock",
            "Finder",
            "loginwindow",
            "System Settings",
            "SystemUIServer",
            "WindowServer",
            "taskmgr.exe",
            "explorer.exe",
            "dwm.exe",
            "winlogon.exe",
            "logonui.exe",
            "credentialui.exe",
            "mstsc.exe",
            "msra.exe",
        ];
        snapshot.window_id == 0
            || snapshot.process_id == self.application_process_id
            || !snapshot.is_visible
            || snapshot.is_minimized
            || snapshot.is_fullscreen
            || snapshot.process_name.as_ref().is_none_or(|name| {
                PROTECTED
                    .iter()
                    .any(|protected| protected.eq_ignore_ascii_case(name))
            })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::domain::{
        distraction::DistractionRule,
        foreground::{ForegroundReadError, WindowSnapshot},
    };

    use super::*;

    struct FakeSource {
        calls: Arc<AtomicUsize>,
        snapshot: Option<WindowSnapshot>,
    }

    impl ForegroundWindowSource for FakeSource {
        fn foreground_window(&self) -> Result<Option<WindowSnapshot>, ForegroundReadError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.snapshot.clone())
        }
    }

    struct FakeMinimizer(Arc<AtomicUsize>);

    impl WindowMinimizer for FakeMinimizer {
        fn minimize(&self, _window_id: isize) -> Result<(), ForegroundReadError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn settings() -> FocusGuardSettings {
        FocusGuardSettings {
            intervention_enabled: true,
            rules: vec![DistractionRule {
                id: "youtube".to_owned(),
                name: "YouTube".to_owned(),
                enabled: true,
                process_name: Some("chrome.exe".to_owned()),
                window_title: Some("youtube".to_owned()),
                grace_seconds: 5,
                cooldown_seconds: 30,
            }],
        }
    }

    fn snapshot(process: &str, title: &str) -> WindowSnapshot {
        WindowSnapshot {
            window_id: 42,
            process_id: 100,
            process_name: Some(process.to_owned()),
            title: Some(title.to_owned()),
            is_visible: true,
            is_minimized: false,
            is_fullscreen: false,
            monitor_left: 0,
            x: 600,
            y: 100,
            width: 800,
            height: 600,
        }
    }

    fn monitor(calls: Arc<AtomicUsize>, minimized: Arc<AtomicUsize>) -> ForegroundMonitor {
        ForegroundMonitor::new(
            Box::new(FakeSource {
                calls,
                snapshot: Some(snapshot("chrome.exe", "Music - YouTube")),
            }),
            Box::new(FakeMinimizer(minimized)),
            999,
        )
    }

    #[test]
    fn does_not_read_the_foreground_window_outside_focus() {
        let calls = Arc::new(AtomicUsize::new(0));
        let monitor = monitor(calls.clone(), Arc::new(AtomicUsize::new(0)));

        assert!(monitor
            .poll(Instant::now(), false, false, &settings())
            .unwrap()
            .is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn starts_after_grace_and_minimizes_only_after_fresh_revalidation() {
        let calls = Arc::new(AtomicUsize::new(0));
        let minimized = Arc::new(AtomicUsize::new(0));
        let monitor = monitor(calls.clone(), minimized.clone());
        let now = Instant::now();
        let first = monitor.poll(now, true, false, &settings()).unwrap();
        assert!(matches!(first.as_slice(), [ForegroundEffect::Detection(_)]));

        let effects = monitor
            .poll(
                now + std::time::Duration::from_secs(5),
                true,
                false,
                &settings(),
            )
            .unwrap();
        let ForegroundEffect::Start(request) = &effects[0] else {
            panic!("expected an intervention request");
        };
        assert_eq!(minimized.load(Ordering::SeqCst), 0);
        assert!(monitor
            .complete(request.intervention_id, now, true, false, &settings())
            .unwrap());
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        assert_eq!(minimized.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn emergency_stop_cancels_a_pending_kick() {
        let monitor = monitor(Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
        let now = Instant::now();
        monitor.poll(now, true, false, &settings()).unwrap();
        monitor
            .poll(
                now + std::time::Duration::from_secs(5),
                true,
                false,
                &settings(),
            )
            .unwrap();

        let effects = monitor.poll(now, true, true, &settings()).unwrap();
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, ForegroundEffect::Cancel(_))));
    }
}
