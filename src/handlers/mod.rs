use crossterm::event::{Event, KeyEvent, MouseEvent, MouseEventKind};
use tui_input::backend::crossterm::EventHandler;

use crate::{
  app::{
    jwt::decode_jwt_token, key_binding::DEFAULT_KEYBINDING, models::StatefulTable, ActiveBlock,
    App, InputMode, RouteId,
  },
  event::Key,
};

pub async fn handle_key_events(key: Key, key_event: KeyEvent, app: &mut App) {
  // if input is enabled capture keystrokes
  if app.data.decoder.encoded.input_mode == InputMode::Editing {
    if key == DEFAULT_KEYBINDING.esc.key {
      app.data.decoder.encoded.input_mode = InputMode::Normal;
    } else {
      app
        .data
        .decoder
        .encoded
        .input
        .handle_event(&Event::Key(key_event));
    }
  } else {
    // First handle any global event and then move to route event
    match key {
      _ if key == DEFAULT_KEYBINDING.esc.key => {
        handle_escape(app);
      }
      _ if key == DEFAULT_KEYBINDING.quit.key || key == DEFAULT_KEYBINDING.quit.alt.unwrap() => {
        app.should_quit = true;
      }
      _ if key == DEFAULT_KEYBINDING.up.key || key == DEFAULT_KEYBINDING.up.alt.unwrap() => {
        handle_block_scroll(app, true, false, false).await;
      }
      _ if key == DEFAULT_KEYBINDING.down.key || key == DEFAULT_KEYBINDING.down.alt.unwrap() => {
        handle_block_scroll(app, false, false, false).await;
      }
      _ if key == DEFAULT_KEYBINDING.pg_up.key => {
        handle_block_scroll(app, true, false, true).await;
      }
      _ if key == DEFAULT_KEYBINDING.pg_down.key => {
        handle_block_scroll(app, false, false, true).await;
      }
      _ if key == DEFAULT_KEYBINDING.toggle_theme.key => {
        app.light_theme = !app.light_theme;
      }
      _ if key == DEFAULT_KEYBINDING.refresh.key => {
        app.refresh();
      }
      _ if key == DEFAULT_KEYBINDING.help.key => {
        if app.get_current_route().active_block != ActiveBlock::Help {
          app.push_navigation_stack(RouteId::Help, ActiveBlock::Help);
        }
      }
      _ if key == DEFAULT_KEYBINDING.cycle_main_views.key => {
        app.cycle_main_routes();
      }
      _ => handle_route_events(key, app).await,
    }
  }
}

pub async fn handle_mouse_events(mouse: MouseEvent, app: &mut App) {
  match mouse.kind {
    // mouse scrolling is inverted
    MouseEventKind::ScrollDown => handle_block_scroll(app, true, true, false).await,
    MouseEventKind::ScrollUp => handle_block_scroll(app, false, true, false).await,
    _ => {}
  }
}

fn handle_escape(app: &mut App) {
  match app.get_current_route().id {
    RouteId::Help => {
      app.pop_navigation_stack();
    }
    _ => {}
  }
}

// Handle event for the current active block
async fn handle_route_events(key: Key, app: &mut App) {
  // route specific events
  match app.get_current_route().id {
    // handle resource tabs on overview
    RouteId::Decoder => {
      match key {
        _ if key == DEFAULT_KEYBINDING.right.key
          || key == DEFAULT_KEYBINDING.right.alt.unwrap() =>
        {
          //   app.context_tabs.next();
          //   app.push_navigation_route(app.context_tabs.get_active_route().clone());
        }
        _ if key == DEFAULT_KEYBINDING.left.key || key == DEFAULT_KEYBINDING.left.alt.unwrap() => {
          //   app.context_tabs.previous();
          //   app.push_navigation_route(app.context_tabs.get_active_route().clone());
        }
        _ if key == DEFAULT_KEYBINDING.toggle_input_edit.key => {
          app.data.decoder.encoded.input_mode = InputMode::Editing;
        }

        // as these are tabs with index the order here matters, atleast for readability
        _ => {}
      };

      // handle block specific stuff
      match app.get_current_route().active_block {
        _ => { /* Do nothing */ }
      }
    }
    _ => { /* Do nothing */ }
  }
}

