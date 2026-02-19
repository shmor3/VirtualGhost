use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub enum Action {
    /// Send raw bytes to the SSH session
    SendBytes(Vec<u8>),
    /// Quit the application
    Quit,
    /// Terminal was resized
    Resize(u16, u16),
    /// No action needed
    None,
}

pub struct InputHandler;

impl InputHandler {
    pub fn handle_event(event: Event) -> Action {
        match event {
            Event::Key(key) => Self::handle_key(key),
            Event::Resize(cols, rows) => Action::Resize(cols, rows),
            _ => Action::None,
        }
    }

    fn handle_key(key: KeyEvent) -> Action {
        // Ctrl+Shift+Q to quit
        if key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
            && key.code == KeyCode::Char('Q')
        {
            return Action::Quit;
        }

        // Ctrl+C, Ctrl+D, etc. — pass through as raw control bytes
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char(c) = key.code {
                let byte = (c as u8).wrapping_sub(b'a').wrapping_add(1);
                return Action::SendBytes(vec![byte]);
            }
        }

        match key.code {
            KeyCode::Char(c) => Action::SendBytes(c.to_string().into_bytes()),
            KeyCode::Enter => Action::SendBytes(vec![b'\r']),
            KeyCode::Backspace => Action::SendBytes(vec![0x7f]),
            KeyCode::Tab => Action::SendBytes(vec![b'\t']),
            KeyCode::Esc => Action::SendBytes(vec![0x1b]),
            KeyCode::Up => Action::SendBytes(b"\x1b[A".to_vec()),
            KeyCode::Down => Action::SendBytes(b"\x1b[B".to_vec()),
            KeyCode::Right => Action::SendBytes(b"\x1b[C".to_vec()),
            KeyCode::Left => Action::SendBytes(b"\x1b[D".to_vec()),
            KeyCode::Home => Action::SendBytes(b"\x1b[H".to_vec()),
            KeyCode::End => Action::SendBytes(b"\x1b[F".to_vec()),
            KeyCode::PageUp => Action::SendBytes(b"\x1b[5~".to_vec()),
            KeyCode::PageDown => Action::SendBytes(b"\x1b[6~".to_vec()),
            KeyCode::Delete => Action::SendBytes(b"\x1b[3~".to_vec()),
            _ => Action::None,
        }
    }
}
