use std::fmt;

use crossterm::event::KeyCode;

use crate::event::Key;

// using a macro so that we can automatically generate an iterable vector for bindings. This beats reflection :)
macro_rules! generate_keybindings {
  ($($field:ident),+) => {
    pub struct KeyBindings { $(pub $field: KeyBinding),+ }
    impl KeyBindings {
      pub fn as_iter(&self) -> Vec<&KeyBinding> {
        vec![
            $(&self.$field),+
        ]
      }
    }
  };
}

generate_keybindings! {
  // order here is shown as is in Help
  quit,
  esc,
  help,
  refresh,
  submit,
  toggle_theme,
  cycle_main_views,
  jump_to_debugger,
  jump_to_intro,
  copy_to_clipboard,
  pg_up,
  pg_down,
  up,
  down,
  left,
  right,
  toggle_input_edit,
  delete_prev_char,
  go_to_prev_char,
  go_to_prev_word,
  go_to_next_char,
  go_to_next_word,
  delete_line,
  delete_prev_word,
  delete_next_word,
  delete_till_end,
  go_to_start,
  go_to_end
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum HContext {
  General,
  Debugger,
  Editor,
  Introduction,
}

impl fmt::Display for HContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Clone)]
pub struct KeyBinding {
  pub key: Key,
  pub alt: Option<Key>,
  pub desc: &'static str,
  pub context: HContext,
}

pub const DEFAULT_KEYBINDING: KeyBindings = KeyBindings {
  quit: KeyBinding {
    key: Key::Ctrl('c'),
    alt: Some(Key::Char('q')),
    desc: "Quit",
    context: HContext::General,
  },
  esc: KeyBinding {
    key: Key::Esc,
    alt: None,
    desc: "Close child page/Go back",
    context: HContext::General,
  },
  help: KeyBinding {
    key: Key::Char('?'),
    alt: None,
    desc: "Help page",
    context: HContext::General,
  },
  submit: KeyBinding {
    key: Key::Enter,
    alt: None,
    desc: "Select item",
    context: HContext::General,
  },
  refresh: KeyBinding {
    key: Key::Ctrl('r'),
    alt: None,
    desc: "Refresh UI",
    context: HContext::General,
  },
  toggle_theme: KeyBinding {
    key: Key::Char('t'),
    alt: None,
    desc: "Toggle theme",
    context: HContext::General,
  },
  jump_to_debugger: KeyBinding {
    key: Key::Char('D'),
    alt: None,
    desc: "Switch to debugger view",
    context: HContext::General,
  },
  jump_to_intro: KeyBinding {
    key: Key::Char('I'),
    alt: None,
    desc: "Switch to JWT introduction view",
    context: HContext::General,
  },
  cycle_main_views: KeyBinding {
    key: Key::Tab,
    alt: None,
    desc: "Cycle through main views",
    context: HContext::General,
  },
  copy_to_clipboard: KeyBinding {
    key: Key::Char('c'),
    alt: None,
    desc: "Copy content to clipboard",
    context: HContext::General,
  },
  down: KeyBinding {
    key: Key::Down,
    alt: Some(Key::Char('j')),
    desc: "Next item/Scroll down",
    context: HContext::General,
  },
  up: KeyBinding {
    key: Key::Up,
    alt: Some(Key::Char('k')),
    desc: "Previous item/Scroll up",
    context: HContext::General,
  },
  pg_up: KeyBinding {
    key: Key::PageUp,
    alt: None,
    desc: "Scroll page up",
    context: HContext::General,
  },
  pg_down: KeyBinding {
    key: Key::PageDown,
    alt: None,
    desc: "Scroll page down",
    context: HContext::General,
  },
  left: KeyBinding {
    key: Key::Left,
    alt: Some(Key::Char('h')),
    desc: "Next block",
    context: HContext::Debugger,
  },
  right: KeyBinding {
    key: Key::Right,
    alt: Some(Key::Char('l')),
    desc: "Previous block",
    context: HContext::Debugger,
  },
  toggle_input_edit: KeyBinding {
    key: Key::Char('e'),
    alt: None,
    desc: "Enable text input edit mode",
    context: HContext::Debugger,
  },

  // (Delete, KeyModifiers::CONTROL) => Some(DeleteNextWord),
  delete_prev_char: KeyBinding {
    key: Key::Backspace,
    alt: Some(Key::Ctrl('h')),
    desc: "Delete previous character",
    context: HContext::Editor,
  },
  go_to_prev_char: KeyBinding {
    key: Key::Left,
    alt: Some(Key::Ctrl('b')),
    desc: "Goto previous character",
    context: HContext::Editor,
  },
  go_to_prev_word: KeyBinding {
    key: Key::CtrlK(KeyCode::Left),
    alt: Some(Key::Meta('b')),
    desc: "Goto previous word",
    context: HContext::Editor,
  },
  go_to_next_char: KeyBinding {
    key: Key::Right,
    alt: Some(Key::Ctrl('f')),
    desc: "Goto next character",
    context: HContext::Editor,
  },
  go_to_next_word: KeyBinding {
    key: Key::CtrlK(KeyCode::Right),
    alt: Some(Key::Meta('f')),
    desc: "Goto next word",
    context: HContext::Editor,
  },
  delete_line: KeyBinding {
    key: Key::Ctrl('u'),
    alt: None,
    desc: "Delete line",
    context: HContext::Editor,
  },
  delete_prev_word: KeyBinding {
    key: Key::Meta('d'),
    alt: Some(Key::Ctrl('w')),
    desc: "Delete previous word",
    context: HContext::Editor,
  },
  delete_next_word: KeyBinding {
    key: Key::CtrlK(KeyCode::Delete),
    alt: None,
    desc: "Delete next word",
    context: HContext::Editor,
  },
  delete_till_end: KeyBinding {
    key: Key::Ctrl('k'),
    alt: None,
    desc: "Delete till end",
    context: HContext::Editor,
  },
  go_to_start: KeyBinding {
    key: Key::Home,
    alt: Some(Key::Ctrl('a')),
    desc: "Goto start",
    context: HContext::Editor,
  },
  go_to_end: KeyBinding {
    key: Key::End,
    alt: Some(Key::Ctrl('e')),
    desc: "Goto end",
    context: HContext::Editor,
  },
};

pub fn get_help_docs() -> Vec<Vec<String>> {
  let items = DEFAULT_KEYBINDING.as_iter();

  items.iter().map(|it| help_row(it)).collect()
}

fn help_row(item: &KeyBinding) -> Vec<String> {
  vec![
    if item.alt.is_some() {
      format!("{} | {}", item.key, item.alt.unwrap())
    } else {
      item.key.to_string()
    },
    String::from(item.desc),
    item.context.to_string(),
  ]
}

#[cfg(test)]
mod tests {
  use super::DEFAULT_KEYBINDING;

  #[test]
  fn test_as_iter() {
    assert!(DEFAULT_KEYBINDING.as_iter().len() >= 28);
  }
}