fn handle_block_action<T: Clone>(key: Key, item: &StatefulTable<T>) -> Option<T> {
  match key {
    _ if key == DEFAULT_KEYBINDING.submit.key => item.get_selected_item_copy(),
    _ => None,
  }
}

async fn handle_block_scroll(_app: &mut App, _up: bool, _is_mouse: bool, _page: bool) {
  //   match app.get_current_route().active_block {
  //     ActiveBlock::Describe | ActiveBlock::Yaml => app
  //       .data
  //       .describe_out
  //       .handle_scroll(inverse_dir(up, is_mouse), page),
  //   }X
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

  use super::*;

  #[test]
  fn test_inverse_dir() {
    assert!(inverse_dir(true, false));
    assert!(!inverse_dir(true, true));
  }

  #[tokio::test]

  async fn test_handle_key_events_for_filter() {
    let mut app = App::default();

    app.route_home();
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Normal);

    let key_evt = KeyEvent::from(KeyCode::Char('f'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app).await;
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Editing);

    let key_evt = KeyEvent::from(KeyCode::Esc);
    handle_key_events(Key::from(key_evt), key_evt, &mut app).await;
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Normal);

    let key_evt = KeyEvent::from(KeyCode::Char('e'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app).await;
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Editing);

    let key_evt = KeyEvent::from(KeyCode::Char('f'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app).await;
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Editing);

    let key_evt = KeyEvent::from(KeyCode::Esc);
    handle_key_events(Key::from(key_evt), key_evt, &mut app).await;
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Normal);
    let key_evt = KeyEvent::from(KeyCode::Char('f'));
    handle_key_events(Key::from(key_evt), key_evt, &mut app).await;
    assert_eq!(app.data.decoder.encoded.input_mode, InputMode::Normal);
  }

  #[tokio::test]
  async fn test_decode_secret() {
    const DATA1: &str = "Hello, World!";
    const DATA2: &str =
      "Neque porro quisquam est qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit";

    let mut app = App::default();
    app.route_home();

    // let mut secret = KubeSecret::default();
    // // ByteString base64 encodes the data
    // secret
    //   .data
    //   .insert(String::from("key1"), ByteString(DATA1.as_bytes().into()));
    // secret
    //   .data
    //   .insert(String::from("key2"), ByteString(DATA2.as_bytes().into()));

    // // ensure that 'x' decodes the secret data
    // assert!(
    //   handle_describe_decode_or_yaml_action(
    //     Key::Char('x'),
    //     &mut app,
    //     &secret,
    //     IoCmdEvent::GetDescribe {
    //       kind: "secret".to_owned(),
    //       value: "name".to_owned(),
    //       ns: Some("namespace".to_owned()),
    //     }
    //   )
    //   .await
    // );

    // assert!(app
    //   .data
    //   .describe_out
    //   .get_txt()
    //   .contains(format!("key1: {}", DATA1).as_str()));
    // assert!(app
    //   .data
    //   .describe_out
    //   .get_txt()
    //   .contains(format!("key2: {}", DATA2).as_str()));
  }

  #[tokio::test]
  async fn test_handle_scroll() {
    let mut app = App::default();

    app.route_home();
    // assert_eq!(app.data.pods.state.selected(), None);

    // app
    //   .data
    //   .pods
    //   .set_items(vec![KubePod::default(), KubePod::default()]);

    // mouse scroll
    // assert_eq!(app.data.pods.state.selected(), Some(0));
    handle_block_scroll(&mut app, false, true, false).await;
    // assert_eq!(app.data.pods.state.selected(), Some(1));
    handle_block_scroll(&mut app, true, true, false).await;
    // assert_eq!(app.data.pods.state.selected(), Some(0));

    // check logs keyboard scroll
    // app.push_navigation_stack(RouteId::Home, ActiveBlock::Logs);
    // assert_eq!(app.data.logs.state.selected(), None);

    // app.data.logs.add_record("record".to_string());
    // app.data.logs.add_record("record 2".to_string());
    // app.data.logs.add_record("record 3".to_string());

    handle_block_scroll(&mut app, true, false, false).await;
    // assert_eq!(app.data.logs.state.selected(), Some(0));
  }
}
