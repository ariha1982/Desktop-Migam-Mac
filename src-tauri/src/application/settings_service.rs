use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::domain::settings::Settings;

pub struct SettingsService {
    settings_path: PathBuf,
}

impl SettingsService {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            settings_path: app_data_dir.join("settings.json"),
        }
    }

    pub fn load_or_default(&self) -> Settings {
        match self.load() {
            Ok(settings) => settings,
            Err(_) => {
                self.preserve_corrupt_file();
                Settings::default()
            }
        }
    }

    pub fn save(&self, settings: &Settings) -> Result<(), String> {
        let parent = self
            .settings_path
            .parent()
            .ok_or_else(|| "settings path has no parent directory".to_owned())?;
        fs::create_dir_all(parent).map_err(Self::safe_io_error)?;
        let bytes = serde_json::to_vec_pretty(settings)
            .map_err(|_| "settings could not be serialized".to_owned())?;
        let temporary = self.settings_path.with_extension("json.tmp");
        let previous = self.settings_path.with_extension("json.previous");
        fs::write(&temporary, bytes).map_err(Self::safe_io_error)?;

        if self.settings_path.exists() {
            let _ = fs::remove_file(&previous);
            fs::rename(&self.settings_path, &previous).map_err(Self::safe_io_error)?;
        }
        if let Err(error) = fs::rename(&temporary, &self.settings_path) {
            if previous.exists() {
                let _ = fs::rename(&previous, &self.settings_path);
            }
            return Err(Self::safe_io_error(error));
        }
        let _ = fs::remove_file(previous);
        Ok(())
    }

    fn load(&self) -> Result<Settings, String> {
        if !self.settings_path.exists() {
            return Ok(Settings::default());
        }
        let json = fs::read_to_string(&self.settings_path).map_err(Self::safe_io_error)?;
        Settings::from_json(&json).map_err(|_| "settings file is invalid".to_owned())
    }

    fn preserve_corrupt_file(&self) {
        if !self.settings_path.exists() {
            return;
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        let corrupt = self
            .settings_path
            .with_file_name(format!("settings.corrupt-{timestamp}.json"));
        let _ = fs::rename(&self.settings_path, corrupt);
    }

    fn safe_io_error(_error: io::Error) -> String {
        "settings storage operation failed".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("desktop-pet-mvp-{name}-{}", std::process::id()))
    }

    #[test]
    fn saves_and_loads_settings() {
        let directory = temporary_directory("save-load");
        let _ = fs::remove_dir_all(&directory);
        let service = SettingsService::new(directory.clone());
        let mut expected = Settings::default();
        expected.pet.visual_scale_percent = 125;

        service.save(&expected).unwrap();
        assert_eq!(service.load_or_default(), expected);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn preserves_corrupt_json_and_returns_safe_defaults() {
        let directory = temporary_directory("recovery");
        let _ = fs::remove_dir_all(&directory);
        let service = SettingsService::new(directory.clone());
        fs::create_dir_all(&directory).unwrap();
        fs::write(&service.settings_path, "not json").unwrap();

        assert_eq!(service.load_or_default(), Settings::default());
        assert!(!service.settings_path.exists());
        assert!(fs::read_dir(&directory).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("settings.corrupt-")));
        let _ = fs::remove_dir_all(directory);
    }
}
