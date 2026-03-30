use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::color::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedColor {
    pub hex: String,
    pub saved_at: String,
}

impl SavedColor {
    pub fn to_color(&self) -> Color {
        Color::from_hex(&self.hex).unwrap_or(Color::new(0, 0, 0))
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ColorStore {
    pub saved_colors: Vec<SavedColor>,
}

impl ColorStore {
    pub fn load(path: &PathBuf) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    pub fn add(&mut self, color: &Color) {
        self.saved_colors.push(SavedColor {
            hex: color.to_hex(),
            saved_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.saved_colors.len() {
            self.saved_colors.remove(index);
        }
    }

    pub fn clear(&mut self) {
        self.saved_colors.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_path(dir: &TempDir) -> PathBuf {
        dir.path().join("colors.json")
    }

    #[test]
    fn test_load_nonexistent_returns_empty() {
        let dir = TempDir::new().unwrap();
        let store = ColorStore::load(&test_path(&dir));
        assert!(store.saved_colors.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = test_path(&dir);

        let mut store = ColorStore::default();
        store.add(&Color::new(255, 87, 51));
        store.save(&path).unwrap();

        let loaded = ColorStore::load(&path);
        assert_eq!(loaded.saved_colors.len(), 1);
        assert_eq!(loaded.saved_colors[0].hex, "#FF5733");
    }

    #[test]
    fn test_add_multiple_colors() {
        let mut store = ColorStore::default();
        store.add(&Color::new(255, 0, 0));
        store.add(&Color::new(0, 255, 0));
        store.add(&Color::new(0, 0, 255));
        assert_eq!(store.saved_colors.len(), 3);
    }

    #[test]
    fn test_remove_by_index() {
        let mut store = ColorStore::default();
        store.add(&Color::new(255, 0, 0));
        store.add(&Color::new(0, 255, 0));
        store.add(&Color::new(0, 0, 255));
        store.remove(1);
        assert_eq!(store.saved_colors.len(), 2);
        assert_eq!(store.saved_colors[0].hex, "#FF0000");
        assert_eq!(store.saved_colors[1].hex, "#0000FF");
    }

    #[test]
    fn test_remove_out_of_bounds_is_noop() {
        let mut store = ColorStore::default();
        store.add(&Color::new(255, 0, 0));
        store.remove(5);
        assert_eq!(store.saved_colors.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut store = ColorStore::default();
        store.add(&Color::new(255, 0, 0));
        store.add(&Color::new(0, 255, 0));
        store.clear();
        assert!(store.saved_colors.is_empty());
    }

    #[test]
    fn test_load_corrupted_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = test_path(&dir);
        std::fs::write(&path, "not json").unwrap();
        let store = ColorStore::load(&path);
        assert!(store.saved_colors.is_empty());
    }

    #[test]
    fn test_saved_color_has_timestamp() {
        let mut store = ColorStore::default();
        store.add(&Color::new(255, 0, 0));
        assert!(!store.saved_colors[0].saved_at.is_empty());
    }

    #[test]
    fn test_save_creates_parent_directory() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("subdir").join("colors.json");
        let mut store = ColorStore::default();
        store.add(&Color::new(255, 0, 0));
        store.save(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_saved_color_to_color() {
        let sc = SavedColor {
            hex: "#FF5733".to_string(),
            saved_at: "2026-03-29T00:00:00Z".to_string(),
        };
        assert_eq!(sc.to_color(), Color::new(255, 87, 51));
    }

    #[test]
    fn test_saved_color_invalid_hex_returns_black() {
        let sc = SavedColor {
            hex: "invalid".to_string(),
            saved_at: "2026-03-29T00:00:00Z".to_string(),
        };
        assert_eq!(sc.to_color(), Color::new(0, 0, 0));
    }
}
