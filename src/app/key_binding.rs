use std::fmt;
use crossterm::event::KeyCode;
use crate::event::Key;

// Macro to generate key bindings with automatic iterable support
macro_rules! generate_keybindings {
    ($($field:ident),+) => {
        pub struct KeyBindings { $(pub $field: KeyBinding),+ }

        impl KeyBindings {
            // Provides an iterable vector of key bindings
            pub fn as_iter(&self) -> Vec<&KeyBinding> {
                vec![$(&self.$field),+]
            }
        }
    };
}

generate_keybindings! {
    // Order as shown in Help documentation
    quit,
    esc,
    help,
    refresh,
    toggle_theme,
    cycle_main_views,
    jump_to_decoder,
    jump_to_encoder,
    copy_to_clipboard,
    pg_up,
    pg_down,
    up,
    down,
    left,
    right,
    toggle_utc_dates,
    toggle_ignore_exp,
    toggle_input_edit,
    clear_input,
    delete_prev_char,
    go_to_prev_char,
    go_to_prev_word,
    go_to_next_char,
    go_to_next_word,
    delete_prev_word,
    delete_next_word,
    delete_till_end,
    go_to_start,
    go_to_end
}

// Defines the context in which the key binding is used
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum HContext {
    General,
    Editable,
    Decoder,
    // Encoder, (Uncomment if required later)
}

impl fmt::Display for HContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Struct representing a single key binding with description and context
#[derive(Clone)]
pub struct KeyBinding {
    pub key: Key,
    pub alt: Option<Key>,
    pub desc: &'static str,
    pub context: HContext,
}

// Default key binding set
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
        desc: "Close child page/Go back/Stop editing",
        context: HContext::General,
    },
    help: KeyBinding {
        key: Key::Char('?'),
        alt: None,
        desc: "Help page",
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
    jump_to_decoder: KeyBinding {
        key: Key::Char('D'),
        alt: None,
        desc: "Switch to decoder view",
        context: HContext::General,
    },
    jump_to_encoder: KeyBinding {
        key: Key::Char('E'),
        alt: None,
        desc: "Switch to encoder view",
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
        desc: "Focus next block",
        context: HContext::General,
    },
    right: KeyBinding {
        key: Key::Right,
        alt: Some(Key::Char('l')),
        desc: "Focus previous block",
        context: HContext::General,
    },
    toggle_utc_dates: KeyBinding {
        key: Key::Char('u'),
        alt: None,
        desc: "Toggle showing dates in UTC format",
        context: HContext::Decoder,
    },
    toggle_ignore_exp: KeyBinding {
        key: Key::Char('i'),
        alt: None,
        desc: "Toggle ignoring exp claim from validation",
        context: HContext::Decoder,
    },
    toggle_input_edit: KeyBinding {
        key: Key::Enter,
        alt: Some(Key::Char('e')),
        desc: "Enable text input edit mode",
        context: HContext::Editable,
    },
    clear_input: KeyBinding {
        key: Key::Ctrl('d'),
        alt: Some(Key::CtrlK(KeyCode::Backspace)),
        desc: "Clear input",
        context: HContext::Editable,
    },
    delete_prev_char: KeyBinding {
        key: Key::Backspace,
        alt: Some(Key::Ctrl('h')),
        desc: "Delete previous character",
        context: HContext::Editable,
    },
    go_to_prev_char: KeyBinding {
        key: Key::Left,
        alt: Some(Key::Ctrl('b')),
        desc: "Goto previous character",
        context: HContext::Editable,
    },
    go_to_prev_word: KeyBinding {
        key: Key::CtrlK(KeyCode::Left),
        alt: None,
        desc: "Goto previous word",
        context: HContext::Editable,
    },
    go_to_next_char: KeyBinding {
        key: Key::Right,
        alt: Some(Key::Ctrl('f')),
        desc: "Goto next character",
        context: HContext::Editable,
    },
    go_to_next_word: KeyBinding {
        key: Key::CtrlK(KeyCode::Right),
        alt: None,
        desc: "Goto next word",
        context: HContext::Editable,
    },
    delete_prev_word: KeyBinding {
        key: Key::Ctrl('w'),
        alt: None,
        desc: "Delete previous word",
        context: HContext::Editable,
    },
    delete_next_word: KeyBinding {
        key: Key::CtrlK(KeyCode::Delete),
        alt: None,
        desc: "Delete next word",
        context: HContext::Editable,
    },
    delete_till_end: KeyBinding {
        key: Key::Ctrl('k'),
        alt: None,
        desc: "Delete till end",
        context: HContext::Editable,
    },
    go_to_start: KeyBinding {
        key: Key::Home,
        alt: Some(Key::Ctrl('a')),
        desc: "Goto start",
        context: HContext::Editable,
    },
    go_to_end: KeyBinding {
        key: Key::End,
        alt: Some(Key::Ctrl('e')),
        desc: "Goto end",
        context: HContext::Editable,
    },
};

// Generates help documentation
pub fn get_help_docs() -> Vec<Vec<String>> {
    DEFAULT_KEYBINDING.as_iter().iter().map(help_row).collect()
}

// Creates a help row for each key binding
fn help_row(item: &KeyBinding) -> Vec<String> {
    vec![
        if let Some(alt_key) = item.alt {
            format!("{} | {}", item.key, alt_key)
        } else {
            item.key.to_string()
        },
        item.desc.to_string(),
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
