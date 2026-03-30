use std::collections::VecDeque;
use std::path::PathBuf;

use crate::color::Color;
use crate::storage::ColorStore;

const MAX_STREAM_SIZE: usize = 200;

pub struct App {
    pub store: ColorStore,
    pub color_stream: VecDeque<Color>,
    pub current_color: Option<Color>,
    pub selected_index: Option<usize>,
    pub should_quit: bool,
    pub clear_confirm: bool,
    pub status_message: Option<String>,
    pub storage_path: PathBuf,
}

impl App {
    pub fn new(storage_path: PathBuf) -> Self {
        let store = ColorStore::load(&storage_path);
        Self {
            store,
            color_stream: VecDeque::new(),
            current_color: None,
            selected_index: None,
            should_quit: false,
            clear_confirm: false,
            status_message: None,
            storage_path,
        }
    }

    pub fn push_color(&mut self, color: Color) {
        if self.color_stream.front() == Some(&color) {
            self.current_color = Some(color);
            return;
        }
        self.current_color = Some(color.clone());
        self.color_stream.push_front(color);
        if self.color_stream.len() > MAX_STREAM_SIZE {
            self.color_stream.pop_back();
        }
    }

    pub fn save_current_color(&mut self) {
        if let Some(ref color) = self.current_color {
            self.store.add(color);
            let _ = self.store.save(&self.storage_path);
            self.set_status(&format!("Saved {}", color.to_hex()));
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some(idx) = self.selected_index {
            if idx < self.store.saved_colors.len() {
                self.store.remove(idx);
                let _ = self.store.save(&self.storage_path);
                if self.store.saved_colors.is_empty() {
                    self.selected_index = None;
                } else if idx >= self.store.saved_colors.len() {
                    self.selected_index = Some(self.store.saved_colors.len() - 1);
                }
            }
        }
    }

    pub fn request_clear(&mut self) {
        if self.clear_confirm {
            self.store.clear();
            let _ = self.store.save(&self.storage_path);
            self.clear_confirm = false;
            self.selected_index = None;
            self.set_status("Cleared all colors");
        } else {
            self.clear_confirm = true;
            self.set_status("Press c again to clear all");
        }
    }

    pub fn cancel_clear(&mut self) {
        self.clear_confirm = false;
    }

    pub fn navigate_down(&mut self) {
        if self.store.saved_colors.is_empty() {
            return;
        }
        match self.selected_index {
            None => self.selected_index = Some(0),
            Some(i) => {
                if i < self.store.saved_colors.len() - 1 {
                    self.selected_index = Some(i + 1);
                }
            }
        }
    }

    pub fn navigate_up(&mut self) {
        if self.store.saved_colors.is_empty() {
            return;
        }
        match self.selected_index {
            None => self.selected_index = Some(0),
            Some(i) => {
                if i > 0 {
                    self.selected_index = Some(i - 1);
                }
            }
        }
    }

    pub fn selected_hex(&self) -> Option<String> {
        self.selected_index
            .and_then(|i| self.store.saved_colors.get(i))
            .map(|sc| sc.hex.clone())
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_app(dir: &TempDir) -> App {
        App::new(dir.path().join("colors.json"))
    }

    #[test]
    fn test_new_app_has_empty_state() {
        let dir = TempDir::new().unwrap();
        let app = test_app(&dir);
        assert!(app.store.saved_colors.is_empty());
        assert!(app.color_stream.is_empty());
        assert!(app.current_color.is_none());
        assert_eq!(app.selected_index, None);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_push_color_adds_to_stream() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        assert_eq!(app.color_stream.len(), 1);
        assert_eq!(app.current_color, Some(Color::new(255, 0, 0)));
    }

    #[test]
    fn test_push_color_deduplicates_consecutive() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.push_color(Color::new(255, 0, 0));
        assert_eq!(app.color_stream.len(), 1);
    }

    #[test]
    fn test_push_color_allows_different_colors() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.push_color(Color::new(0, 255, 0));
        assert_eq!(app.color_stream.len(), 2);
    }

