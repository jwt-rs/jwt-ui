use std::fmt;

use base64::{engine::general_purpose::STANDARD as base64_engine, Engine as _};
use jsonwebtoken::{
  errors::{Error, ErrorKind},
  Algorithm,
};

use super::utils::slurp_file;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JWTError {
  Internal(String),
  External(Error, String),
}

pub type JWTResult<T> = Result<T, JWTError>;

impl From<jsonwebtoken::errors::Error> for JWTError {
  fn from(value: jsonwebtoken::errors::Error) -> Self {
    let msg = map_external_error(&value);
    JWTError::External(value, msg)
  }
}

impl From<serde_json::Error> for JWTError {
  fn from(value: serde_json::Error) -> Self {
    JWTError::Internal(value.to_string())
  }
}

impl From<base64::DecodeError> for JWTError {
  fn from(value: base64::DecodeError) -> Self {
    JWTError::Internal(value.to_string())
  }
}

impl From<std::io::Error> for JWTError {
  fn from(value: std::io::Error) -> Self {
    JWTError::Internal(value.to_string())
  }
}

impl From<String> for JWTError {
  fn from(value: String) -> Self {
    JWTError::Internal(value)
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

pub fn get_secret(alg: &Algorithm, secret_string: &str) -> JWTResult<Vec<u8>> {
  return match alg {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
      if secret_string.starts_with('@') {
        slurp_file(&secret_string.chars().skip(1).collect::<String>()).map_err(JWTError::from)
      } else if secret_string.starts_with("b64:") {
        base64_engine
          .decode(secret_string.chars().skip(4).collect::<String>())
          .map_err(JWTError::from)
      } else {
        Ok(secret_string.as_bytes().to_owned())
      }
    }
    _ => {
      if !&secret_string.starts_with('@') {
        return Err(JWTError::Internal(format!(
          "Secret for {alg:?} must be a file path starting with @",
        )));
      }

      slurp_file(&secret_string.chars().skip(1).collect::<String>()).map_err(JWTError::from)
    }
  };
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
          "`exp` is missing, but is required. This error can be ignored by pressing `i`.".to_string()
        } else {
          format!("`{:?}` is missing, but is required", missing)
        }
        ErrorKind::ExpiredSignature => {
          "The token has expired (or the `exp` claim is not set). This error can be ignored by pressing `i`.".to_string()
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
