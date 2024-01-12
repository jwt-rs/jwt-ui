use std::fmt;

use base64::{engine::general_purpose::STANDARD as base64_engine, Engine as _};
use jsonwebtoken::{
  errors::{Error, ErrorKind},
  jwk, Algorithm, DecodingKey, Header,
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

pub enum SecretFileType {
  Pem,
  Der,
  Jwks,
  Na,
}

pub fn get_secret(alg: &Algorithm, secret_string: &str) -> (JWTResult<Vec<u8>>, SecretFileType) {
  return match alg {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
      if secret_string.starts_with('@') {
        (
          slurp_file(&secret_string.chars().skip(1).collect::<String>()).map_err(JWTError::from),
          SecretFileType::Na,
        )
      } else if secret_string.starts_with("b64:") {
        (
          base64_engine
            .decode(secret_string.chars().skip(4).collect::<String>())
            .map_err(JWTError::from),
          SecretFileType::Na,
        )
      } else {
        (Ok(secret_string.as_bytes().to_owned()), SecretFileType::Na)
      }
    }
    Algorithm::EdDSA => {
      if !&secret_string.starts_with('@') {
        return (
          Err(JWTError::Internal(format!(
            "Secret for {alg:?} must be a file path starting with @",
          ))),
          SecretFileType::Na,
        );
      }

      (
        slurp_file(&secret_string.chars().skip(1).collect::<String>()).map_err(JWTError::from),
        get_secret_file_type(secret_string),
      )
    }
    _ => {
      if secret_string.starts_with('@') {
        (
          slurp_file(&secret_string.chars().skip(1).collect::<String>()).map_err(JWTError::from),
          get_secret_file_type(secret_string),
        )
      } else {
        // allows to read JWKS from argument (e.g. output of 'curl https://auth.domain.com/jwks.json')
        (Ok(secret_string.as_bytes().to_vec()), SecretFileType::Jwks)
      }
    }
  };
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

  match &jwk.algorithm {
    jwk::AlgorithmParameters::RSA(rsa) => {
      DecodingKey::from_rsa_components(&rsa.n, &rsa.e).map_err(Error::into)
    }
    jwk::AlgorithmParameters::EllipticCurve(ec) => {
      DecodingKey::from_ec_components(&ec.x, &ec.y).map_err(Error::into)
    }
    _ => Err(JWTError::Internal("Unsupported alg".to_string())),
  }
}

fn parse_jwks(secret: &[u8]) -> Option<jwk::JwkSet> {
  match serde_json::from_slice(secret) {
    Ok(jwks) => Some(jwks),
    Err(_) => None,
  }
}

fn get_secret_file_type(secret_string: &str) -> SecretFileType {
  if secret_string.ends_with(".pem") {
    SecretFileType::Pem
  } else if secret_string.ends_with(".json") {
    SecretFileType::Jwks
  } else {
    SecretFileType::Der
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
