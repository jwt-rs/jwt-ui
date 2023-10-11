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
  jwt_utils::{JWTError, JWTResult},
  models::{ScrollableTxt, TabRoute, TabsState},
  utils::slurp_file,
  ActiveBlock, App, InputMode, Route, RouteId, TextInput,
};

pub struct Decoder {
  pub encoded: TextInput,
  pub header: ScrollableTxt,
  pub payload: ScrollableTxt,
  pub secret: TextInput,
  pub signature_verified: bool,
  pub blocks: TabsState,
  /// do not manipulate directly, use `set_decoded` instead
  pub _decoded: Option<TokenData<Payload>>,
}

impl Default for Decoder {
  fn default() -> Self {
    Self {
      encoded: TextInput {
        input: Input::default(),
        input_mode: InputMode::Normal,
      },
      header: Default::default(),
      payload: Default::default(),
      secret: Default::default(),
      signature_verified: Default::default(),
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
            active_block: ActiveBlock::DecoderSignature,
          },
        },
      ]),
      _decoded: Default::default(),
    }
  }
}

impl Decoder {
  pub fn is_decoded(&self) -> bool {
    self._decoded.is_some()
  }
  pub fn get_decoded(&self) -> Option<TokenData<Payload>> {
    self._decoded.clone()
  }
  pub fn set_decoded(&mut self, decoded: Option<TokenData<Payload>>) {
    match decoded.as_ref() {
      Some(payload) => {
        self.header = ScrollableTxt::new(to_string_pretty(&payload.header).unwrap());
        self.payload = ScrollableTxt::new(to_string_pretty(&payload.claims).unwrap())
      }
      None => {
        self.header = ScrollableTxt::default();
        self.payload = ScrollableTxt::default();
      }
    }
    self._decoded = decoded;
  }
}

#[derive(Debug, Clone)]
pub struct DecodeArgs {
  /// The JWT to decode.
  pub jwt: String,
  /// Display unix timestamps as ISO 8601 UTC dates
  pub time_format_utc: bool,
  /// The secret to validate the JWT with. Prefix with @ to read from a file or b64: to use base-64 encoded bytes
  pub secret: String,
  /// Ignore token expiration date (`exp` claim) during validation
  pub ignore_exp: bool,
}

