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
    drop(tx);
    let _ = tui_handle.join();
}
