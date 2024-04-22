use tokio::sync::mpsc;

use super::actions::*;
use super::*;
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    pub fn handle_events(
        &mut self,
        tx: mpsc::UnboundedSender<Option<Action>>,
    ) -> tokio::task::JoinHandle<()> {
        let tick_rate = std::time::Duration::from_millis(250);
        tokio::spawn(async move {
            loop {
                let action = if crossterm::event::poll(tick_rate).unwrap() {
                    if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                        handle_key_event(key)
                    } else {
                        None
                    }
                } else {
                    None
                };
                if tx.send(action).is_err() {
                    break;
                }
            }
        })
    }
}

fn handle_key_event(key: KeyEvent) -> Option<Action> {
    if key.kind == crossterm::event::KeyEventKind::Press {
        let action = match key.code {
            KeyCode::Esc => Action::Exit,

            // Navigation
            KeyCode::Right => Action::MoveRight,
            KeyCode::Left => Action::MoveLeft,
            KeyCode::Down => Action::MoveDown,
            KeyCode::Up => Action::MoveUp,
            KeyCode::Enter => Action::Enter,

            // Enter words
            KeyCode::Char(x) if x.is_ascii_alphabetic() => Action::EnterChar(x),
            KeyCode::Backspace => Action::DeleteChar,
            KeyCode::Tab => Action::ToggleStatus,
            _ => return None,
        };
        Some(action)
    } else {
        None
    }
}
