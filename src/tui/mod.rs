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
