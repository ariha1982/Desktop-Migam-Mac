use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::domain::todo::TodoState;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusTodoLink {
    pub todo_id: String,
    pub title_at_start: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoSnapshot {
    #[serde(flatten)]
    pub state: TodoState,
    pub active_focus_todo: Option<FocusTodoLink>,
    pub pending_focus_todo: Option<FocusTodoLink>,
}

pub struct TodoService {
    path: PathBuf,
    state: Mutex<TodoState>,
    active_focus: Mutex<Option<FocusTodoLink>>,
    pending_focus: Mutex<Option<FocusTodoLink>>,
    next_id: AtomicU64,
}

impl TodoService {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let path = app_data_dir.join("todo.json");
        let state = Self::load(&path).unwrap_or_else(|_| {
            Self::preserve_corrupt_file(&path);
            TodoState::default()
        });
        Self {
            path,
            state: Mutex::new(state),
            active_focus: Mutex::new(None),
            pending_focus: Mutex::new(None),
            next_id: AtomicU64::new(now_millis() as u64),
        }
    }

    pub fn snapshot(&self) -> Result<TodoSnapshot, String> {
        Ok(TodoSnapshot {
            state: self.state.lock().map_err(|_| unavailable())?.clone(),
            active_focus_todo: self.active_focus.lock().map_err(|_| unavailable())?.clone(),
            pending_focus_todo: self
                .pending_focus
                .lock()
                .map_err(|_| unavailable())?
                .clone(),
        })
    }

    pub fn add(&self, text: &str) -> Result<TodoSnapshot, String> {
        let id = format!("todo-{}", self.next_id.fetch_add(1, Ordering::SeqCst));
        let (snapshot, _) = self.change(|state| state.add(id, text, iso_now()).map(|_| false))?;
        Ok(snapshot)
    }

    pub fn update(&self, id: &str, text: &str) -> Result<TodoSnapshot, String> {
        let (snapshot, _) = self.change(|state| state.update(id, text).map(|_| false))?;
        Ok(snapshot)
    }

    pub fn set_completed(&self, id: &str, completed: bool) -> Result<(TodoSnapshot, bool), String> {
        self.change(|state| state.set_completed(id, completed, iso_now()))
    }

    pub fn select(&self, id: Option<String>) -> Result<TodoSnapshot, String> {
        let (snapshot, _) = self.change(|state| state.select(id).map(|_| false))?;
        Ok(snapshot)
    }

    pub fn delete(&self, id: &str) -> Result<TodoSnapshot, String> {
        self.change(|state| state.delete(id).map(|_| false))?;
        if self
            .active_focus
            .lock()
            .map_err(|_| unavailable())?
            .as_ref()
            .map(|link| link.todo_id.as_str())
            == Some(id)
        {
            *self.active_focus.lock().map_err(|_| unavailable())? = None;
        }
        if self
            .pending_focus
            .lock()
            .map_err(|_| unavailable())?
            .as_ref()
            .map(|link| link.todo_id.as_str())
            == Some(id)
        {
            *self.pending_focus.lock().map_err(|_| unavailable())? = None;
        }
        self.snapshot()
    }

    pub fn begin_focus(&self) -> Result<TodoSnapshot, String> {
        let state = self.state.lock().map_err(|_| unavailable())?;
        let link = state.selected_todo_id.as_ref().and_then(|id| {
            state
                .items
                .iter()
                .find(|item| &item.id == id && item.completed_at.is_none())
                .map(|item| FocusTodoLink {
                    todo_id: item.id.clone(),
                    title_at_start: item.text.clone(),
                })
        });
        drop(state);
        *self.active_focus.lock().map_err(|_| unavailable())? = link;
        *self.pending_focus.lock().map_err(|_| unavailable())? = None;
        self.snapshot()
    }

    pub fn finish_focus(&self) -> Result<TodoSnapshot, String> {
        let active = self.active_focus.lock().map_err(|_| unavailable())?.take();
        let state = self.state.lock().map_err(|_| unavailable())?;
        let pending = active.filter(|link| {
            state
                .items
                .iter()
                .any(|item| item.id == link.todo_id && item.completed_at.is_none())
        });
        drop(state);
        *self.pending_focus.lock().map_err(|_| unavailable())? = pending;
        self.snapshot()
    }

    pub fn cancel_focus(&self) -> Result<TodoSnapshot, String> {
        *self.active_focus.lock().map_err(|_| unavailable())? = None;
        *self.pending_focus.lock().map_err(|_| unavailable())? = None;
        self.snapshot()
    }

    pub fn resolve_focus(&self, complete: bool) -> Result<(TodoSnapshot, bool), String> {
        let pending = self.pending_focus.lock().map_err(|_| unavailable())?.take();
        if complete {
            if let Some(link) = pending {
                return self.set_completed(&link.todo_id, true);
            }
        }
        Ok((self.snapshot()?, false))
    }

    fn change<F>(&self, action: F) -> Result<(TodoSnapshot, bool), String>
    where
        F: FnOnce(&mut TodoState) -> Result<bool, String>,
    {
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        let celebrated = action(&mut state)?;
        self.save(&state)?;
        drop(state);
        Ok((self.snapshot()?, celebrated))
    }

    fn save(&self, state: &TodoState) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "todo path has no parent".to_owned())?;
        fs::create_dir_all(parent).map_err(safe_io_error)?;
        let temporary = self.path.with_extension("json.tmp");
        let previous = self.path.with_extension("json.previous");
        fs::write(
            &temporary,
            serde_json::to_vec_pretty(state).map_err(|_| "todo serialization failed".to_owned())?,
        )
        .map_err(safe_io_error)?;
        if self.path.exists() {
            let _ = fs::remove_file(&previous);
            fs::rename(&self.path, &previous).map_err(safe_io_error)?;
        }
        if let Err(error) = fs::rename(&temporary, &self.path) {
            if previous.exists() {
                let _ = fs::rename(&previous, &self.path);
            }
            return Err(safe_io_error(error));
        }
        let _ = fs::remove_file(previous);
        Ok(())
    }

    fn load(path: &Path) -> Result<TodoState, String> {
        if !path.exists() {
            return Ok(TodoState::default());
        }
        serde_json::from_str(&fs::read_to_string(path).map_err(safe_io_error)?)
            .map_err(|_| "todo file is invalid".to_owned())
    }

    fn preserve_corrupt_file(path: &Path) {
        if !path.exists() {
            return;
        }
        let corrupt = path.with_file_name(format!("todo.corrupt-{}.json", now_millis()));
        let _ = fs::rename(path, corrupt);
    }
}

