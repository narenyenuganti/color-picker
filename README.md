# color-picker

A macOS desktop-wide color picker with a terminal UI. Hover over any pixel on your screen to see its color, save colors you like, and copy hex values to your clipboard.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-only-blue)

## Features

- **Live color sampling** — a circle overlay follows your cursor, streaming the color under it to a TUI in real-time
- **Two-column TUI** — saved colors on the left, live color stream on the right
- **Persistent storage** — saved colors are stored in `~/.color-picker/colors.json` across sessions
- **Clipboard support** — copy any saved color's hex value with a single keypress

## Requirements

- macOS (uses CoreGraphics and AppKit APIs)
- Rust toolchain
- **Screen Recording** permission — macOS will prompt on first run. Without it, all sampled colors return `#000000`.

## Install

```bash
git clone https://github.com/narenyenuganti/color-picker.git
cd color-picker
cargo build --release
```

The binary will be at `target/release/color-picker`.

## Usage

```bash
cargo run
```

A small circle overlay appears and follows your cursor anywhere on the desktop. The TUI opens in your terminal showing two columns:

| Left — Saved Colors | Right — Color Stream |
|---|---|
| Colors you've saved with their hex values | Live scrolling log of colors under your cursor |

### Keybindings

| Key | Action |
|-----|--------|
| `s` / `Enter` | Save current color |
| `y` | Copy selected color's hex to clipboard |
| `d` | Delete selected saved color |
| `c` `c` | Clear all saved colors (double-tap) |
| `↑` / `↓` | Navigate saved colors |
| `q` / `Esc` | Quit |

## How It Works

Two threads communicate via channels:

- **Main thread** — runs the macOS `NSApplication` event loop, manages the cursor overlay window, and samples the pixel color under the cursor every ~50ms using `CGWindowListCreateImage`
- **TUI thread** — runs the Ratatui/Crossterm event loop, receives colors from the main thread, handles keyboard input, and renders the UI

The overlay window is transparent, always-on-top, and click-through (`ignoresMouseEvents`). It excludes itself from screenshots so it doesn't sample its own circle.

## License

MIT
