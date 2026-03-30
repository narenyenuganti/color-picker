# Color Picker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a macOS desktop-wide color picker with a TUI that shows a live color stream and persistent saved colors.

**Architecture:** Single Rust binary, two threads. Main thread runs the macOS event loop with a transparent overlay window and color sampler polling at ~50ms. TUI thread runs Ratatui in the alternate terminal screen, receiving colors via `mpsc::channel`. Quit signal via `Arc<AtomicBool>`.

**Tech Stack:** Rust, Ratatui 0.29, Crossterm 0.28, cocoa/objc (macOS overlay), core-graphics (pixel sampling), arboard (clipboard), serde/serde_json (persistence)

---

## File Structure

```
color-picker/
├── Cargo.toml
├── src/
│   ├── main.rs           — entry point, thread spawning, channel setup
│   ├── color.rs           — Color struct, hex conversion
│   ├── storage.rs         — SavedColor, ColorStore, JSON persistence
│   ├── sampler.rs         — macOS pixel sampling via CoreGraphics
│   ├── overlay.rs         — transparent NSWindow with circle cursor
│   └── tui/
│       ├── mod.rs         — re-exports
│       ├── app.rs         — App state struct, state management methods
│       ├── ui.rs          — Ratatui rendering (two-column layout)
│       └── input.rs       — Action enum, key-to-action mapping
```

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/color.rs`
- Create: `src/storage.rs`
- Create: `src/sampler.rs`
- Create: `src/overlay.rs`
- Create: `src/tui/mod.rs`
- Create: `src/tui/app.rs`
- Create: `src/tui/ui.rs`
- Create: `src/tui/input.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "color-picker"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
cocoa = "0.26"
objc = "0.2"
core-graphics = "0.24"
core-graphics-types = "0.2"
arboard = "3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Create stub files**

`src/main.rs`:
```rust
mod color;
mod overlay;
mod sampler;
mod storage;
mod tui;

fn main() {
    println!("color-picker");
}
```

`src/color.rs`:
```rust
// Color type and hex conversion
```

`src/storage.rs`:
```rust
// JSON persistence for saved colors
```

`src/sampler.rs`:
```rust
// macOS color sampling
```

`src/overlay.rs`:
```rust
// macOS cursor overlay window
```

`src/tui/mod.rs`:
```rust
pub mod app;
pub mod input;
pub mod ui;
```

`src/tui/app.rs`:
```rust
// TUI application state
```

`src/tui/ui.rs`:
```rust
// Ratatui rendering
```

`src/tui/input.rs`:
```rust
// Keyboard input handling
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles with no errors (may have warnings about empty files)

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: scaffold project structure with dependencies"
```

---

### Task 2: Color Module (TDD)

**Files:**
- Modify: `src/color.rs`

- [ ] **Step 1: Write failing tests**

Add to `src/color.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex_black() {
        let c = Color::new(0, 0, 0);
        assert_eq!(c.to_hex(), "#000000");
    }

    #[test]
    fn test_to_hex_white() {
        let c = Color::new(255, 255, 255);
        assert_eq!(c.to_hex(), "#FFFFFF");
    }

    #[test]
    fn test_to_hex_color() {
        let c = Color::new(255, 87, 51);
        assert_eq!(c.to_hex(), "#FF5733");
    }

    #[test]
    fn test_from_hex_with_hash() {
        let c = Color::from_hex("#FF5733").unwrap();
        assert_eq!(c, Color::new(255, 87, 51));
    }

    #[test]
    fn test_from_hex_without_hash() {
        let c = Color::from_hex("3498DB").unwrap();
        assert_eq!(c, Color::new(52, 152, 219));
    }

    #[test]
    fn test_from_hex_lowercase() {
        let c = Color::from_hex("#ff5733").unwrap();
        assert_eq!(c, Color::new(255, 87, 51));
    }

    #[test]
    fn test_from_hex_invalid_length() {
        assert!(Color::from_hex("#FFF").is_err());
    }

    #[test]
    fn test_from_hex_invalid_chars() {
        assert!(Color::from_hex("#GGGGGG").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let original = Color::new(123, 45, 67);
        let hex = original.to_hex();
        let parsed = Color::from_hex(&hex).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_to_ratatui_color() {
        let c = Color::new(255, 87, 51);
        assert_eq!(c.to_ratatui_color(), ratatui::style::Color::Rgb(255, 87, 51));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib color`
