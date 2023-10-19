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
            id: RouteId::Decoder,
            active_block: ActiveBlock::DecoderToken,
          },
        },
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Decoder,
            active_block: ActiveBlock::DecoderHeader,
          },
        },
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Decoder,
            active_block: ActiveBlock::DecoderPayload,
          },
        },
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Decoder,
            active_block: ActiveBlock::DecoderSecret,
          },
        },
      ]),
    }
  }
}
