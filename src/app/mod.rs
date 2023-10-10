pub(crate) mod jwt_decoder;
pub(crate) mod jwt_encoder;
pub(crate) mod jwt_utils;
pub(crate) mod key_binding;
pub(crate) mod models;
mod utils;

use jsonwebtoken::TokenData;
use ratatui::layout::Rect;
use serde::de;
use serde_json::to_string_pretty;
use tui_input::Input;

use self::{
  jwt_decoder::{decode_jwt_token, Decoder, Payload},
  jwt_encoder::Encoder,
  jwt_utils::JWTError,
  key_binding::DEFAULT_KEYBINDING,
  models::{ScrollableTxt, StatefulTable, TabRoute, TabsState},
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ActiveBlock {
  Help,
  DecoderToken,
  DecoderHeader,
  DecoderPayload,
  DecoderSignature,
  EncoderToken,
  EncoderHeader,
  EncoderPayload,
  EncoderSignature,
  Intro,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum RouteId {
  Help,
  Decoder,
  Encoder,
  Intro,
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

/// Holds data state for various views
#[derive(Default)]
pub struct Data {
  pub error: String,
  pub decoder: Decoder,
  pub encoder: Encoder,
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
  pub refresh: bool,
  pub help_docs: StatefulTable<Vec<String>>,
  pub data: Data,
}

impl Default for App {
  fn default() -> Self {
    App {
      navigation_stack: vec![DEFAULT_ROUTE],
      title: " JWT CLI - A command line UI for decoding JSON Web Tokens ",
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
          title: format!("Encoder {}", DEFAULT_KEYBINDING.jump_to_decoder.key),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderHeader,
          },
        },
        TabRoute {
          title: format!("JWT Introduction {}", DEFAULT_KEYBINDING.jump_to_intro.key),
          route: Route {
            id: RouteId::Intro,
            active_block: ActiveBlock::Intro,
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
      refresh: true,
      help_docs: StatefulTable::with_items(key_binding::get_help_docs()),
      data: Data::default(),
    }
  }
}

impl App {
  pub fn new(tick_rate: u64, token: Option<String>) -> Self {
    App {
      tick_rate,
      data: Data {
        decoder: Decoder {
          encoded: TextInput {
            input: Input::new(token.unwrap_or_default()),
            input_mode: InputMode::Normal,
          },
          ..Decoder::default()
        },
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

  pub async fn on_tick(&mut self) {
    decode_jwt_token(
      self,
      self.data.decoder.encoded.input.value().into(),
      self.data.decoder.secret.input.value().into(),
    );
  }
}

/// utility methods for tests
#[cfg(test)]
#[macro_use]
mod test_utils {

  //   pub fn convert_resource_from_file<K: Serialize, T>(filename: &str) -> (Vec<T>, Vec<K>)
  //   where
  //     K: Clone + DeserializeOwned + fmt::Debug,
  //     T: KubeResource<K> + From<K>,
  //   {
  //     let res_list = load_resource_from_file(filename);
  //     let original_res_list = res_list.items.clone();

  //     let resources: Vec<T> = res_list.into_iter().map(K::into).collect::<Vec<_>>();

  //     (resources, original_res_list)
  //   }

  //   pub fn load_resource_from_file<K>(filename: &str) -> ObjectList<K>
  //   where
  //     K: Clone + DeserializeOwned + fmt::Debug,
  //   {
  //     let yaml = fs::read_to_string(format!("./test_data/{}.yaml", filename))
  //       .expect("Something went wrong reading yaml file");
  //     assert_ne!(yaml, "".to_string());

  //     let res_list: serde_yaml::Result<ObjectList<K>> = serde_yaml::from_str(&yaml);
  //     assert!(res_list.is_ok(), "{:?}", res_list.err());
  //     res_list.unwrap()
  //   }

  #[macro_export]
  macro_rules! map_string_object {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {
        std::iter::Iterator::collect(IntoIterator::into_iter([$(($k.to_string(), $v),)*]))
    };
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[tokio::test]
  async fn test_on_tick_first_render() {
    let mut app = App {
      tick_rate: 250,
      ..App::default()
    };

    // test first render
    app.on_tick().await;

    assert!(!app.refresh);
    assert!(!app.is_routing);
  }
  #[tokio::test]
  async fn test_on_tick_refresh_tick_limit() {
    let mut app = App {
      tick_rate: 250,
      refresh: true,
      ..App::default()
    };

    // test first render
    app.on_tick().await;

    assert!(!app.refresh);
    assert!(!app.is_routing);
  }
  #[tokio::test]
  async fn test_on_tick_routing() {
    let mut app = App {
      tick_rate: 250,
      is_routing: true,
      refresh: false,
      ..App::default()
    };

    // test first render
    app.on_tick().await;

    assert!(!app.refresh);
    assert!(!app.is_routing);
  }
}