Expected: FAIL — `Color` struct not defined

- [ ] **Step 3: Implement Color struct**

Replace contents of `src/color.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(format!("expected 6 hex characters, got {}", hex.len()));
        }
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
        Ok(Self { r, g, b })
    }

    pub fn to_ratatui_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex_black() {
        let c = Color::new(0, 0, 0);
        assert_eq!(c.to_hex(), "#000000");
    }

    #[test]
    fn test_to_hex_white() {
        let c = Color::new(255, 255, 255);
        assert_eq!(c.to_hex(), "#FFFFFF");
    }

    #[test]
    fn test_to_hex_color() {
        let c = Color::new(255, 87, 51);
        assert_eq!(c.to_hex(), "#FF5733");
    }

    #[test]
    fn test_from_hex_with_hash() {
        let c = Color::from_hex("#FF5733").unwrap();
        assert_eq!(c, Color::new(255, 87, 51));
    }

    #[test]
    fn test_from_hex_without_hash() {
        let c = Color::from_hex("3498DB").unwrap();
        assert_eq!(c, Color::new(52, 152, 219));
    }

    #[test]
    fn test_from_hex_lowercase() {
        let c = Color::from_hex("#ff5733").unwrap();
        assert_eq!(c, Color::new(255, 87, 51));
    }

    #[test]
    fn test_from_hex_invalid_length() {
        assert!(Color::from_hex("#FFF").is_err());
    }

    #[test]
    fn test_from_hex_invalid_chars() {
        assert!(Color::from_hex("#GGGGGG").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let original = Color::new(123, 45, 67);
        let hex = original.to_hex();
        let parsed = Color::from_hex(&hex).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_to_ratatui_color() {
        let c = Color::new(255, 87, 51);
        assert_eq!(c.to_ratatui_color(), ratatui::style::Color::Rgb(255, 87, 51));
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib color`
Expected: All 10 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/color.rs
git commit -m "feat: add Color struct with hex conversion and tests"
```

---

### Task 3: Storage Module (TDD)

**Files:**
- Modify: `src/storage.rs`

- [ ] **Step 1: Write failing tests**

Add to `src/storage.rs`:
```rust
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
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib storage`
Expected: FAIL — `ColorStore` not defined

- [ ] **Step 3: Implement storage module**

Replace contents of `src/storage.rs`:
```rust
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
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib storage`
Expected: All 9 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/storage.rs
git commit -m "feat: add storage module with JSON persistence and tests"
```

---

### Task 4: TUI App State (TDD)

**Files:**
- Modify: `src/tui/app.rs`

- [ ] **Step 1: Write failing tests**

Add to `src/tui/app.rs`:
```rust
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

        // First press sets confirm
        app.request_clear();
        assert!(app.clear_confirm);
        assert_eq!(app.store.saved_colors.len(), 1);

        // Second press clears
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
        assert_eq!(app.selected_index, Some(1)); // stays at bottom
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
        assert_eq!(app.selected_index, Some(0)); // stays at top
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
        // After deleting last item, selection moves up
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
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib tui::app`
Expected: FAIL — `App` struct not defined

- [ ] **Step 3: Implement App state**

Replace contents of `src/tui/app.rs`:
```rust
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
        // Deduplicate consecutive same colors
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
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib tui::app`
Expected: All 20 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/tui/app.rs
git commit -m "feat: add TUI app state with navigation, save, delete, clear"
```

---

### Task 5: TUI Input Handling (TDD)

**Files:**
- Modify: `src/tui/input.rs`

- [ ] **Step 1: Write failing tests**

Add to `src/tui/input.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_quit_q() {
        assert_eq!(Action::from_key(key(KeyCode::Char('q'))), Some(Action::Quit));
    }

    #[test]
    fn test_quit_esc() {
        assert_eq!(Action::from_key(key(KeyCode::Esc)), Some(Action::Quit));
    }

    #[test]
    fn test_save_s() {
        assert_eq!(Action::from_key(key(KeyCode::Char('s'))), Some(Action::Save));
    }

    #[test]
    fn test_save_enter() {
        assert_eq!(Action::from_key(key(KeyCode::Enter)), Some(Action::Save));
    }

    #[test]
    fn test_copy_y() {
        assert_eq!(Action::from_key(key(KeyCode::Char('y'))), Some(Action::CopyHex));
    }

    #[test]
    fn test_delete_d() {
        assert_eq!(Action::from_key(key(KeyCode::Char('d'))), Some(Action::Delete));
    }

    #[test]
    fn test_clear_c() {
        assert_eq!(Action::from_key(key(KeyCode::Char('c'))), Some(Action::Clear));
    }

    #[test]
    fn test_navigate_up() {
        assert_eq!(Action::from_key(key(KeyCode::Up)), Some(Action::NavigateUp));
    }

    #[test]
    fn test_navigate_down() {
        assert_eq!(Action::from_key(key(KeyCode::Down)), Some(Action::NavigateDown));
    }

    #[test]
    fn test_unknown_key_returns_none() {
        assert_eq!(Action::from_key(key(KeyCode::Char('z'))), None);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib tui::input`
Expected: FAIL — `Action` enum not defined

- [ ] **Step 3: Implement input handling**

Replace contents of `src/tui/input.rs`:
```rust
use crossterm::event::KeyEvent;
use crossterm::event::KeyCode;

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Quit,
    Save,
    CopyHex,
    Delete,
    Clear,
    NavigateUp,
    NavigateDown,
}