impl DecodeArgs {
  pub fn new(jwt: String, secret: String) -> Self {
    DecodeArgs {
      jwt,
      secret,
      time_format_utc: false,
      ignore_exp: true,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Payload(pub BTreeMap<String, Value>);

impl Payload {
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

pub fn decode_jwt_token(app: &mut App, token: String, secret: String) {
  if !token.is_empty() {
    let out = decode_token(&DecodeArgs::new(token, secret));
    match out {
      (Ok(decoded), Ok(_)) => {
        app.data.error = String::new();
        app.data.decoder.signature_verified = true;
        app.data.decoder.set_decoded(Some(decoded));
      }
      (Ok(decoded), Err(e)) => {
        app.handle_error(e);
        app.data.decoder.signature_verified = false;
        app.data.decoder.set_decoded(Some(decoded));
      }
      (Err(e), _) => {
        app.handle_error(e);
        app.data.decoder.signature_verified = false;
        app.data.decoder.set_decoded(None);
      }
    };
  }
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

/// returns the base64 decoded values and signature verified result
fn decode_token(
  arguments: &DecodeArgs,
) -> (JWTResult<TokenData<Payload>>, JWTResult<TokenData<Payload>>) {
  let mut insecure_validator = Validation::new(Algorithm::HS256);
  // disable signature validation as its not needed for just decoding
  insecure_validator.insecure_disable_signature_validation();
  insecure_validator.required_spec_claims = HashSet::new();
  insecure_validator.validate_exp = false;
  let insecure_decoding_key = DecodingKey::from_secret("".as_ref());

  let decode_only = decode::<Payload>(&arguments.jwt, &insecure_decoding_key, &insecure_validator)
    .map_err(Error::into);

  let decode_only = decode_only.map(|mut token| {
    if arguments.time_format_utc {
      token.claims.convert_timestamps();
    }
    token
  });

  let algorithm = match decode_only.as_ref() {
    Ok(data) => data.header.alg.clone(),
    Err(_) => Algorithm::HS256,
  };

  let secret = match arguments.secret.len() {
    0 => None,
    _ => Some(decoding_key_from_secret(&algorithm, &arguments.secret)),
  };

  let mut secret_validator = Validation::new(algorithm);

  secret_validator.leeway = 1000;

  if arguments.ignore_exp {
    secret_validator
      .required_spec_claims
      .retain(|claim| claim != "exp");
    secret_validator.validate_exp = false;
  }

  let verified_token_data = match secret {
    Some(Ok(secret_key)) => {
      decode::<Payload>(&arguments.jwt, &secret_key, &secret_validator).map_err(Error::into)
    }
    Some(Err(err)) => Err(err),
    None => decode::<Payload>(&arguments.jwt, &insecure_decoding_key, &secret_validator)
      .map_err(Error::into),
  };

  (decode_only, verified_token_data)
}

fn decoding_key_from_secret(alg: &Algorithm, secret_string: &str) -> JWTResult<DecodingKey> {
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

#[cfg(test)]
mod tests {
  use std::{fs::File, io::Write};

  use super::*;

  #[test]
  fn test_decode_token_with_valid_jwt_and_secret_hs256() {
    let args = DecodeArgs {
            jwt: String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"),
            secret: String::from("your-256-bit-secret"),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_ok());

    let decode_only_token = decode_only.unwrap();
    let verified_token_data = verified_token_data.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::HS256);
    assert_eq!(verified_token_data.header.alg, Algorithm::HS256);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("name").unwrap()),
      "String(\"John Doe\")"
    );
  }

  #[test]
  fn test_decode_token_with_valid_jwt_and_b64secret_hs256() {
    let args = DecodeArgs {
            jwt: String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.DCwemWTIxJURgfU0rFIIo20__ZAhQbl3ZpQ44nf6Mqs"),
            secret: String::from("b64:eW91ci0yNTYtYml0LXNlY3JldAo="),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_ok());

    let decode_only_token = decode_only.unwrap();
    let verified_token_data = verified_token_data.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::HS256);
    assert_eq!(verified_token_data.header.alg, Algorithm::HS256);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("name").unwrap()),
      "String(\"John Doe\")"
    );
  }

