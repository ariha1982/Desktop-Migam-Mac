use serde::{Deserialize, Serialize};

pub const TODO_SCHEMA_VERSION: u8 = 1;
pub const TODO_TEXT_LIMIT: usize = 200;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoState {
    pub schema_version: u8,
    pub items: Vec<TodoItem>,
    pub selected_todo_id: Option<String>,
    pub all_completed_celebrated: bool,
}

impl Default for TodoState {
    fn default() -> Self {
        Self {
            schema_version: TODO_SCHEMA_VERSION,
            items: Vec::new(),
            selected_todo_id: None,
            all_completed_celebrated: false,
        }
    }
}

impl TodoState {
    pub fn add(&mut self, id: String, text: &str, now: String) -> Result<(), String> {
        let text = normalize_text(text)?;
        self.items.push(TodoItem {
            id,
            text,
            created_at: now,
            completed_at: None,
        });
        self.all_completed_celebrated = false;
        Ok(())
    }

    pub fn update(&mut self, id: &str, text: &str) -> Result<(), String> {
        let text = normalize_text(text)?;
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| "todo item was not found".to_owned())?;
        item.text = text;
        Ok(())
    }

    pub fn set_completed(
        &mut self,
        id: &str,
        completed: bool,
        now: String,
    ) -> Result<bool, String> {
        let incomplete_before = self
            .items
            .iter()
            .filter(|item| item.completed_at.is_none())
            .count();
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| "todo item was not found".to_owned())?;
        if completed {
            item.completed_at = Some(now);
        } else {
            item.completed_at = None;
            self.all_completed_celebrated = false;
        }
        let all_completed = incomplete_before > 0
            && !self.items.is_empty()
            && self.items.iter().all(|item| item.completed_at.is_some())
            && !self.all_completed_celebrated;
        if all_completed {
            self.all_completed_celebrated = true;
        }
        Ok(all_completed)
    }

    pub fn select(&mut self, id: Option<String>) -> Result<(), String> {
        if let Some(id) = &id {
            let selectable = self
                .items
                .iter()
                .any(|item| &item.id == id && item.completed_at.is_none());
            if !selectable {
                return Err("only an incomplete todo can be selected".to_owned());
            }
        }
        self.selected_todo_id = id;
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let before = self.items.len();
        self.items.retain(|item| item.id != id);
        if self.items.len() == before {
            return Err("todo item was not found".to_owned());
        }
        if self.selected_todo_id.as_deref() == Some(id) {
            self.selected_todo_id = None;
        }
        Ok(())
    }
}

fn normalize_text(text: &str) -> Result<String, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("todo text cannot be empty".to_owned());
    }
    if text.chars().count() > TODO_TEXT_LIMIT {
        return Err(format!(
            "todo text cannot exceed {TODO_TEXT_LIMIT} characters"
        ));
    }
    Ok(text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(text: &str) -> TodoState {
        let mut state = TodoState::default();
        state
            .add("1".to_owned(), text, "created".to_owned())
            .unwrap();
        state
    }

    #[test]
    fn trims_and_validates_text() {
        let mut state = TodoState::default();
        state
            .add("1".to_owned(), "  할 일  ", "now".to_owned())
            .unwrap();
        assert_eq!(state.items[0].text, "할 일");
        assert!(state.add("2".to_owned(), "   ", "now".to_owned()).is_err());
        assert!(state
            .add("3".to_owned(), &"가".repeat(201), "now".to_owned())
            .is_err());
    }

    #[test]
    fn only_user_completion_of_the_last_item_celebrates() {
        let mut state = state_with("첫째");
        state
            .add("2".to_owned(), "둘째", "created".to_owned())
            .unwrap();
        assert!(!state.set_completed("1", true, "done".to_owned()).unwrap());
        assert!(state.set_completed("2", true, "done".to_owned()).unwrap());
        assert!(!state.set_completed("2", true, "done".to_owned()).unwrap());
        state
            .set_completed("1", false, "ignored".to_owned())
            .unwrap();
        assert!(state.set_completed("1", true, "done".to_owned()).unwrap());
    }

    #[test]
    fn deletion_never_triggers_completion_and_clears_selection() {
        let mut state = state_with("남은 일");
        state.select(Some("1".to_owned())).unwrap();
        state.delete("1").unwrap();
        assert!(state.items.is_empty());
        assert_eq!(state.selected_todo_id, None);
        assert!(!state.all_completed_celebrated);
    }
}