impl Action {
    pub fn from_key(key: KeyEvent) -> Option<Self> {
        match key.code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Esc => Some(Action::Quit),
            KeyCode::Char('s') => Some(Action::Save),
            KeyCode::Enter => Some(Action::Save),
            KeyCode::Char('y') => Some(Action::CopyHex),
            KeyCode::Char('d') => Some(Action::Delete),
            KeyCode::Char('c') => Some(Action::Clear),
            KeyCode::Up => Some(Action::NavigateUp),
            KeyCode::Down => Some(Action::NavigateDown),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_quit_q() {
        assert_eq!(Action::from_key(key(KeyCode::Char('q'))), Some(Action::Quit));
    }

    #[test]
    fn test_quit_esc() {
        assert_eq!(Action::from_key(key(KeyCode::Esc)), Some(Action::Quit));
    }

    #[test]
    fn test_save_s() {
        assert_eq!(Action::from_key(key(KeyCode::Char('s'))), Some(Action::Save));
    }

    #[test]
    fn test_save_enter() {
        assert_eq!(Action::from_key(key(KeyCode::Enter)), Some(Action::Save));
    }

    #[test]
    fn test_copy_y() {
        assert_eq!(Action::from_key(key(KeyCode::Char('y'))), Some(Action::CopyHex));
    }

    #[test]
    fn test_delete_d() {
        assert_eq!(Action::from_key(key(KeyCode::Char('d'))), Some(Action::Delete));
    }

    #[test]
    fn test_clear_c() {
        assert_eq!(Action::from_key(key(KeyCode::Char('c'))), Some(Action::Clear));
    }

    #[test]
    fn test_navigate_up() {
        assert_eq!(Action::from_key(key(KeyCode::Up)), Some(Action::NavigateUp));
    }

    #[test]
    fn test_navigate_down() {
        assert_eq!(Action::from_key(key(KeyCode::Down)), Some(Action::NavigateDown));
    }

    #[test]
    fn test_unknown_key_returns_none() {
        assert_eq!(Action::from_key(key(KeyCode::Char('z'))), None);
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib tui::input`
Expected: All 10 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/tui/input.rs
git commit -m "feat: add keyboard input handling with action mapping"
```

---

### Task 6: TUI Rendering

**Files:**
- Modify: `src/tui/ui.rs`
- Modify: `src/tui/mod.rs`

- [ ] **Step 1: Write failing render test**

Add to `src/tui/ui.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use tempfile::TempDir;

    #[test]
    fn test_render_empty_state_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let app = App::new(dir.path().join("colors.json"));
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_colors_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.push_color(Color::new(0, 255, 0));
        app.selected_index = Some(0);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_status_message() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.push_color(Color::new(255, 0, 0));
        app.set_status("Copied!");

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_clear_confirm() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.clear_confirm = true;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib tui::ui`
Expected: FAIL — `render` function not defined

- [ ] **Step 3: Implement rendering**

Replace contents of `src/tui/ui.rs`:
```rust
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::color::Color;
use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[0]);

    render_saved_colors(f, app, columns[0]);
    render_color_stream(f, app, columns[1]);
    render_bottom_bar(f, app, main_chunks[1]);
}

fn render_saved_colors(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = format!(" Saved Colors ({}) ", app.store.saved_colors.len());

    let items: Vec<ListItem> = app
        .store
        .saved_colors
        .iter()
        .map(|sc| {
            let color = sc.to_color();
            let rcolor = color.to_ratatui_color();
            ListItem::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(rcolor)),
                Span::raw(&sc.hex),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    state.select(app.selected_index);
    f.render_stateful_widget(list, area, &mut state);
}

fn render_color_stream(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .color_stream
        .iter()
        .map(|c| {
            let rcolor = c.to_ratatui_color();
            ListItem::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(rcolor)),
                Span::styled(c.to_hex(), Style::default().fg(rcolor)),
                Span::raw(" "),
                Span::styled("████", Style::default().fg(rcolor)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Color Stream "),
    );

    f.render_widget(list, area);
}

fn render_bottom_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let current_text = match &app.current_color {
        Some(c) => {
            let rcolor = c.to_ratatui_color();
            Line::from(vec![
                Span::raw(" Current: "),
                Span::styled("● ", Style::default().fg(rcolor)),
                Span::styled(
                    c.to_hex(),
                    Style::default().fg(rcolor).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                status_span(app),
                Span::raw("  "),
                Span::styled(
                    "s:save  d:del  c:clear  y:copy  q:quit",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                ),
            ])
        }
        None => Line::from(vec![
            Span::raw(" Waiting for color..."),
            Span::raw("  "),
            Span::styled(
                "q:quit",
                Style::default().fg(ratatui::style::Color::DarkGray),
            ),
        ]),
    };

    let bar = Paragraph::new(current_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Status "),
    );

    f.render_widget(bar, area);
}

fn status_span(app: &App) -> Span<'_> {
    match &app.status_message {
        Some(msg) if app.clear_confirm => {
            Span::styled(msg.as_str(), Style::default().fg(ratatui::style::Color::Red))
        }
        Some(msg) => {
            Span::styled(msg.as_str(), Style::default().fg(ratatui::style::Color::Yellow))
        }
        None => Span::raw(""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use tempfile::TempDir;

    #[test]
    fn test_render_empty_state_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let app = App::new(dir.path().join("colors.json"));
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_colors_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.push_color(Color::new(0, 255, 0));
        app.selected_index = Some(0);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_status_message() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.push_color(Color::new(255, 0, 0));
        app.set_status("Copied!");

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_clear_confirm() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.clear_confirm = true;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }
}
```

- [ ] **Step 4: Update `src/tui/mod.rs`**

```rust
pub mod app;
pub mod input;
pub mod ui;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib tui::ui`
Expected: All 4 tests PASS

- [ ] **Step 6: Commit**

```bash
git add src/tui/
git commit -m "feat: add TUI rendering with two-column layout and status bar"
```

---

### Task 7: macOS Color Sampler

**Files:**
- Modify: `src/sampler.rs`

- [ ] **Step 1: Implement color sampler**

Replace contents of `src/sampler.rs`:
```rust
use core_graphics::display::{
    kCGWindowImageDefault, kCGWindowListOptionOnScreenBelowWindow, CGWindowID,
    CGWindowListCreateImage,
};
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::{CGPoint, CGRect, CGSize};

use crate::color::Color;

/// Returns the current cursor position in screen coordinates (top-left origin).
pub fn get_cursor_position() -> (f64, f64) {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .expect("failed to create event source");
    let event = CGEvent::new(source).expect("failed to create event");
    let point = event.location();
    (point.x, point.y)
}

/// Samples the pixel color at the given screen coordinates.
/// `exclude_window` is the CGWindowID of the overlay window to exclude from capture.
pub fn sample_color(x: f64, y: f64, exclude_window: CGWindowID) -> Option<Color> {
    let rect = CGRect::new(&CGPoint::new(x, y), &CGSize::new(1.0, 1.0));

    let image = unsafe {
        CGWindowListCreateImage(
            rect,
            kCGWindowListOptionOnScreenBelowWindow,
            exclude_window,
            kCGWindowImageDefault,
        )
    };

    if image.is_null() {
        return None;
    }

    let width = unsafe { core_graphics::display::CGImageGetWidth(image) };
    let height = unsafe { core_graphics::display::CGImageGetHeight(image) };

    if width == 0 || height == 0 {
        unsafe { core_graphics::display::CGImageRelease(image) };
        return None;
    }

    let data_provider = unsafe { core_graphics::display::CGImageGetDataProvider(image) };
    if data_provider.is_null() {
        unsafe { core_graphics::display::CGImageRelease(image) };
        return None;
    }

    let data = unsafe { core_graphics::display::CGDataProviderCopyData(data_provider) };
    if data.is_null() {
        unsafe { core_graphics::display::CGImageRelease(image) };
        return None;
    }

    let ptr = unsafe { core_graphics::display::CFDataGetBytePtr(data) };
    let len = unsafe { core_graphics::display::CFDataGetLength(data) } as usize;

    let result = if len >= 4 {
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        // macOS pixel format is typically BGRA
        Some(Color::new(bytes[2], bytes[1], bytes[0]))
    } else {
        None
    };

    unsafe {
        core_graphics::display::CFRelease(data as *const _);
        core_graphics::display::CGImageRelease(image);
    };

    result
}
```

**Note:** The exact FFI function signatures may need adjustment based on the `core-graphics` crate version. If the crate doesn't expose `CGWindowListCreateImage` directly, add raw FFI declarations:

```rust
extern "C" {
    fn CGWindowListCreateImage(
        screenBounds: CGRect,
        listOption: u32,
        windowID: u32,
        imageOption: u32,
    ) -> *const std::ffi::c_void;
    fn CGImageGetWidth(image: *const std::ffi::c_void) -> usize;
    fn CGImageGetHeight(image: *const std::ffi::c_void) -> usize;
    fn CGImageGetDataProvider(image: *const std::ffi::c_void) -> *const std::ffi::c_void;
    fn CGDataProviderCopyData(provider: *const std::ffi::c_void) -> *const std::ffi::c_void;
    fn CFDataGetBytePtr(data: *const std::ffi::c_void) -> *const u8;
    fn CFDataGetLength(data: *const std::ffi::c_void) -> isize;
    fn CFRelease(cf: *const std::ffi::c_void);
    fn CGImageRelease(image: *const std::ffi::c_void);
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles (may need to adjust FFI signatures)

- [ ] **Step 3: Manual test — run a quick sanity check**

Add a temporary test binary or println in main to verify sampling works:
```rust
// Temporary in main.rs
let (x, y) = sampler::get_cursor_position();
println!("Cursor at ({}, {})", x, y);
if let Some(color) = sampler::sample_color(x, y, 0) {
    println!("Color: {}", color.to_hex());
}
```

Run: `cargo run`
Expected: Prints cursor position and a hex color value. If all colors are `#000000`, Screen Recording permission needs to be granted.

- [ ] **Step 4: Remove temporary test code and commit**

```bash
git add src/sampler.rs
git commit -m "feat: add macOS color sampler via CoreGraphics"
```

---

### Task 8: macOS Cursor Overlay

**Files:**
- Modify: `src/overlay.rs`

- [ ] **Step 1: Implement overlay window**

Replace contents of `src/overlay.rs`:
```rust
use cocoa::appkit::{
    NSApplication, NSApplicationActivationPolicyAccessory, NSBackingStoreBuffered, NSColor,
    NSView, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize};
use core_graphics::display::CGWindowLevelForKey;
use core_graphics::display::kCGFloatingWindowLevelKey;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL};
use objc::{class, msg_send, sel, sel_impl};

const OVERLAY_SIZE: f64 = 40.0;

/// Registers a custom NSView subclass that draws a circle.
fn register_circle_view_class() -> &'static Class {
    let superclass = class!(NSView);
    let mut decl = ClassDecl::new("CircleOverlayView", superclass).unwrap();

    extern "C" fn draw_rect(this: &Object, _sel: Sel, _dirty_rect: NSRect) {
        unsafe {
            let bounds: NSRect = msg_send![this, bounds];
            let inset_rect = NSRect::new(
                NSPoint::new(bounds.origin.x + 4.0, bounds.origin.y + 4.0),
                NSSize::new(bounds.size.width - 8.0, bounds.size.height - 8.0),
            );

            // Dark outline for visibility on light backgrounds
            let path: id = msg_send![class!(NSBezierPath), bezierPathWithOvalInRect:inset_rect];
            let dark: id = msg_send![class!(NSColor),
                colorWithRed:0.0f64
                green:0.0f64
                blue:0.0f64
                alpha:0.5f64
            ];
            let _: () = msg_send![dark, set];
            let _: () = msg_send![path, setLineWidth:3.0f64];
            let _: () = msg_send![path, stroke];

            // White inner stroke for visibility on dark backgrounds
            let white: id = msg_send![class!(NSColor),
                colorWithRed:1.0f64
                green:1.0f64
                blue:1.0f64
                alpha:0.9f64
            ];
            let _: () = msg_send![white, set];
            let _: () = msg_send![path, setLineWidth:1.5f64];
            let _: () = msg_send![path, stroke];
        }
    }

    extern "C" fn is_opaque(_this: &Object, _sel: Sel) -> BOOL {
        NO
    }

    unsafe {
        decl.add_method(
            sel!(drawRect:),
            draw_rect as extern "C" fn(&Object, Sel, NSRect),
        );
        decl.add_method(
            sel!(isOpaque),
            is_opaque as extern "C" fn(&Object, Sel) -> BOOL,
        );
    }

    decl.register()
}

pub struct Overlay {
    window: id,
}

impl Overlay {
    /// Creates the overlay. Must be called from the main thread after NSApplication init.
    pub fn new() -> Self {
        let view_class = register_circle_view_class();

        unsafe {
            let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(OVERLAY_SIZE, OVERLAY_SIZE),
                ),
                NSWindowStyleMask::NSBorderlessWindowMask,
                NSBackingStoreBuffered,
                NO,
            );

            window.setOpaque_(NO);
            window.setBackgroundColor_(NSColor::clearColor(nil));
            let level = CGWindowLevelForKey(kCGFloatingWindowLevelKey) as i64 + 1;
            let _: () = msg_send![window, setLevel: level];
            window.setIgnoresMouseEvents_(YES);
            window.setHasShadow_(NO);

            let frame = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(OVERLAY_SIZE, OVERLAY_SIZE),
            );
            let view: id = msg_send![view_class, alloc];
            let view: id = msg_send![view, initWithFrame: frame];
            window.setContentView_(view);
            window.makeKeyAndOrderFront_(nil);

            Self { window }
        }
    }

    /// Returns the CGWindowID of the overlay window (for excluding from screenshots).
    pub fn window_id(&self) -> u32 {
        unsafe {
            let num: i64 = msg_send![self.window, windowNumber];
            num as u32
        }
    }

    /// Moves the overlay to follow the cursor.
    /// `x` and `y` are in CoreGraphics coordinates (top-left origin).
    pub fn update_position(&self, x: f64, y: f64) {
        unsafe {
            let screen: id = msg_send![class!(NSScreen), mainScreen];
            let screen_frame: NSRect = msg_send![screen, frame];
            // Convert CG (top-left origin) to NS (bottom-left origin)
            let ns_y = screen_frame.size.height - y - (OVERLAY_SIZE / 2.0);
            let ns_x = x - (OVERLAY_SIZE / 2.0);
            let origin = NSPoint::new(ns_x, ns_y);
            self.window.setFrameOrigin_(origin);
        }
    }

    /// Hides the overlay window.
    pub fn hide(&self) {
        unsafe {
            let _: () = msg_send![self.window, orderOut: nil];
        }
    }
}

/// Initializes the macOS application environment. Must be called from the main thread.
pub fn init_macos_app() {
    unsafe {
        let app = NSApplication::sharedApplication(nil);
        app.setActivationPolicy_(NSApplicationActivationPolicyAccessory);
    }
}

/// Processes pending macOS events. Call this in the main loop.
pub fn process_events() {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let app = NSApplication::sharedApplication(nil);
        loop {
            let event: id = msg_send![app,
                nextEventMatchingMask:u64::MAX
                untilDate:nil
                inMode:cocoa::foundation::NSDefaultRunLoopMode
                dequeue:YES
            ];
            if event == nil {
                break;
            }
            let _: () = msg_send![app, sendEvent: event];
        }
        let _: () = msg_send![pool, drain];
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully

- [ ] **Step 3: Manual test — verify overlay appears**

Temporarily add to main.rs:
```rust
use std::thread;
use std::time::Duration;

fn main() {
    overlay::init_macos_app();
    let overlay = overlay::Overlay::new();

    for _ in 0..100 {
        let (x, y) = sampler::get_cursor_position();
        overlay.update_position(x, y);
        overlay::process_events();
        thread::sleep(Duration::from_millis(50));
    }

    overlay.hide();
}
```

Run: `cargo run`
Expected: A small circle follows the cursor for ~5 seconds, then disappears.

- [ ] **Step 4: Remove temporary test code and commit**

```bash
git add src/overlay.rs
git commit -m "feat: add macOS cursor overlay with transparent circle window"
```

---

### Task 9: Main Integration

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement the TUI event loop function**

Add to `src/tui/mod.rs` (or create `src/tui/run.rs`):

Update `src/tui/mod.rs`:
```rust
pub mod app;
pub mod input;
pub mod ui;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::color::Color;
use self::app::App;
use self::input::Action;
use self::ui::render;

pub fn run_tui(
    rx: Receiver<Color>,
    quit_signal: Arc<AtomicBool>,
    storage_path: std::path::PathBuf,
) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(storage_path);

    loop {
        // Receive colors (non-blocking)
        while let Ok(color) = rx.try_recv() {
            app.push_color(color);
        }

        // Render
        terminal.draw(|f| render(f, &app))?;

        // Handle input (poll with 16ms timeout for ~60fps)
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match Action::from_key(key) {
                    Some(Action::Quit) => {
                        app.should_quit = true;
                    }
                    Some(Action::Save) => {
                        app.cancel_clear();
                        app.save_current_color();
                    }
                    Some(Action::CopyHex) => {
                        app.cancel_clear();
                        if let Some(hex) = app.selected_hex() {
                            match arboard::Clipboard::new()
                                .and_then(|mut cb| cb.set_text(&hex))
                            {
                                Ok(()) => app.set_status(&format!("Copied {}", hex)),
                                Err(_) => app.set_status("Failed to copy"),
                            }
                        }
                    }
                    Some(Action::Delete) => {
                        app.cancel_clear();
                        app.delete_selected();
                    }
                    Some(Action::Clear) => {
                        app.request_clear();
                    }
                    Some(Action::NavigateUp) => {
                        app.cancel_clear();
                        app.navigate_up();
                    }
                    Some(Action::NavigateDown) => {
                        app.cancel_clear();
                        app.navigate_down();
                    }
                    None => {
                        app.cancel_clear();
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Signal main thread to stop
    quit_signal.store(true, Ordering::Relaxed);

    Ok(())
}
```

- [ ] **Step 2: Implement main.rs**

Replace contents of `src/main.rs`:
```rust
mod color;
mod overlay;
mod sampler;
mod storage;
mod tui;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let storage_path = dirs::home_dir()
        .expect("could not find home directory")
        .join(".color-picker")
        .join("colors.json");

    // Channel for sending colors from sampler to TUI
    let (tx, rx) = mpsc::channel();

    // Shared quit signal
    let quit = Arc::new(AtomicBool::new(false));

    // Spawn TUI on a separate thread
    let quit_clone = quit.clone();
    let storage_clone = storage_path.clone();
    let tui_handle = thread::spawn(move || {
        if let Err(e) = tui::run_tui(rx, quit_clone, storage_clone) {
            eprintln!("TUI error: {}", e);
        }
    });

    // Main thread: macOS overlay + color sampling
    overlay::init_macos_app();
    let overlay = overlay::Overlay::new();
    let window_id = overlay.window_id();

    // Main sampling loop
    while !quit.load(Ordering::Relaxed) {
        let (x, y) = sampler::get_cursor_position();
        overlay.update_position(x, y);

        if let Some(color) = sampler::sample_color(x, y, window_id) {
            // If TUI thread has exited, send will fail — that's fine
            let _ = tx.send(color);
        }

        overlay::process_events();
        thread::sleep(Duration::from_millis(50));
    }

    // Cleanup
    overlay.hide();
    drop(tx); // Close channel so TUI thread's rx.try_recv() returns Disconnected
    let _ = tui_handle.join();
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles with no errors

- [ ] **Step 4: Full manual test**

Run: `cargo run`
Expected:
1. A small circle overlay appears and follows the cursor
2. The TUI shows two columns — saved colors on the left, streaming colors on the right
3. Moving the cursor over different parts of the screen populates the color stream
4. Press `s` to save the current color — it appears in the left column
5. Arrow keys navigate the saved colors
6. Press `y` to copy a saved color's hex to clipboard
7. Press `d` to delete a saved color
8. Press `c` twice to clear all saved colors
9. Press `q` to quit — overlay disappears, terminal restores

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/tui/mod.rs
git commit -m "feat: wire up main integration — sampler, overlay, and TUI"
```

- [ ] **Step 6: Final integration commit**

Run all tests one more time:
```bash
cargo test
```
Expected: All unit tests pass.

```bash
git add -A
git commit -m "chore: final cleanup and verify all tests pass"
```
