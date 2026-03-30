use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

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
        assert_eq!(
            Action::from_key(key(KeyCode::Char('s'))),
            Some(Action::Save)
        );
    }

    #[test]
    fn test_save_enter() {
        assert_eq!(Action::from_key(key(KeyCode::Enter)), Some(Action::Save));
    }

    #[test]
    fn test_copy_y() {
        assert_eq!(
            Action::from_key(key(KeyCode::Char('y'))),
            Some(Action::CopyHex)
        );
    }

    #[test]
    fn test_delete_d() {
        assert_eq!(
            Action::from_key(key(KeyCode::Char('d'))),
            Some(Action::Delete)
        );
    }

    #[test]
    fn test_clear_c() {
        assert_eq!(
            Action::from_key(key(KeyCode::Char('c'))),
            Some(Action::Clear)
        );
    }

    #[test]
    fn test_navigate_up() {
        assert_eq!(
            Action::from_key(key(KeyCode::Up)),
            Some(Action::NavigateUp)
        );
    }

    #[test]
    fn test_navigate_down() {
        assert_eq!(
            Action::from_key(key(KeyCode::Down)),
            Some(Action::NavigateDown)
        );
    }

    #[test]
    fn test_unknown_key_returns_none() {
        assert_eq!(Action::from_key(key(KeyCode::Char('z'))), None);
    }

    #[test]
    fn test_tab_returns_none() {
        assert_eq!(Action::from_key(key(KeyCode::Tab)), None);
    }

    #[test]
    fn test_backspace_returns_none() {
        assert_eq!(Action::from_key(key(KeyCode::Backspace)), None);
    }
}
