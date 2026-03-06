// crates/sup/src/tui/editor.rs
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct InlineEditor {
    pub content: String,
    pub cursor: usize,
    pub node_id: Option<String>, // None = new node, Some = editing existing
}

pub enum EditorResult {
    Continue,
    #[allow(dead_code)]
    Commit(String),
    Cancel,
}

impl InlineEditor {
    #[allow(dead_code)]
    pub fn new_empty() -> Self {
        Self { content: String::new(), cursor: 0, node_id: None }
    }

    pub fn edit_existing(node_id: String, content: &str) -> Self {
        let len = content.len();
        Self { content: content.to_string(), cursor: len, node_id: Some(node_id) }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> EditorResult {
        match key.code {
            KeyCode::Enter => EditorResult::Commit(self.content.clone()),
            KeyCode::Esc => EditorResult::Cancel,
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.content.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                EditorResult::Continue
            }
            KeyCode::Backspace if self.cursor > 0 => {
                // find the previous character boundary
                let prev = self.content[..self.cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                self.content.drain(prev..self.cursor);
                self.cursor = prev;
                EditorResult::Continue
            }
            KeyCode::Left if self.cursor > 0 => {
                let prev = self.content[..self.cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                self.cursor = prev;
                EditorResult::Continue
            }
            KeyCode::Right if self.cursor < self.content.len() => {
                let next = self.content[self.cursor..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| self.cursor + i)
                    .unwrap_or(self.content.len());
                self.cursor = next;
                EditorResult::Continue
            }
            _ => EditorResult::Continue,
        }
    }
}
