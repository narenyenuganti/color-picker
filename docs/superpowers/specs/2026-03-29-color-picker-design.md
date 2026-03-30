# Color Picker — Design Spec

## Overview

A macOS desktop-wide color picker with a terminal UI. When launched, a small circle overlay follows the cursor anywhere on the desktop. The color under the cursor is streamed in real-time to a TUI. Users can save colors, copy hex values to the clipboard, and manage a persistent color collection.

## Architecture

**Single Rust binary, two threads, three logical modules.**

### Thread Model

- **Main thread**: Runs the macOS `NSApplication` run loop. Handles the cursor overlay window and color sampling on a ~50ms timer via `CGDisplayCreateImageForRect`. Sends sampled colors to the TUI thread over an `mpsc::channel`.
- **TUI thread**: Runs the Ratatui/Crossterm event loop in the alternate terminal screen. Receives color updates from the channel, handles keyboard input, and renders the UI.

### Module Layout

```
color-picker
├── Cargo.toml
├── src/
│   ├── main.rs           — entry point, spawns threads, sets up channels
│   ├── sampler.rs         — macOS color sampling (CGDisplay pixel read)
│   ├── overlay.rs         — transparent NSWindow overlay with circle cursor
│   ├── tui/
│   │   ├── mod.rs         — TUI app state, event loop
│   │   ├── ui.rs          — Ratatui rendering (two-column layout)
│   │   └── input.rs       — keyboard event handling
│   └── storage.rs         — JSON file persistence
```

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.29 | TUI rendering framework |
| `crossterm` | 0.28 | Terminal backend & event handling |
| `objc2` | latest | macOS Objective-C runtime bindings |
| `objc2-foundation` | latest | NSObject, NSString, etc. |
| `objc2-app-kit` | latest | NSWindow, NSApplication, NSView |
| `core-graphics` | latest | CGDisplayCreateImageForRect, CGEvent |
| `arboard` | latest | Clipboard access (copy hex) |
| `serde` | 1 | Serialization |
| `serde_json` | 1 | JSON persistence |

## macOS Integration

### Color Sampling

- Uses `CGDisplayCreateImageForRect` to capture a 1x1 pixel at the current cursor position obtained from `CGEvent::location`.
- Polls every ~50ms — fast enough for real-time feel, light on CPU.
- Returns RGB values, converted to hex string (`#RRGGBB`).
- Requires **Screen Recording** permission. macOS prompts on first run.
- If permission is denied, all colors return `#000000`. The TUI detects this and displays a message asking the user to grant Screen Recording permission in System Settings.

### Cursor Overlay

- Creates a borderless, transparent `NSWindow` at floating window level (always on top).
- Window is ~40x40px, follows cursor position every frame.
- `ignoresMouseEvents = true` — click-through, does not interfere with normal use.
- Draws a thin circle (2px stroke, white with dark outline for visibility on any background).
- Runs on the main thread as part of the `NSApplication` run loop.

## TUI Layout

**40/60 two-column split.**

### Left Column — Saved Colors (40%)

- Bordered block titled "Saved Colors (N)".
- Each row: colored circle (`●`) + hex value (e.g., `● #FF5733`).
- Selected row is highlighted for navigation.
- Keybinding hints displayed at the bottom of the block.

### Right Column — Color Stream (60%)

- Bordered block titled "Color Stream".
- Scrolling log of sampled colors, newest at top.
- Each row: colored circle + hex value + color bar swatch.
- Deduplication: if color hasn't changed since last sample, no new row is added.
- Capped at ~200 entries in memory; older entries scroll off.

### Bottom Bar

- Single row showing the current color under cursor prominently (the color you'd save if you pressed `s`).

## Keyboard Controls

| Key | Action |
|-----|--------|
| `s` / `Enter` | Save current color to saved list |
| `y` | Copy selected saved color's hex to clipboard |
| `q` / `Esc` | Quit (saves state, closes overlay, restores terminal) |
| `d` | Delete selected saved color |
| `c` (double-tap) | Clear all saved colors |
| `↑` / `↓` | Navigate saved colors list |

## Data Persistence

### Storage Location

`~/.color-picker/colors.json`

### Format

```json
{
  "saved_colors": [
    { "hex": "#FF5733", "saved_at": "2026-03-29T17:30:00Z" },
    { "hex": "#3498DB", "saved_at": "2026-03-29T17:31:15Z" }
  ]
}
```

### Behavior

- Loaded on startup — saved colors appear immediately in the left column.
- Written on every save/delete/clear operation.
- Directory (`~/.color-picker/`) created automatically on first run if it doesn't exist.
- `d` removes one entry, `c` (double-tap) wipes to an empty array.
