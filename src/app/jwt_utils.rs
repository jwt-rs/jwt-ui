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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JWTError {
  Internal(String),
  External(Error, String),
}

pub type JWTResult<T> = Result<T, JWTError>;

impl From<Error> for JWTError {
  fn from(value: Error) -> Self {
    let msg = map_external_error(&value);
    JWTError::External(value, msg)
  }
}

impl From<serde_json::Error> for JWTError {
  fn from(value: serde_json::Error) -> Self {
    JWTError::Internal(value.to_string())
  }
}

impl fmt::Display for JWTError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      JWTError::Internal(err) => write!(f, "{err}"),
      JWTError::External(err, msg) => write!(f, "{msg}: {err}"),
    }
  }
}

fn map_external_error(ext_err: &Error) -> String {
  return match ext_err.kind() {
        ErrorKind::InvalidToken => {
          "The JWT provided is invalid".to_string()
        }
        ErrorKind::InvalidSignature => {
          "The JWT provided has an invalid signature".to_string()
        }
        ErrorKind::InvalidRsaKey(_) => {
          "The secret provided isn't a valid RSA key".to_string()
        }
        ErrorKind::InvalidEcdsaKey => {
          "The secret provided isn't a valid ECDSA key".to_string()
        }
        ErrorKind::MissingRequiredClaim(missing) => if missing.as_str() == "exp" {
          "`exp` is missing, but is required. This error can be ignored via the `--ignore-exp` parameter.".to_string()
        } else {
          format!("`{:?}` is missing, but is required", missing)
        }
        ErrorKind::ExpiredSignature => {
          "The token has expired (or the `exp` claim is not set). This error can be ignored via the `--ignore-exp` parameter.".to_string()
        }
        ErrorKind::InvalidIssuer => {
          "The token issuer is invalid".to_string()
        }
        ErrorKind::InvalidAudience => {
          "The token audience doesn't match the subject".to_string()
        }
        ErrorKind::InvalidSubject => {
          "The token subject doesn't match the audience".to_string()
        }
        ErrorKind::ImmatureSignature => {
          "The `nbf` claim is in the future which isn't allowed".to_string()
        }
        ErrorKind::InvalidAlgorithm => "The JWT provided has a different signing algorithm than the one you provided".to_string(),
        _ => format!("The JWT provided is invalid because {:?}", ext_err),
      };
}
