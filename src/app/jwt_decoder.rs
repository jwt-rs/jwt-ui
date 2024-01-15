use std::{
  collections::{BTreeMap, HashSet},
  str::from_utf8,
};

use chrono::{TimeZone, Utc};
use jsonwebtoken::{
  decode, decode_header, errors::Error, Algorithm, DecodingKey, Header, TokenData, Validation,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_string_pretty, Value};

use super::{
  models::{BlockState, ScrollableTxt},
  utils::{
    decoding_key_from_jwks_secret, get_secret_from_file_or_input, JWTError, JWTResult, SecretType,
  },
  ActiveBlock, App, Route, RouteId, TextInput,
};

#[derive(Default)]
pub struct Decoder {
  pub encoded: TextInput,
  pub header: ScrollableTxt,
  pub payload: ScrollableTxt,
  pub secret: TextInput,
  pub signature_verified: bool,
  pub blocks: BlockState,
  pub utc_dates: bool,
  pub ignore_exp: bool,
  /// do not manipulate directly, use `set_decoded` instead
  decoded: Option<TokenData<Payload>>,
}

impl Decoder {
  pub fn new(token: Option<String>, secret: String) -> Self {
    Self {
      encoded: TextInput::new(token.unwrap_or_default()),
      secret: TextInput::new(secret),
      ignore_exp: true,
      blocks: BlockState::new(vec![
        Route {
          id: RouteId::Decoder,
          active_block: ActiveBlock::DecoderToken,
        },
        Route {
          id: RouteId::Decoder,
          active_block: ActiveBlock::DecoderSecret,
        },
        Route {
          id: RouteId::Decoder,
          active_block: ActiveBlock::DecoderHeader,
        },
        Route {
          id: RouteId::Decoder,
          active_block: ActiveBlock::DecoderPayload,
        },
      ]),
      ..Decoder::default()
    }
  }

  pub fn is_decoded(&self) -> bool {
    self.decoded.is_some()
  }

  pub fn get_decoded(&self) -> Option<TokenData<Payload>> {
    self.decoded.clone()
  }

