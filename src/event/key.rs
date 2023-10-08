// from https://github.com/Rigellute/spotify-tui
use std::fmt;

use crossterm::event::{self, KeyCode};

/// Represents an key.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Key {
  /// Both Enter (or Return) and numpad Enter
  Enter,
  Tab,
  Backspace,
  Esc,
  /// Left arrow
  Left,
  /// Right arrow
  Right,
  /// Up arrow
  Up,
  /// Down arrow
  Down,
  /// Insert key
  Ins,
  /// Delete key
  Delete,
  /// Home key
  Home,
  /// End key
  End,
  /// Page Up key
  PageUp,
  /// Page Down key
  PageDown,
  /// F0 key
  F0,
  /// F1 key
  F1,
  /// F2 key
  F2,
  /// F3 key
  F3,
  /// F4 key
  F4,
  /// F5 key
  F5,
  /// F6 key
  F6,
  /// F7 key
  F7,
  /// F8 key
  F8,
  /// F9 key
  F9,
  /// F10 key
  F10,
  /// F11 key
  F11,
  /// F12 key
  F12,
  Char(char),
  Ctrl(char),
  CtrlK(KeyCode),
  Alt(char),
  Meta(char),
  Unknown,
}

impl Key {
  /// Returns the function key corresponding to the given number
  ///
  /// 1 -> F1, etc...
  ///
  /// # Panics
  ///
  /// If `n == 0 || n > 12`
  pub fn from_f(n: u8) -> Key {
    match n {
      0 => Key::F0,
      1 => Key::F1,
      2 => Key::F2,
      3 => Key::F3,
      4 => Key::F4,
      5 => Key::F5,
      6 => Key::F6,
      7 => Key::F7,
      8 => Key::F8,
      9 => Key::F9,
      10 => Key::F10,
      11 => Key::F11,
      12 => Key::F12,
      _ => panic!("unknown function key: F{}", n),
    }
  }
}

impl fmt::Display for Key {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      Key::Alt(' ') => write!(f, "<Alt+Space>"),
      Key::Ctrl(' ') => write!(f, "<Ctrl+Space>"),
      Key::Char(' ') => write!(f, "<Space>"),
      Key::Alt(c) => write!(f, "<Alt+{}>", c),
      Key::Meta(c) => write!(f, "<Meta+{}>", c),
      Key::Ctrl(c) => write!(f, "<Ctrl+{}>", c),
      Key::CtrlK(k) => write!(f, "<Ctrl+{:?}>", k),
      Key::Char(c) => write!(f, "<{}>", c),
      Key::Left | Key::Right | Key::Up | Key::Down => write!(f, "<{:?} Arrow Key>", self),
      _ => write!(f, "<{:?}>", self),
    }
  }
}

impl From<event::KeyEvent> for Key {
  fn from(key_event: event::KeyEvent) -> Self {
    match key_event {
      event::KeyEvent {
        code: KeyCode::Left,
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => Key::CtrlK(KeyCode::Left),
      event::KeyEvent {
        code: KeyCode::Right,
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => Key::CtrlK(KeyCode::Right),
      event::KeyEvent {
        code: KeyCode::Delete,
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => Key::CtrlK(KeyCode::Delete),
      event::KeyEvent {
        code: KeyCode::Esc, ..
      } => Key::Esc,
      event::KeyEvent {
        code: KeyCode::Backspace,
        ..
      } => Key::Backspace,
      event::KeyEvent {
        code: KeyCode::Left,
        ..
      } => Key::Left,
      event::KeyEvent {
        code: KeyCode::Right,
        ..
      } => Key::Right,
      event::KeyEvent {
        code: KeyCode::Up, ..
      } => Key::Up,
      event::KeyEvent {
        code: KeyCode::Down,
        ..
      } => Key::Down,
      event::KeyEvent {
        code: KeyCode::Home,
        ..
      } => Key::Home,
      event::KeyEvent {
        code: KeyCode::End, ..
      } => Key::End,
      event::KeyEvent {
        code: KeyCode::PageUp,
        ..
      } => Key::PageUp,
      event::KeyEvent {
        code: KeyCode::PageDown,
        ..
      } => Key::PageDown,
      event::KeyEvent {
        code: KeyCode::Delete,
        ..
      } => Key::Delete,
      event::KeyEvent {
        code: KeyCode::Insert,
        ..
      } => Key::Ins,
      event::KeyEvent {
        code: KeyCode::F(n),
        ..
      } => Key::from_f(n),
      event::KeyEvent {
        code: KeyCode::Enter,
        ..
      } => Key::Enter,
      event::KeyEvent {
        code: KeyCode::Tab, ..
      } => Key::Tab,

      // First check for char + modifier
      event::KeyEvent {
        code: KeyCode::Char(c),
        modifiers: event::KeyModifiers::ALT,
        ..
      } => Key::Alt(c),
      event::KeyEvent {
        code: KeyCode::Char(c),
        modifiers: event::KeyModifiers::CONTROL,
        ..
      } => Key::Ctrl(c),
      event::KeyEvent {
        code: KeyCode::Char(c),
        modifiers: event::KeyModifiers::META,
        ..
      } => Key::Meta(c),

      event::KeyEvent {
        code: KeyCode::Char(c),
        ..
      } => Key::Char(c),

      _ => Key::Unknown,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_key_fmt() {
    assert_eq!(format!("{}", Key::Left), "<Left Arrow Key>");
    assert_eq!(format!("{}", Key::Alt(' ')), "<Alt+Space>");
    assert_eq!(format!("{}", Key::Alt('c')), "<Alt+c>");
    assert_eq!(format!("{}", Key::Char('c')), "<c>");
    assert_eq!(format!("{}", Key::Enter), "<Enter>");
    assert_eq!(format!("{}", Key::F10), "<F10>");
  }
  #[test]
  fn test_key_from_event() {
    assert_eq!(Key::from(event::KeyEvent::from(KeyCode::Esc)), Key::Esc);
    assert_eq!(Key::from(event::KeyEvent::from(KeyCode::F(2))), Key::F2);
    assert_eq!(
      Key::from(event::KeyEvent::from(KeyCode::Char('J'))),
      Key::Char('J')
    );
    assert_eq!(
      Key::from(event::KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: event::KeyModifiers::ALT,
        kind: event::KeyEventKind::Press,
        state: event::KeyEventState::NONE,
      }),
      Key::Alt('c')
    );
    assert_eq!(
      Key::from(event::KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: event::KeyModifiers::CONTROL,
        kind: event::KeyEventKind::Press,
        state: event::KeyEventState::NONE
      }),
      Key::Ctrl('c')
    );
  }
}