  #[test]
  fn test_decode_token_with_valid_jwt_and_secret_es384_pem() {
    let secret_file_name = "./test_data/test_ecdsa_public.pem";

    let args = DecodeArgs {
            jwt: String::from("eyJhbGciOiJFUzM4NCIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.VUPWQZuClnkFbaEKCsPy7CZVMh5wxbCSpaAWFLpnTe9J0--PzHNeTFNXCrVHysAa3eFbuzD8_bLSsgTKC8SzHxRVSj5eN86vBPo_1fNfE7SHTYhWowjY4E_wuiC13yoj"),
            secret: format!("@{}", secret_file_name),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_ok());

    let decode_only_token = decode_only.unwrap();
    let verified_token_data = verified_token_data.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::ES384);
    assert_eq!(verified_token_data.header.alg, Algorithm::ES384);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("name").unwrap()),
      "String(\"John Doe\")"
    );
  }

  #[test]
  fn test_decode_token_with_valid_jwt_and_secret_rs256_der() {
    let secret_file_name = "./test_data/test_rsa_public.der";

    let args = DecodeArgs {
            jwt: String::from("eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJleHAiOjE2OTY5NzExNzgsImZpZWxkIjoidmFsdWUiLCJpYXQiOjE2OTY5NjkzNzh9.HL0TsttFnWgfXexoMofB0pXBbN4ABD7nYb0MUMZVwnGn4OU6Zi8PzVbGnIevBU73xrgDiyG4jEWJw5Ra88y0BBd99U9VXhv9g5ky10Imt9dhwkfHnJ7AqWEHueidSWLUObvyLuv2Tu01xc8NbPJq1ggYLWhJp4ap7G2huM6uQ5wB199CqZ4MGefNFgwH9gbUjMEeT5CJ0DXFDVR2ySwJRsBTJsjanDrXpNA2svI-UCmhO2WVa-ArZW0QUm0fQzm5VuQJ87C2Y5l7u1r73ckrQnm_B5OLT4Erqu7DFs7kr0rOVenbRYtllsDYs79hj_mFypZebuLhqtdgtxPiYOeKww"),
            secret: format!("@{}", secret_file_name),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_ok());

    let decode_only_token = decode_only.unwrap();
    let verified_token_data = verified_token_data.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::RS256);
    assert_eq!(verified_token_data.header.alg, Algorithm::RS256);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("field").unwrap()),
      "String(\"value\")"
    );
  }

  #[test]
  fn test_decode_token_with_valid_jwt_and_empty_secret() {
    let args = DecodeArgs {
            jwt: String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"),
            secret: String::from(""),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_err());

    let decode_only_token = decode_only.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::HS256);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("name").unwrap()),
      "String(\"John Doe\")"
    );
  }

  #[test]
  fn test_decode_token_with_invalid_jwt() {
    let args = DecodeArgs {
      jwt: String::from("invalid_jwt"),
      secret: String::from("secret"),
      time_format_utc: false,
      ignore_exp: true,
    };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_err());
    assert!(verified_token_data.is_err());
  }

  #[test]
  fn test_decode_token_with_valid_jwt_and_invalid_secret() {
    let args = DecodeArgs {
            jwt: String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"),
            secret: String::from("invalid_secret"),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_err());

    let decode_only_token = decode_only.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::HS256);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("name").unwrap()),
      "String(\"John Doe\")"
    );
  }

  #[test]
  fn test_decode_token_with_valid_jwt_and_valid_exp_utc() {
    let args = DecodeArgs {
            jwt: String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"),
            secret: String::from("your-256-bit-secret"),
            time_format_utc: true,
            ignore_exp: false,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_err());

    let decode_only_token = decode_only.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::HS256);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("iat").unwrap()),
      "String(\"2018-01-18T01:30:22+00:00\")"
    );
  }

  #[test]
  fn test_decoding_key_from_secret_hs256() {
    let secret = "mysecret";
    let alg = Algorithm::HS256;

    let result = decoding_key_from_secret(&alg, secret);

    assert!(result.is_ok());
  }

  #[test]
  fn test_decoding_key_from_secret_hs256_file() {
    let secret_file_name = "test.txt";
    let secret_content = b"mysecret";
    let alg = Algorithm::HS256;

    let mut secret_file = File::create(secret_file_name).unwrap();
    secret_file.write_all(secret_content).unwrap();

    let secret_string = format!("@{}", secret_file_name);

    let result = decoding_key_from_secret(&alg, &secret_string);

    assert!(result.is_ok());

    std::fs::remove_file(secret_file_name).unwrap();
  }

  #[test]
  fn test_decoding_key_from_secret_rs256_file_pem() {
    let secret_file_name = "./test_data/test_ecdsa_public.pem";
    let alg = Algorithm::ES384;

    let secret_string = format!("@{}", secret_file_name);

    let result = decoding_key_from_secret(&alg, &secret_string);

    assert!(result.is_ok());
  }

  #[test]
  #[should_panic(expected = "Secret for ES256 must be a file path starting with @")]
  fn test_decoding_key_from_secret_es256_no_file() {
    let secret = "mysecret";
    let alg = Algorithm::ES256;

    decoding_key_from_secret(&alg, secret).unwrap();
  }

  #[test]
  #[should_panic(expected = "Unable to read file")]
  fn test_decoding_key_from_secret_nonexistent_file() {
    let secret_file_name = "nonexistent.txt";
    let alg = Algorithm::HS256;

    let secret_string = format!("@{}", secret_file_name);

    decoding_key_from_secret(&alg, &secret_string).unwrap();
  }
}
