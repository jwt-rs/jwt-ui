// adapted from https://github.com/mike-engel/jwt-cli
use std::{
  collections::{BTreeMap, HashSet},
  fmt, io,
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

use super::{utils::slurp_file, App};

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

#[derive(Debug, Clone)]
pub struct DecodeArgs {
  /// The JWT to decode.
  pub jwt: String,
  /// The algorithm used to sign the JWT
  pub algorithm: Algorithm,
  /// Display unix timestamps as ISO 8601 UTC dates
  pub time_format_utc: bool,
  /// The secret to validate the JWT with. Prefix with @ to read from a file or b64: to use base-64 encoded bytes
  pub secret: String,
  /// Render the decoded JWT as JSON
  pub json: bool,
  /// Ignore token expiration date (`exp` claim) during validation
  pub ignore_exp: bool,
}

impl DecodeArgs {
  pub fn new(jwt: String) -> Self {
    DecodeArgs {
      jwt,
      algorithm: Algorithm::HS256,
      time_format_utc: false,
      secret: "".to_string(),
      json: false,
      ignore_exp: true,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Payload(pub BTreeMap<String, Value>);

impl Payload {
  //   pub fn from_payloads(payloads: Vec<PayloadItem>) -> Payload {
  //     let mut payload = BTreeMap::new();

  //     for PayloadItem(k, v) in payloads {
  //       payload.insert(k, v);
  //     }

  //     Payload(payload)
  //   }

  pub fn convert_timestamps(&mut self) {
    let timestamp_claims: Vec<String> = vec!["iat".into(), "nbf".into(), "exp".into()];

    for (key, value) in self.0.iter_mut() {
      if timestamp_claims.contains(key) && value.is_number() {
        *value = match value.as_i64() {
          Some(timestamp) => Utc.timestamp_opt(timestamp, 0).unwrap().to_rfc3339().into(),
          None => value.clone(),
        }
      }
    }
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TokenOutput {
  pub header: Header,
  pub payload: Payload,
}

impl TokenOutput {
  pub fn new(data: TokenData<Payload>) -> Self {
    TokenOutput {
      header: data.header,
      payload: data.claims,
    }
  }
}

pub fn decoding_key_from_secret(alg: &Algorithm, secret_string: &str) -> JWTResult<DecodingKey> {
  match alg {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
      if secret_string.starts_with('@') {
        let secret = slurp_file(&secret_string.chars().skip(1).collect::<String>());
        Ok(DecodingKey::from_secret(&secret))
      } else if secret_string.starts_with("b64:") {
        Ok(DecodingKey::from_secret(
          &base64_engine
            .decode(secret_string.chars().skip(4).collect::<String>())
            .unwrap(),
        ))
      } else {
        Ok(DecodingKey::from_secret(secret_string.as_bytes()))
      }
    }
    Algorithm::RS256
    | Algorithm::RS384
    | Algorithm::RS512
    | Algorithm::PS256
    | Algorithm::PS384
    | Algorithm::PS512 => {
      if !&secret_string.starts_with('@') {
        return Err(JWTError::Internal(format!(
          "Secret for {alg:?} must be a file path starting with @",
        )));
      }

      let secret = slurp_file(&secret_string.chars().skip(1).collect::<String>());

      match secret_string.ends_with(".pem") {
        true => DecodingKey::from_rsa_pem(&secret).map_err(Error::into),
        false => Ok(DecodingKey::from_rsa_der(&secret)),
      }
    }
    Algorithm::ES256 | Algorithm::ES384 => {
      if !&secret_string.starts_with('@') {
        return Err(JWTError::Internal(format!(
          "Secret for {alg:?} must be a file path starting with @",
        )));
      }

      let secret = slurp_file(&secret_string.chars().skip(1).collect::<String>());

      match secret_string.ends_with(".pem") {
        true => DecodingKey::from_ec_pem(&secret).map_err(Error::into),
        false => Ok(DecodingKey::from_ec_der(&secret)),
      }
    }
    Algorithm::EdDSA => {
      if !&secret_string.starts_with('@') {
        return Err(JWTError::Internal(format!(
          "Secret for {alg:?} must be a file path starting with @",
        )));
      }

      let secret = slurp_file(&secret_string.chars().skip(1).collect::<String>());

      match secret_string.ends_with(".pem") {
        true => DecodingKey::from_ed_pem(&secret).map_err(Error::into),
        false => Ok(DecodingKey::from_ed_der(&secret)),
      }
    }
  }
}

pub fn decode_jwt_token(app: &mut App, token: String) {
  let out = decode_token(&DecodeArgs::new(token));
  match out {
    Ok(decoded_token) => {
      app.error = String::new();
      app.data.decoded_token = Some(decoded_token);
    }
    Err(e) => app.handle_error(e),
  };
}

pub fn print_decoded_token(token: &TokenData<Payload>, json: bool) {
  match json {
    true => {
      println!("\nToken JSON\n----------");
      println!(
        "{}",
        to_string_pretty(&TokenOutput::new(token.clone())).unwrap()
      )
    }
    false => {
      println!("\nToken header\n------------");
      println!("{}\n", to_string_pretty(&token.header).unwrap());
      println!("Token claims\n------------");
      println!("{}", to_string_pretty(&token.claims).unwrap());
    }
  }
}

fn decode_token(arguments: &DecodeArgs) -> JWTResult<TokenData<Payload>> {
  let algorithm = arguments.algorithm;
  let secret = match arguments.secret.len() {
    0 => None,
    _ => Some(decoding_key_from_secret(&algorithm, &arguments.secret)),
  };
  let jwt = match arguments.jwt.as_str() {
    "-" => {
      let mut buffer = String::new();

      io::stdin()
        .read_line(&mut buffer)
        .expect("STDIN was not valid UTF-8");

      buffer
    }
    _ => arguments.jwt.clone(),
  }
  .trim()
  .to_owned();

  let mut secret_validator = Validation::new(algorithm);

  secret_validator.leeway = 1000;

  if arguments.ignore_exp {
    secret_validator
      .required_spec_claims
      .retain(|claim| claim != "exp");
    secret_validator.validate_exp = false;
  }

  let mut insecure_validator = secret_validator.clone();
  let insecure_decoding_key = DecodingKey::from_secret("".as_ref());

  insecure_validator.insecure_disable_signature_validation();
  insecure_validator.required_spec_claims = HashSet::new();
  insecure_validator.validate_exp = false;

  let token_data = match secret {
    Some(Ok(secret_key)) => {
      decode::<Payload>(&jwt, &secret_key, &secret_validator).map_err(Error::into)
    }
    Some(Err(err)) => Err(err),
    None => {
      decode::<Payload>(&jwt, &insecure_decoding_key, &insecure_validator).map_err(Error::into)
    }
  };

  token_data.map(|mut token| {
    if arguments.time_format_utc {
      token.claims.convert_timestamps();
    }
    token
  })
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