fn unavailable() -> String {
    "todo state is unavailable".to_owned()
}
fn safe_io_error(_error: io::Error) -> String {
    "todo storage operation failed".to_owned()
}
fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or_default()
}
fn iso_now() -> String {
    let millis = now_millis() as i64;
    let seconds = millis / 1_000;
    let days = seconds / 86_400;
    let seconds_of_day = seconds % 86_400;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let hour = seconds_of_day / 3_600;
    let minute = seconds_of_day % 3_600 / 60;
    let second = seconds_of_day % 60;
    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{:03}Z",
        millis % 1_000
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("desktop-pet-todo-{name}-{}", std::process::id()))
    }

    #[test]
    fn changes_survive_restart_and_focus_link_uses_title_snapshot() {
        let directory = directory("persistence");
        let _ = fs::remove_dir_all(&directory);
        let service = TodoService::new(directory.clone());
        let added = service.add("문서 작성").unwrap();
        let id = added.state.items[0].id.clone();
        service.select(Some(id.clone())).unwrap();
        assert_eq!(
            service
                .begin_focus()
                .unwrap()
                .active_focus_todo
                .unwrap()
                .title_at_start,
            "문서 작성"
        );
        service.update(&id, "문서 검토").unwrap();
        assert_eq!(
            service
                .finish_focus()
                .unwrap()
                .pending_focus_todo
                .unwrap()
                .title_at_start,
            "문서 작성"
        );
        assert_eq!(
            TodoService::new(directory.clone())
                .snapshot()
                .unwrap()
                .state
                .items[0]
                .text,
            "문서 검토"
        );
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn corrupt_data_is_preserved() {
        let directory = directory("corrupt");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("todo.json"), "bad").unwrap();
        assert!(TodoService::new(directory.clone())
            .snapshot()
            .unwrap()
            .state
            .items
            .is_empty());
        assert!(fs::read_dir(&directory).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("todo.corrupt-")));
        let _ = fs::remove_dir_all(directory);
    }
}
