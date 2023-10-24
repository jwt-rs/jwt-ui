pub(crate) mod jwt_decoder;
pub(crate) mod jwt_encoder;
pub(crate) mod jwt_utils;
pub(crate) mod key_binding;
pub(crate) mod models;
mod utils;

use ratatui::layout::Rect;
use tui_input::Input;
use tui_textarea::TextArea;

use self::{
  jwt_decoder::{decode_jwt_token, Decoder},
  jwt_encoder::{encode_jwt_token, Encoder},
  jwt_utils::JWTError,
  key_binding::DEFAULT_KEYBINDING,
  models::{StatefulTable, TabRoute, TabsState},
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ActiveBlock {
  Help,
  DecoderToken,
  DecoderHeader,
  DecoderPayload,
  DecoderSecret,
  EncoderToken,
  EncoderHeader,
  EncoderPayload,
  EncoderSecret,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum RouteId {
  Help,
  Decoder,
  Encoder,
}

#[derive(Debug, Clone)]
pub struct Route {
  pub id: RouteId,
  pub active_block: ActiveBlock,
}

const DEFAULT_ROUTE: Route = Route {
  id: RouteId::Decoder,
  active_block: ActiveBlock::DecoderToken,
};

#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub enum InputMode {
  #[default]
  Normal,
  Editing,
}

#[derive(Default, Debug, Clone)]
pub struct TextInput {
  /// Current value of the input box
  pub input: Input,
  /// Current input mode
  pub input_mode: InputMode,
}

impl TextInput {
  fn new(input: String) -> Self {
    Self {
      input: Input::new(input),
      input_mode: InputMode::Normal,
    }
  }
}

#[derive(Default, Debug, Clone)]
pub struct TextAreaInput<'a> {
  /// Current value of the text area
  pub input: TextArea<'a>,
  /// Current input mode
  pub input_mode: InputMode,
}

impl<'a> TextAreaInput<'a> {
  fn new(input: Vec<String>) -> Self {
    Self {
      input: input.into(),
      input_mode: InputMode::Normal,
    }
  }
}

/// Holds data state for various views
#[derive(Default)]
pub struct Data {
  pub error: String,
  pub decoder: Decoder,
  pub encoder: Encoder<'static>,
}

/// Holds main application state
pub struct App {
  navigation_stack: Vec<Route>,
  pub title: &'static str,
  pub should_quit: bool,
  pub main_tabs: TabsState,
  pub is_loading: bool,
  pub is_routing: bool,
  pub tick_rate: u64,
  pub size: Rect,
  pub dialog: Option<String>,
  pub confirm: bool,
  pub light_theme: bool,
  pub help_docs: StatefulTable<Vec<String>>,
  pub data: Data,
}

impl Default for App {
  fn default() -> Self {
    App {
      navigation_stack: vec![DEFAULT_ROUTE],
      title: " JWT UI - A Terminal UI for decoding/encoding JSON Web Tokens ",
      should_quit: false,
      main_tabs: TabsState::new(vec![
        TabRoute {
          title: format!("Decoder {}", DEFAULT_KEYBINDING.jump_to_decoder.key),
          route: Route {
            id: RouteId::Decoder,
            active_block: ActiveBlock::DecoderToken,
          },
        },
        TabRoute {
          title: format!("Encoder {}", DEFAULT_KEYBINDING.jump_to_encoder.key),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderHeader,
          },
        },
      ]),
      is_loading: false,
      is_routing: false,
      tick_rate: 0,
      size: Rect::default(),
      dialog: None,
      confirm: false,
      light_theme: false,
      help_docs: StatefulTable::with_items(key_binding::get_help_docs()),
      data: Data::default(),
    }
  }
}

impl App {
  pub fn new(tick_rate: u64, token: Option<String>, secret: String) -> Self {
    App {
      tick_rate,
      data: Data {
        decoder: Decoder::new(token, secret.clone()),
        encoder: Encoder::new(secret),
        ..Data::default()
      },
      ..App::default()
    }
  }

  pub fn refresh(&mut self) {
    self.data.error = String::new();
    self.data = Data::default();
    self.route_home();
  }

  pub fn handle_error(&mut self, e: JWTError) {
    self.data.error = format!("{}", e)
  }

  pub fn push_navigation_stack(&mut self, id: RouteId, active_block: ActiveBlock) {
    self.push_navigation_route(Route { id, active_block });
  }

  pub fn push_navigation_route(&mut self, route: Route) {
    self.navigation_stack.push(route);
    self.is_routing = true;
    self.data.error = String::default();
  }

  pub fn pop_navigation_stack(&mut self) -> Option<Route> {
    self.is_routing = true;
    if self.navigation_stack.len() == 1 {
      None
    } else {
      self.navigation_stack.pop()
    }
  }

  pub fn get_current_route(&self) -> &Route {
    // if for some reason there is no route return the default
    self.navigation_stack.last().unwrap_or(&DEFAULT_ROUTE)
  }

  pub fn cycle_main_routes(&mut self) {
    self.main_tabs.next();
    let route = self.main_tabs.get_active_route().clone();
    self.push_navigation_route(route);
  }

  pub fn route_home(&mut self) {
    let route = self.main_tabs.set_index(0).route.clone();
    self.push_navigation_route(route);
  }

  pub fn on_tick(&mut self) {
    match self.get_current_route().id {
      RouteId::Decoder => decode_jwt_token(self),
      RouteId::Encoder => encode_jwt_token(self),
      RouteId::Help => { /* nothing to do */ }
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_on_tick_first_render() {
    let mut app = App::new(250, Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.XbPfbIHMI6arZ3Y922BhjWgQzWXcXNrz0ogtVhfEd2o".to_string()), "secret".to_string());

    // test first render
    app.on_tick();

    assert!(!app.is_routing);
    assert!(app.data.error.is_empty());
    assert!(!app.data.decoder.header.get_txt().is_empty());
    assert!(!app.data.decoder.payload.get_txt().is_empty());
  }
}
