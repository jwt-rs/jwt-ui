use crossterm::event::{Event, KeyEvent, MouseEvent, MouseEventKind};
use tui_input::backend::crossterm::EventHandler;

use crate::{
  app::{
    key_binding::DEFAULT_KEYBINDING, models::Scrollable, ActiveBlock, App, InputMode, RouteId,
    TextAreaInput, TextInput,
  },
  event::Key,
};

pub fn handle_key_events(key: Key, key_event: KeyEvent, app: &mut App) {
  // if input is enabled capture keystrokes
  if !is_any_text_editing(app, key, key_event) {
    // First handle any global event and then move to route event
    match key {
      _ if key == DEFAULT_KEYBINDING.esc.key && app.get_current_route().id == RouteId::Help => {
        app.pop_navigation_stack();
      }
      _ if key == DEFAULT_KEYBINDING.quit.key || key == DEFAULT_KEYBINDING.quit.alt.unwrap() => {
        app.should_quit = true;
      }
      _ if key == DEFAULT_KEYBINDING.up.key || key == DEFAULT_KEYBINDING.up.alt.unwrap() => {
        handle_block_scroll(app, true, false, false);
      }
      _ if key == DEFAULT_KEYBINDING.down.key || key == DEFAULT_KEYBINDING.down.alt.unwrap() => {
        handle_block_scroll(app, false, false, false);
      }
      _ if key == DEFAULT_KEYBINDING.pg_up.key => {
        handle_block_scroll(app, true, false, true);
      }
      _ if key == DEFAULT_KEYBINDING.pg_down.key => {
        handle_block_scroll(app, false, false, true);
      }
      _ if key == DEFAULT_KEYBINDING.right.key || key == DEFAULT_KEYBINDING.right.alt.unwrap() => {
        handle_right_key_events(app);
      }
      _ if key == DEFAULT_KEYBINDING.left.key || key == DEFAULT_KEYBINDING.left.alt.unwrap() => {
        handle_left_key_events(app);
      }
      _ if key == DEFAULT_KEYBINDING.toggle_theme.key => {
        app.light_theme = !app.light_theme;
      }
      _ if key == DEFAULT_KEYBINDING.refresh.key => app.refresh(),
      _ if key == DEFAULT_KEYBINDING.help.key
        && app.get_current_route().active_block != ActiveBlock::Help =>
      {
        app.push_navigation_stack(RouteId::Help, ActiveBlock::Help);
      }
      _ if key == DEFAULT_KEYBINDING.cycle_main_views.key => app.cycle_main_routes(),

      _ if key == DEFAULT_KEYBINDING.toggle_input_edit.key => handle_edit_event(app),

      _ if key == DEFAULT_KEYBINDING.copy_to_clipboard.key => handle_copy_event(app),

      _ => handle_route_events(key, app),
    }
  }
}

pub fn handle_mouse_events(mouse: MouseEvent, app: &mut App) {
  match mouse.kind {
    // mouse scrolling is inverted
    MouseEventKind::ScrollDown => handle_block_scroll(app, true, true, false),
    MouseEventKind::ScrollUp => handle_block_scroll(app, false, true, false),
    _ => { /* do nothing */ }
  }
}

fn handle_edit_event(app: &mut App) {
  match app.get_current_route().active_block {
    ActiveBlock::DecoderToken => app.data.decoder.encoded.input_mode = InputMode::Editing,
    ActiveBlock::DecoderSecret => app.data.decoder.secret.input_mode = InputMode::Editing,
    ActiveBlock::EncoderHeader => app.data.encoder.header.input_mode = InputMode::Editing,
    ActiveBlock::EncoderPayload => app.data.encoder.payload.input_mode = InputMode::Editing,
    ActiveBlock::EncoderSecret => app.data.encoder.secret.input_mode = InputMode::Editing,
    _ => { /* do nothing */ }
  }
}

fn handle_copy_event(app: &mut App) {
  match app.get_current_route().active_block {
    ActiveBlock::DecoderToken => {
      copy_to_clipboard(app.data.decoder.encoded.input.value().into());
    }
    ActiveBlock::DecoderHeader => {
      copy_to_clipboard(app.data.decoder.header.get_txt());
    }
    ActiveBlock::DecoderPayload => {
      copy_to_clipboard(app.data.decoder.payload.get_txt());
    }
    ActiveBlock::DecoderSecret => {
      copy_to_clipboard(app.data.decoder.secret.input.value().into());
    }
    ActiveBlock::EncoderToken => {
      todo!()
      //   copy_to_clipboard(app.data.decoder.encoded.input.value().into());
    }
    ActiveBlock::EncoderHeader => {
      todo!()
      //   copy_to_clipboard(app.data.decoder.encoded.input.value().into());
    }
    ActiveBlock::EncoderPayload => {
      todo!()
      //   copy_to_clipboard(app.data.decoder.encoded.input.value().into());
    }
    ActiveBlock::EncoderSecret => {
      todo!()
      //   copy_to_clipboard(app.data.decoder.encoded.input.value().into());
    }
    _ => { /* Do nothing */ }
  }
}

