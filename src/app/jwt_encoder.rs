use super::{
  models::{ScrollableTxt, TabRoute, TabsState},
  ActiveBlock, Route, RouteId, TextAreaInput, TextInput,
};

pub struct Encoder<'a> {
  pub encoded: ScrollableTxt,
  pub header: TextAreaInput<'a>,
  pub payload: TextAreaInput<'a>,
  pub secret: TextInput,
  pub signature_verified: bool,
  pub blocks: TabsState,
}

impl Default for Encoder<'_> {
  fn default() -> Self {
    Self {
      encoded: Default::default(),
      header: Default::default(),
      payload: Default::default(),
      secret: Default::default(),
      signature_verified: false,
      blocks: TabsState::new(vec![
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderHeader,
          },
        },
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderPayload,
          },
        },
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderSecret,
          },
        },
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderToken,
          },
        },
      ]),
    }
  }
}