  pub fn set_decoded(&mut self, decoded: Option<TokenData<Payload>>) {
    match decoded.as_ref() {
      Some(payload) => {
        let header = to_string_pretty(&payload.header).unwrap();
        if header != self.header.get_txt() {
          self.header = ScrollableTxt::new(header);
        }
        let payload = to_string_pretty(&payload.claims).unwrap();
        if payload != self.payload.get_txt() {
          self.payload = ScrollableTxt::new(payload);
        }
      }
      None => {
        self.header = ScrollableTxt::default();
        self.payload = ScrollableTxt::default();
      }
    }
    self.decoded = decoded;
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
struct TokenOutput {
  pub header: Header,
  pub payload: Payload,
}

impl TokenOutput {
  fn new(data: TokenData<Payload>) -> Self {
    TokenOutput {
      header: data.header,
      payload: data.claims,
    }
  }
}

#[derive(Debug, Clone)]
struct DecodeArgs {
  /// The JWT to decode.
  pub jwt: String,
  /// Display unix timestamps as ISO 8601 UTC dates
  pub time_format_utc: bool,
  /// The secret to validate the JWT with. Prefix with @ to read from a file or b64: to use base-64 encoded bytes
  pub secret: String,
  /// Ignore token expiration date (`exp` claim) during validation
  pub ignore_exp: bool,
}

/// decode the given JWT token and verify its signature if secret is provided
pub fn decode_jwt_token(app: &mut App) {
  let token = app.data.decoder.encoded.input.value();
  if !token.is_empty() {
    let secret = app.data.decoder.secret.input.value();

    let out = decode_token(&DecodeArgs {
      jwt: token.into(),
      secret: secret.into(),
      time_format_utc: app.data.decoder.utc_dates,
      ignore_exp: app.data.decoder.ignore_exp,
    });
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
  let header = match decode_header(&arguments.jwt) {
    Ok(header) => Some(header),
    Err(_) => None,
  };

  let algorithm = header.as_ref().map(|h| h.alg).unwrap_or(Algorithm::HS256);

  let mut insecure_validator = Validation::new(algorithm);

  // disable signature validation as its not needed for just decoding
  insecure_validator.insecure_disable_signature_validation();
  insecure_validator.required_spec_claims = HashSet::new();
  insecure_validator.validate_exp = false;
  insecure_validator.validate_aud = false;

  let insecure_decoding_key = match algorithm {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => Ok(DecodingKey::from_secret(b"")),
    Algorithm::ES256 | Algorithm::ES384 => DecodingKey::from_ec_components("", ""),
    Algorithm::EdDSA => DecodingKey::from_ed_components(""),
    _ => DecodingKey::from_rsa_components("", ""),
  }
  .map_or(DecodingKey::from_secret(b""), |key| key);

  let decode_only = decode::<Payload>(&arguments.jwt, &insecure_decoding_key, &insecure_validator)
    .map_err(Error::into);

  let decode_only = decode_only.map(|mut token| {
    if arguments.time_format_utc {
      token.claims.convert_timestamps();
    }
    token
  });

  let secret = match arguments.secret.len() {
    0 => None,
    _ => Some(decoding_key_from_secret(
      &algorithm,
      &arguments.secret,
      header,
    )),
  };

  let mut secret_validator = Validation::new(algorithm);

  secret_validator.leeway = 1000;
  secret_validator.validate_aud = false;

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

fn decoding_key_from_secret(
  alg: &Algorithm,
  secret_string: &str,
  header: Option<Header>,
) -> JWTResult<DecodingKey> {
  let (secret, file_type) = get_secret_from_file_or_input(alg, secret_string);
  let secret = secret?;
  match alg {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => match file_type {
      SecretType::Plain => Ok(DecodingKey::from_secret(&secret)),
      SecretType::Jwks => decoding_key_from_jwks_secret(&secret, header),
      SecretType::B64 => DecodingKey::from_base64_secret(from_utf8(&secret)?).map_err(Error::into),
      _ => Err(JWTError::Internal(format!(
        "Invalid secret file type for {alg:?}"
      ))),
    },
    Algorithm::RS256
    | Algorithm::RS384
    | Algorithm::RS512
    | Algorithm::PS256
    | Algorithm::PS384
    | Algorithm::PS512 => match file_type {
      SecretType::Pem => DecodingKey::from_rsa_pem(&secret).map_err(Error::into),
      SecretType::Der => Ok(DecodingKey::from_rsa_der(&secret)),
      SecretType::Jwks => decoding_key_from_jwks_secret(&secret, header),
      _ => Err(JWTError::Internal(format!(
        "Invalid secret file type for {alg:?}"
      ))),
    },
    Algorithm::ES256 | Algorithm::ES384 => match file_type {
      SecretType::Pem => DecodingKey::from_ec_pem(&secret).map_err(Error::into),
      SecretType::Der => Ok(DecodingKey::from_ec_der(&secret)),
      SecretType::Jwks => decoding_key_from_jwks_secret(&secret, header),
      _ => Err(JWTError::Internal(format!(
        "Invalid secret file type for {alg:?}"
      ))),
    },
    Algorithm::EdDSA => match file_type {
      SecretType::Pem => DecodingKey::from_ed_pem(&secret).map_err(Error::into),
      SecretType::Der => Ok(DecodingKey::from_ed_der(&secret)),
      SecretType::Jwks => decoding_key_from_jwks_secret(&secret, header),
      _ => Err(JWTError::Internal(format!(
        "Invalid secret file type for {alg:?}"
      ))),
    },
  }
}

#[cfg(test)]
mod tests {
  use std::{fs::File, io::Write};

  use super::*;

  #[test]
  fn test_decode_hs256_token_with_valid_jwt_and_secret() {
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
  fn test_decode_hs256_token_with_valid_jwt_and_b64secret() {
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
  fn test_decode_rs256_token_with_valid_jwt() {
    let args = DecodeArgs {
            jwt: String::from("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IkRGbzcxemxOdV9vLTkxOFJIN0lIVyJ9.eyJodHRwczovL3d3dy5qaGlwc3Rlci50ZWNoL3JvbGVzIjpbIkFkbWluaXN0cmF0b3IiLCJST0xFX0FETUlOIiwiUk9MRV9VU0VSIl0sImlzcyI6Imh0dHBzOi8vZGV2LTA2YnpzMWN1LnVzLmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2MWJjYmM3NmY2NGQ0YTAwNzJhZjhhMWQiLCJhdWQiOlsiaHR0cHM6Ly9kZXYtMDZienMxY3UudXMuYXV0aDAuY29tL2FwaS92Mi8iLCJodHRwczovL2Rldi0wNmJ6czFjdS51cy5hdXRoMC5jb20vdXNlcmluZm8iXSwiaWF0IjoxNzA1MDAyMDQxLCJleHAiOjE3MDUwODg0NDEsImF6cCI6IjFmbTdJMUdHRXRNZlRabW5vdFV1azVVT3gyWm10NnR0Iiwic2NvcGUiOiJvcGVuaWQifQ.eWdbVEolnmqqyx_Z5rR-09H3kg06EaokYoAAdrqLmB6FHwZbbyZrPaHImmEnY8BSRM42FpE9NZehqVAeQ5VQhOVdMMklCQSA5h13oQbKn6ciuc9Etyq2jg4sk2lOEkSmw4e_hWUGjkXnzP_J84o9-2qpN7VKNTGEvtk3mdQYXxwoeD8RvQjYJq6LsKIKA0biEyGWZxIpK1LCAFH1dmo5ZMpTeNGIwnUBdOxkL4jbKe26e9t7TDO0EtFjXmq-C218bbr1AgFN2eyj6n-3kNy9XfRcnfIlyXWJ0ZvcDVa9UoaTGP9Wdo0Ze3q2IrcgYrP7zTeZia5O2tejkaNknKNnwA"),
            secret: "".into(),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data
      .unwrap_err()
      .to_string()
      .contains("The JWT provided has an invalid signature: InvalidSignature"));

    let decode_only_token = decode_only.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::RS256);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("scope").unwrap()),
      "String(\"openid\")"
    );
  }

  #[test]
  fn test_decode_es384_token_with_valid_jwt_and_secret_pem() {
    let secret_file_name = "./test_data/test_ecdsa_public_key.pem";

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
  fn test_decode_rs256_token_with_valid_jwt_and_secret_der() {
    let secret_file_name = "./test_data/test_rsa_public_key.der";

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
  fn test_decode_rsa_token_using_jwks_secret_file() {
    let secret_file_name = "./test_data/test_rsa_public_jwks.json";

    let args = DecodeArgs {
            jwt: String::from("eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6IjJjYUZjUHgtYVhhQzZTZXZoVjc5VURJcnM4TGdVb2syeG8wQTZESlBxSm8ifQ.eyJleHAiOjE3MDUwNzg3MzMsImZpZWxkIjoidmFsdWUiLCJpYXQiOjE3MDUwNzY5MzN9.iQIMqpDqsvBfVI1lL83GR1ihXaWcRuv4yrIqEWS6k_zjm2Pt2EsLTB1C2QA66oZgc0pIX_sOZ4S-4fGKNmKrBz5UCNH7v5aXqHA7kvgh5CaFx7kAosIhQZWzt2O_Ca9T-G6uQNvKKBOcdfSfTGKt464TbjWS_knbHj-aQC-eKu7uhJTy0ercu3eqIGJFCNj2BdhtXNrACcDoTzZZsjvEvXgr9qRtHbaghJL6l1rF3cm_q9O8GWd_7cWtQC8yrKinZNz2P4O_PBqeDKDjApmZPqORU_gBaN9RmmU6Z0jHq68oeAprl6PfJdUkCR-q8UrHJofRKtAEiRcTTy60YdiJCw"),
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
  fn test_decode_rsa_ssa_pss_token_using_jwks_secret() {
    let jwks = r#"{"keys":[{"use":"sig","kty":"RSA","kid":"2caFcPx-aXaC6SevhV79UDIrs8LgUok2xo0A6DJPqJo","n":"589r2P-JpeFPkH2T8-SBw7ttzHPPlVzqJwb_fcXJl8MGZ_7Jkt8k58Ukgp3cgRdChDNlnrFeXu1wSwU47Mf_o9bBLVQbNCJ7uL-vQYdFwzEipqHusywJ-Qm5qpJyWO5f2hXMHnomZ1KZW4isg7g1kvynUznlSwU25wNUvRurRImxigT2ohmZzHf37n51zyzci5JZxneOojcyfXdhDWtRGuSbREW3XZqKnJbUOK9HqosrgidbFZil3j2uf4br7DLtdlZMJ4JzTE_ZX273el_uv_XFg-OuHvgdBHtgzN9rkKapkPyUT0BsWfOPyjEtrjzdAAiFQfuwhwIWQPidzBUKtw","e":"AQAB"},{"use":"enc","kty":"RSA","kid":"2caFcPx-aXaC6SevhV79UDIrs8LgUok2xo0A6DJPqJo","n":"589r2P-JpeFPkH2T8-SBw7ttzHPPlVzqJwb_fcXJl8MGZ_7Jkt8k58Ukgp3cgRdChDNlnrFeXu1wSwU47Mf_o9bBLVQbNCJ7uL-vQYdFwzEipqHusywJ-Qm5qpJyWO5f2hXMHnomZ1KZW4isg7g1kvynUznlSwU25wNUvRurRImxigT2ohmZzHf37n51zyzci5JZxneOojcyfXdhDWtRGuSbREW3XZqKnJbUOK9HqosrgidbFZil3j2uf4br7DLtdlZMJ4JzTE_ZX273el_uv_XFg-OuHvgdBHtgzN9rkKapkPyUT0BsWfOPyjEtrjzdAAiFQfuwhwIWQPidzBUKtw","e":"AQAB"}]}"#;

    let args = DecodeArgs {
            jwt: String::from("eyJ0eXAiOiJKV1QiLCJraWQiOiIyY2FGY1B4LWFYYUM2U2V2aFY3OVVESXJzOExnVW9rMnhvMEE2REpQcUpvIiwiYWxnIjoiUFM1MTIifQ.eyJmaWVsZCI6InZhbHVlIiwiZm9vIjoiYmFyIn0.O6r-pK6rDw0BAadqJmBivtjk7ELU2pYpKIOU7qD8rah9mzwm29A0KoCoOabtQCkKNcmlcIKoC812UrP_nDZrAsC1msHPfjvkKlbkX63_zEcRCv-6VC1FMuek8yY6mhKiFaTISPDBfHCg_Fru2BDar_qBJn8rtct9y6cgDA5vLvL81jLmJrCXW8C5wP9xrkG5CUXdW9A8fqtxcEDoNZoYUoxCnLkh3Pz5IfAluepqDYjj6kvMWuAC88K1B_a1Z8QTqCuJZNIj_5g6UExmK7pqKvB5RZo62KGTw8wWqkmaPTf4TnD4n3Rb1K-MN1LTWMySqgPaw5YlSxT2eFwDvhRBnA"),
            secret: jwks.into(),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_ok());

    let decode_only_token = decode_only.unwrap();
    let verified_token_data = verified_token_data.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::PS512);
    assert_eq!(verified_token_data.header.alg, Algorithm::PS512);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("field").unwrap()),
      "String(\"value\")"
    );
  }

  #[test]
  fn test_decode_ecdsa_token_using_jwks_secret_file() {
    let secret_file_name = "./test_data/test_ecdsa_public_jwks.json";

    let args = DecodeArgs {
            jwt: String::from("eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiIsImtpZCI6IjRoN3d0MklISHVfUkxSNk90bFpqQ2VfbUl0OHhBUmVTMGNERXd3V0FlS1UifQ.eyJleHAiOjE3MDUwNzkyNTEsImZpZWxkIjoidmFsdWUiLCJpYXQiOjE3MDUwNzc0NTF9.-HzKN93IVNfNg6fasPQm382o-CqelRsPLu3t59kl3LCWRkYzSwV9GZMPEkVtl0VPS5hhtE4d7b8Ho-YsdCGVWg"),
            secret: format!("@{}", secret_file_name),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_ok());

    let decode_only_token = decode_only.unwrap();
    let verified_token_data = verified_token_data.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::ES256);
    assert_eq!(verified_token_data.header.alg, Algorithm::ES256);
    assert_eq!(
      format!("{:?}", decode_only_token.claims.0.get("field").unwrap()),
      "String(\"value\")"
    );
  }

  #[test]
  fn test_decode_eddsa_token_using_secret_file() {
    let secret_file_name = "./test_data/test_eddsa_public_key.pem";

    let args = DecodeArgs {
            jwt: String::from("eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9.eyJleHAiOjE3MDUwOTMyMzMsImZpZWxkIjoidmFsdWUiLCJpYXQiOjE3MDUwOTE0MzN9.1EpR_PbE2SeK87hCk15QeZ7p5E6_2mWi4NhO6R0ixFdouW_-hunEQdYCu2YzaKRZKqHFiuuuIGidEaMw3mq-AA"),
            secret: format!("@{}", secret_file_name),
            time_format_utc: false,
            ignore_exp: true,
        };

    let (decode_only, verified_token_data) = decode_token(&args);

    assert!(decode_only.is_ok());
    assert!(verified_token_data.is_ok());

    let decode_only_token = decode_only.unwrap();
    let verified_token_data = verified_token_data.unwrap();

    assert_eq!(decode_only_token.header.alg, Algorithm::EdDSA);
    assert_eq!(verified_token_data.header.alg, Algorithm::EdDSA);
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

    let result = decoding_key_from_secret(&alg, secret, None);

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

    let result = decoding_key_from_secret(&alg, &secret_string, None);

    assert!(result.is_ok());

    std::fs::remove_file(secret_file_name).unwrap();
  }

  #[test]
  fn test_decoding_key_from_secret_rs256_file_pem() {
    let secret_file_name = "./test_data/test_ecdsa_public_key.pem";
    let alg = Algorithm::ES384;

    let secret_string = format!("@{}", secret_file_name);

    let result = decoding_key_from_secret(&alg, &secret_string, None);

    assert!(result.is_ok());
  }

  #[test]
  #[should_panic(expected = "Invalid jwks secret format")]
  fn test_decoding_key_from_secret_es256_no_file() {
    let secret = "mysecret";
    let alg = Algorithm::ES256;

    decoding_key_from_secret(&alg, secret, Some(Header::default())).unwrap();
  }

  #[test]
  #[should_panic(expected = "The system cannot find the file specified. (os error 2)")]
  #[cfg(target_os = "windows")]
  fn test_decoding_key_from_secret_nonexistent_file() {
    let secret_file_name = "nonexistent.txt";
    let alg = Algorithm::HS256;

    let secret_string = format!("@{}", secret_file_name);

    decoding_key_from_secret(&alg, &secret_string, None).unwrap();
  }

  #[test]
  #[should_panic(expected = "No such file or directory (os error 2)")]
  #[cfg(not(target_os = "windows"))]
  fn test_decoding_key_from_secret_nonexistent_file() {
    let secret_file_name = "nonexistent.txt";
    let alg = Algorithm::HS256;

    let secret_string = format!("@{}", secret_file_name);

    decoding_key_from_secret(&alg, &secret_string, None).unwrap();
  }
}
