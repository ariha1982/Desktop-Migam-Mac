use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistractionRule {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub process_name: Option<String>,
    #[serde(default)]
    pub window_title: Option<String>,
    #[serde(default = "default_grace_seconds")]
    pub grace_seconds: u32,
    #[serde(default = "default_cooldown_seconds")]
    pub cooldown_seconds: u32,
}

const fn default_enabled() -> bool {
    true
}

const fn default_grace_seconds() -> u32 {
    5
}

const fn default_cooldown_seconds() -> u32 {
    30
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum DistractionRuleError {
    #[error("a distraction rule requires a process name or window title")]
    MissingConditions,
    #[error("a distraction rule process must be a file name, not a path")]
    ProcessNameIsPath,
    #[error("grace seconds must be between 5 and 600")]
    GraceOutOfRange,
    #[error("cooldown seconds must be between 30 and 3600")]
    CooldownOutOfRange,
}

impl DistractionRule {
    pub fn validate(&self) -> Result<(), DistractionRuleError> {
        if self.process_condition().is_none() && self.title_condition().is_none() {
            return Err(DistractionRuleError::MissingConditions);
        }
        if self
            .process_condition()
            .is_some_and(|value| value.contains(['\\', '/']))
        {
            return Err(DistractionRuleError::ProcessNameIsPath);
        }
        if !(5..=600).contains(&self.grace_seconds) {
            return Err(DistractionRuleError::GraceOutOfRange);
        }
        if !(30..=3600).contains(&self.cooldown_seconds) {
            return Err(DistractionRuleError::CooldownOutOfRange);
        }
        Ok(())
    }

    pub fn matches(&self, process_name: &str, window_title: &str) -> bool {
        if self.validate().is_err() {
            return false;
        }

        let process_matches = self
            .process_condition()
            .is_none_or(|expected| expected.eq_ignore_ascii_case(process_name));
        let title_matches = self.title_condition().is_none_or(|expected| {
            window_title
                .to_lowercase()
                .contains(&expected.to_lowercase())
        });

        process_matches && title_matches
    }

    fn process_condition(&self) -> Option<&str> {
        Self::populated(self.process_name.as_deref())
    }

    fn title_condition(&self) -> Option<&str> {
        Self::populated(self.window_title.as_deref())
    }

    fn populated(value: Option<&str>) -> Option<&str> {
        value.map(str::trim).filter(|value| !value.is_empty())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn rule(process_name: Option<&str>, window_title: Option<&str>) -> DistractionRule {
        DistractionRule {
            id: "rule-1".to_owned(),
            name: "Test rule".to_owned(),
            enabled: true,
            process_name: process_name.map(str::to_owned),
            window_title: window_title.map(str::to_owned),
            grace_seconds: 5,
            cooldown_seconds: 30,
        }
    }

    #[test]
    fn process_name_matches_case_insensitively_but_not_as_a_substring() {
        let rule = rule(Some("Chrome.EXE"), None);

        assert!(rule.matches("chrome.exe", "Any title"));
        assert!(!rule.matches("my-chrome.exe", "Any title"));
    }

    #[test]
    fn window_title_matches_a_case_insensitive_substring() {
        let rule = rule(None, Some("YouTube"));

        assert!(rule.matches("msedge.exe", "music - YOUTUBE"));
        assert!(!rule.matches("msedge.exe", "Documentation"));
    }

    #[test]
    fn all_populated_conditions_must_match() {
        let rule = rule(Some("chrome.exe"), Some("YouTube"));

        assert!(rule.matches("CHROME.EXE", "Video - youtube"));
        assert!(!rule.matches("firefox.exe", "Video - YouTube"));
        assert!(!rule.matches("chrome.exe", "Mail"));
    }

    #[test]
    fn a_rule_with_neither_populated_condition_is_rejected() {
        let empty = rule(None, None);
        let whitespace = rule(Some("  "), Some("\t"));

        assert_eq!(
            empty.validate(),
            Err(DistractionRuleError::MissingConditions)
        );
        assert_eq!(
            whitespace.validate(),
            Err(DistractionRuleError::MissingConditions)
        );
        assert!(!empty.matches("chrome.exe", "YouTube"));
    }

    #[test]
    fn unsafe_process_paths_and_timing_ranges_are_rejected() {
        let mut unsafe_path = rule(Some("C:\\Apps\\browser.exe"), None);
        assert_eq!(
            unsafe_path.validate(),
            Err(DistractionRuleError::ProcessNameIsPath)
        );

        unsafe_path.process_name = Some("browser.exe".to_owned());
        unsafe_path.grace_seconds = 4;
        assert_eq!(
            unsafe_path.validate(),
            Err(DistractionRuleError::GraceOutOfRange)
        );

        unsafe_path.grace_seconds = 5;
        unsafe_path.cooldown_seconds = 29;
        assert_eq!(
            unsafe_path.validate(),
            Err(DistractionRuleError::CooldownOutOfRange)
        );
    }
}