fn is_any_text_editing(app: &mut App, key: Key, key_event: KeyEvent) -> bool {
  match app.get_current_route().active_block {
    ActiveBlock::DecoderToken => is_text_editing(&mut app.data.decoder.encoded, key, key_event),
    ActiveBlock::DecoderSecret => is_text_editing(&mut app.data.decoder.secret, key, key_event),
    ActiveBlock::EncoderHeader => {
      is_text_area_editing(&mut app.data.encoder.header, key, key_event)
    }
    ActiveBlock::EncoderPayload => false,
    ActiveBlock::EncoderSecret => false,
    _ => false,
  }
}

fn is_text_editing(input: &mut TextInput, key: Key, key_event: KeyEvent) -> bool {
  if input.input_mode == InputMode::Editing {
    if key == DEFAULT_KEYBINDING.esc.key {
      input.input_mode = InputMode::Normal;
    } else {
      input.input.handle_event(&Event::Key(key_event));
    }
    true
  } else {
    false
  }
}

fn is_text_area_editing(input: &mut TextAreaInput<'_>, key: Key, key_event: KeyEvent) -> bool {
  if input.input_mode == InputMode::Editing {
    if key == DEFAULT_KEYBINDING.esc.key {
      input.input_mode = InputMode::Normal;
    } else {
      input.input.input(Event::Key(key_event));
    }
    true
  } else {
    false
  }
}

// Handle event for the current active block
fn handle_route_events(key: Key, app: &mut App) {
  // route specific events
  match app.get_current_route().id {
    // handle resource tabs on overview
    RouteId::Decoder => {
      match key {
        _ if key == DEFAULT_KEYBINDING.toggle_utc_dates.key => {
          app.data.decoder.utc_dates = !app.data.decoder.utc_dates;
        }
        _ if key == DEFAULT_KEYBINDING.toggle_ignore_exp.key => {
          app.data.decoder.ignore_exp = !app.data.decoder.ignore_exp;
        }
        // as these are tabs with index the order here matters, at least for readability
        _ => {}
      };
    }
    RouteId::Encoder => {}
    _ => { /* Do nothing */ }
  }
}

fn handle_left_key_events(app: &mut App) {
  // route specific events
  match app.get_current_route().id {
    RouteId::Decoder => {
      app.data.decoder.blocks.previous();
      app.push_navigation_route(app.data.decoder.blocks.get_active_route().clone());
    }
    RouteId::Encoder => {}
    _ => {
      todo!()
    }
  }
}

fn handle_right_key_events(app: &mut App) {
  // route specific events
  match app.get_current_route().id {
    RouteId::Decoder => {
      app.data.decoder.blocks.next();
      app.push_navigation_route(app.data.decoder.blocks.get_active_route().clone());
    }
    RouteId::Encoder => {}
    _ => {
      todo!()
    }
  }
}

fn handle_block_scroll(app: &mut App, up: bool, is_mouse: bool, page: bool) {
  match app.get_current_route().active_block {
    ActiveBlock::Help => app.help_docs.handle_scroll(up, page),
    ActiveBlock::DecoderHeader => app
      .data
      .decoder
      .header
      .handle_scroll(inverse_dir(up, is_mouse), page),
    ActiveBlock::DecoderPayload => app
      .data
      .decoder
      .payload
      .handle_scroll(inverse_dir(up, is_mouse), page),
    _ => {}
  }
}

