use std::{fmt, fs, io, str::Utf8Error};

use jsonwebtoken::{
  errors::{Error, ErrorKind},
  jwk, Algorithm, DecodingKey, Header,
};

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

impl From<Utf8Error> for JWTError {
  fn from(value: Utf8Error) -> Self {
    JWTError::Internal(value.to_string())
  }
}

impl From<serde_json::Error> for JWTError {
  fn from(value: serde_json::Error) -> Self {
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

pub enum SecretType {
  Pem,
  Der,
  Jwks,
  B64,
  Plain,
}

pub fn get_secret_from_file_or_input(
  alg: &Algorithm,
  secret_string: &str,
) -> (JWTResult<Vec<u8>>, SecretType) {
  return match alg {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
      if secret_string.starts_with('@') {
        (
          slurp_file(strip_leading_symbol(secret_string)).map_err(JWTError::from),
          if secret_string.ends_with(".json") {
            SecretType::Jwks
          } else {
            SecretType::Plain
          },
        )
      } else if secret_string.starts_with("b64:") {
        (
          Ok(
            secret_string
              .chars()
              .skip(4)
              .collect::<String>()
              .as_bytes()
              .to_owned(),
          ),
          SecretType::B64,
        )
      } else {
        (Ok(secret_string.as_bytes().to_owned()), SecretType::Plain)
      }
    }
    _ => {
      if secret_string.starts_with('@') {
        (
          slurp_file(strip_leading_symbol(secret_string)).map_err(JWTError::from),
          get_secret_file_type(secret_string),
        )
      } else {
        // allows to read JWKS from argument (e.g. output of 'curl https://auth.domain.com/jwks.json')
        (Ok(secret_string.as_bytes().to_vec()), SecretType::Jwks)
      }
    }
  };
}

pub fn strip_leading_symbol(secret_string: &str) -> String {
  secret_string.chars().skip(1).collect::<String>()
}

pub fn decoding_key_from_jwks_secret(
  secret: &[u8],
  header: Option<Header>,
) -> JWTResult<DecodingKey> {
  if let Some(h) = header {
    return match parse_jwks(secret) {
      Some(jwks) => decoding_key_from_jwks(jwks, &h),
      None => Err(JWTError::Internal("Invalid jwks secret format".to_string())),
    };
  }
  Err(JWTError::Internal(
    "Invalid jwt header for jwks secret".to_string(),
  ))
}

pub fn slurp_file(file_name: String) -> io::Result<Vec<u8>> {
  fs::read(file_name)
}

fn decoding_key_from_jwks(jwks: jwk::JwkSet, header: &Header) -> JWTResult<DecodingKey> {
  let kid = match &header.kid {
    Some(k) => k.to_owned(),
    None => {
      return Err(JWTError::Internal(
        "Missing 'kid' from jwt header. Required for jwks secret".to_string(),
      ));
    }
  };

  let jwk = match jwks.find(&kid) {
    Some(j) => j,
    None => {
      return Err(JWTError::Internal(format!(
        "No jwk found for 'kid' {kid:?}",
      )));
    }
  };

  DecodingKey::from_jwk(jwk).map_err(Error::into)
}

fn parse_jwks(secret: &[u8]) -> Option<jwk::JwkSet> {
  match serde_json::from_slice(secret) {
    Ok(jwks) => Some(jwks),
    Err(_) => None,
  }
}

fn get_secret_file_type(secret_string: &str) -> SecretType {
  if secret_string.ends_with(".pem") {
    SecretType::Pem
  } else if secret_string.ends_with(".json") {
    SecretType::Jwks
  } else {
    SecretType::Der
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

#[cfg(test)]
mod tests {
  use std::{fs::File, io::Write};

  use super::*;

  #[test]
  fn test_slurp_file() {
    let file_name = "test.txt";
    let content = b"Hello, world!";

    let mut file = File::create(file_name).unwrap();
    file.write_all(content).unwrap();

    let result = slurp_file(file_name.to_string()).unwrap();

    assert_eq!(result, content);

    std::fs::remove_file(file_name).unwrap();
  }

  #[test]
  #[should_panic(expected = "The system cannot find the file specified.")]
  #[cfg(target_os = "windows")]
  fn test_slurp_file_nonexistent() {
    let file_name = "nonexistent.txt";

    slurp_file(file_name).unwrap();
  }

  #[test]
  #[should_panic(expected = "No such file or directory")]
  #[cfg(not(target_os = "windows"))]
  fn test_slurp_file_nonexistent() {
    let file_name = "nonexistent.txt";

    slurp_file(file_name.to_string()).unwrap();
  }
}