    #[test]
    fn test_push_color_caps_at_200() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        for i in 0..250 {
            app.push_color(Color::new(i as u8, 0, 0));
        }
        assert_eq!(app.color_stream.len(), 200);
    }

    #[test]
    fn test_push_color_newest_at_front() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.push_color(Color::new(0, 255, 0));
        assert_eq!(app.color_stream[0], Color::new(0, 255, 0));
        assert_eq!(app.color_stream[1], Color::new(255, 0, 0));
    }

    #[test]
    fn test_save_current_color() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 87, 51));
        app.save_current_color();
        assert_eq!(app.store.saved_colors.len(), 1);
        assert_eq!(app.store.saved_colors[0].hex, "#FF5733");
    }

    #[test]
    fn test_save_no_current_color_is_noop() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.save_current_color();
        assert!(app.store.saved_colors.is_empty());
    }

    #[test]
    fn test_delete_selected() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.push_color(Color::new(0, 255, 0));
        app.save_current_color();
        app.selected_index = Some(0);
        app.delete_selected();
        assert_eq!(app.store.saved_colors.len(), 1);
        assert_eq!(app.store.saved_colors[0].hex, "#00FF00");
    }

    #[test]
    fn test_delete_no_selection_is_noop() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.selected_index = None;
        app.delete_selected();
        assert_eq!(app.store.saved_colors.len(), 1);
    }

    #[test]
    fn test_clear_confirm_flow() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();

        app.request_clear();
        assert!(app.clear_confirm);
        assert_eq!(app.store.saved_colors.len(), 1);

        app.request_clear();
        assert!(!app.clear_confirm);
        assert!(app.store.saved_colors.is_empty());
    }

    #[test]
    fn test_clear_confirm_cancelled_by_other_action() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();

        app.request_clear();
        assert!(app.clear_confirm);

        app.cancel_clear();
        assert!(!app.clear_confirm);
        assert_eq!(app.store.saved_colors.len(), 1);
    }

    #[test]
    fn test_navigate_down() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.push_color(Color::new(0, 255, 0));
        app.save_current_color();

        app.navigate_down();
        assert_eq!(app.selected_index, Some(0));
        app.navigate_down();
        assert_eq!(app.selected_index, Some(1));
        app.navigate_down();
        assert_eq!(app.selected_index, Some(1));
    }

    #[test]
    fn test_navigate_up() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.push_color(Color::new(0, 255, 0));
        app.save_current_color();

        app.selected_index = Some(1);
        app.navigate_up();
        assert_eq!(app.selected_index, Some(0));
        app.navigate_up();
        assert_eq!(app.selected_index, Some(0));
    }

    #[test]
    fn test_navigate_empty_list_is_noop() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.navigate_down();
        assert_eq!(app.selected_index, None);
    }

    #[test]
    fn test_selected_hex() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 87, 51));
        app.save_current_color();
        app.selected_index = Some(0);
        assert_eq!(app.selected_hex(), Some("#FF5733".to_string()));
    }

    #[test]
    fn test_selected_hex_none_when_no_selection() {
        let dir = TempDir::new().unwrap();
        let app = test_app(&dir);
        assert_eq!(app.selected_hex(), None);
    }

    #[test]
    fn test_delete_adjusts_selection_index() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.push_color(Color::new(0, 255, 0));
        app.save_current_color();
        app.push_color(Color::new(0, 0, 255));
        app.save_current_color();

        app.selected_index = Some(2);
        app.delete_selected();
        assert_eq!(app.selected_index, Some(1));
    }

    #[test]
    fn test_delete_last_item_clears_selection() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.selected_index = Some(0);
        app.delete_selected();
        assert_eq!(app.selected_index, None);
    }

    #[test]
    fn test_status_message() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.set_status("Copied!");
        assert_eq!(app.status_message, Some("Copied!".to_string()));
    }

    #[test]
    fn test_save_sets_status() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        assert_eq!(app.status_message, Some("Saved #FF0000".to_string()));
    }

    #[test]
    fn test_clear_sets_status() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir);
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.request_clear();
        assert_eq!(app.status_message, Some("Press c again to clear all".to_string()));
        app.request_clear();
        assert_eq!(app.status_message, Some("Cleared all colors".to_string()));
    }
}
