// adapted from https://github.com/mike-engel/jwt-cli
use std::{
  collections::{BTreeMap, HashSet},
  fmt,
};

use base64::{engine::general_purpose::STANDARD as base64_engine, Engine as _};
use chrono::{TimeZone, Utc};
use jsonwebtoken::{
  decode,
  errors::{Error, ErrorKind},
  Algorithm, DecodingKey, Header, TokenData, Validation,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_string_pretty, Value};
use tui_input::Input;

use super::{
  models::{ScrollableTxt, TabRoute, TabsState},
  utils::slurp_file,
  ActiveBlock, App, InputMode, Route, RouteId, TextInput,
};

#[derive(Default)]
pub struct Encoder {}
