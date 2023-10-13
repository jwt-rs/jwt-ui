use super::{
  models::{ScrollableTxt, TabsState},
  TextInput,
};

#[derive(Default)]
pub struct Encoder {
  pub encoded: ScrollableTxt,
  pub header: TextInput,
  pub payload: TextInput,
  pub secret: TextInput,
  pub signature_verified: bool,
  pub blocks: TabsState,
}