#[cfg(target_arch = "x86_64")]
fn copy_to_clipboard(content: String) {
  use clipboard::{ClipboardContext, ClipboardProvider};

  let mut ctx: ClipboardContext = ClipboardProvider::new().expect("Unable to obtain clipboard");
  ctx
    .set_contents(content)
    .expect("Unable to set content to clipboard");
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
fn copy_to_clipboard(_content: String) {
  // do nothing as its a PITA to compile for ARM with XCB and this feature is not that important
}

/// inverse direction for natural scrolling on mouse and keyboard
fn inverse_dir(up: bool, is_mouse: bool) -> bool {
  if is_mouse {
    !up
  } else {
    up
  }
}

#[cfg(test)]
mod tests {
  use crossterm::event::KeyCode;

  use crate::app::{models::ScrollableTxt, Route};

  use super::*;

  #[test]
  fn test_inverse_dir() {
    assert!(inverse_dir(true, false));
    assert!(!inverse_dir(true, true));
  }

  #[test]
  fn test_handle_key_events_for_editor() {
    let mut app = App::default();

    app.route_home();
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Normal);

    let key_evt = KeyEvent::from(KeyCode::Char('e'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app);
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Editing);

    let key_evt = KeyEvent::from(KeyCode::Char('f'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app);
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Editing);
    assert_eq!(app.data.decoder.encoded.input.value(), String::from("f"));

    let key_evt = KeyEvent::from(KeyCode::Esc);
    handle_key_events(Key::from(key_evt), key_evt, &mut app);
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Normal);
  }

  #[test]
  fn test_handle_key_events_for_editor_editing() {
    let mut app = App::default();

    app.data.decoder.encoded.input_mode = InputMode::Editing;

    app.route_home();
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Editing);

    let key_evt = KeyEvent::from(KeyCode::Char('e'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app);
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Editing);
    assert_eq!(app.data.decoder.encoded.input.value(), String::from("e"));

    let key_evt = KeyEvent::from(KeyCode::Esc);
    handle_key_events(Key::from(key_evt), key_evt, &mut app);
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Normal);

    let key_evt = KeyEvent::from(KeyCode::Char('e'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app);
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Editing);
  }

  #[test]
  fn test_handle_key_events_for_textarea_editing() {
    let mut app = App::default();

    app.data.encoder.header.input_mode = InputMode::Editing;

    let route = app.main_tabs.set_index(1).route.clone();
    app.push_navigation_route(route);

    assert_eq!(app.data.encoder.header.input_mode, InputMode::Editing);

    let key_evt = KeyEvent::from(KeyCode::Char('e'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app);
    assert_eq!(app.data.encoder.header.input_mode, InputMode::Editing);
    assert_eq!(
      app.data.encoder.header.input.lines().join("sep"),
      String::from("e")
    );

    let key_evt = KeyEvent::from(KeyCode::Esc);
    handle_key_events(Key::from(key_evt), key_evt, &mut app);
    assert_eq!(app.data.encoder.header.input_mode, InputMode::Normal);

    let key_evt = KeyEvent::from(KeyCode::Char('e'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app);
    assert_eq!(app.data.encoder.header.input_mode, InputMode::Editing);
  }

  #[test]
  fn test_handle_block_scroll_with_help_block() {
    let mut app = App::default();
    app.push_navigation_route(Route {
      id: RouteId::Help,
      active_block: ActiveBlock::Help,
    });

    assert_eq!(app.help_docs.state.selected(), Some(0));

    handle_block_scroll(&mut app, true, false, false);
    assert_eq!(app.help_docs.state.selected(), Some(0));

    handle_block_scroll(&mut app, false, false, false);
    assert_eq!(app.help_docs.state.selected(), Some(1));

    handle_block_scroll(&mut app, false, false, true);
    assert_eq!(app.help_docs.state.selected(), Some(11));

    handle_block_scroll(&mut app, true, false, true);
    assert_eq!(app.help_docs.state.selected(), Some(1));
  }

  #[test]
  fn test_handle_block_scroll_with_decoder_header_block() {
    let mut app = App::default();
    app.data.decoder.header = ScrollableTxt::new("test\n multiline\n string".into());
    app.push_navigation_route(Route {
      id: RouteId::Decoder,
      active_block: ActiveBlock::DecoderHeader,
    });

    handle_block_scroll(&mut app, false, false, false);
    assert_eq!(app.data.decoder.header.offset, 0);

    app.data.decoder.header =
      ScrollableTxt::new("te\nst\nm\n\n\n\n\n\n\n\n\nul\ntil\ni\nne\nstr\ni\nn\ng".into());

    handle_block_scroll(&mut app, false, false, false);
    assert_eq!(app.data.decoder.header.offset, 1);

    handle_block_scroll(&mut app, false, false, false);
    assert_eq!(app.data.decoder.header.offset, 2);

    handle_block_scroll(&mut app, false, false, true);
    assert_eq!(app.data.decoder.header.offset, 12);

    handle_block_scroll(&mut app, true, false, true);
    assert_eq!(app.data.decoder.header.offset, 2);

    handle_block_scroll(&mut app, true, false, true);
    assert_eq!(app.data.decoder.header.offset, 0);
  }
}
