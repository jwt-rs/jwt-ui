use jsonwebtoken::{errors::Error, Algorithm, EncodingKey, Header};

use super::{
  jwt_decoder::Payload,
  jwt_utils::{get_secret, JWTResult},
  models::{ScrollableTxt, TabRoute, TabsState},
  ActiveBlock, App, Route, RouteId, TextAreaInput, TextInput,
};

#[derive(Default)]
pub struct Encoder<'a> {
  pub encoded: ScrollableTxt,
  pub header: TextAreaInput<'a>,
  pub payload: TextAreaInput<'a>,
  pub secret: TextInput,
  pub signature_verified: bool,
  pub blocks: TabsState,
}

impl Encoder<'_> {
  pub fn new(secret: String) -> Self {
    let header = TextAreaInput::new(vec![
      "{".to_string(),
      r#"  "alg": "HS256","#.to_string(),
      r#"  "typ": "JWT""#.to_string(),
      "}".to_string(),
    ]);

    Self {
      header,
      secret: TextInput::new(secret),
      blocks: TabsState::new(vec![
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderHeader,
          },
        },
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderPayload,
          },
        },
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderSecret,
          },
        },
        TabRoute {
          title: "".into(),
          route: Route {
            id: RouteId::Encoder,
            active_block: ActiveBlock::EncoderToken,
          },
        },
      ]),
      ..Encoder::default()
    }
  }
}

pub fn encode_jwt_token(app: &mut App) {
  app.data.error = String::new();
  let header = app.data.encoder.header.input.lines().join("\n");
  if header.is_empty() {
    app.handle_error(String::from("Header should not be empty").into());
    return;
  }
  let header: Result<Header, serde_json::Error> = serde_json::from_str(&header);
  match header {
    Ok(header) => {
      let alg = header.alg;
      let payload = app.data.encoder.payload.input.lines().join("\n");
      if payload.is_empty() {
        app.handle_error(String::from("Payload should not be empty").into());
        return;
      }
      let payload: Result<Payload, serde_json::Error> = serde_json::from_str(&payload);
      match payload {
        Ok(payload) => {
          let secret = app.data.encoder.secret.input.value();
          let encoding_key = encoding_key_from_secret(&alg, secret);
          match encoding_key {
            Ok(encoding_key) => {
              let token = jsonwebtoken::encode(&header, &payload, &encoding_key);
              match token {
                Ok(token) => {
                  app.data.encoder.encoded = ScrollableTxt::new(token);
                  app.data.encoder.signature_verified = true;
                }
                Err(e) => app.handle_error(e.into()),
              }
            }
            Err(e) => app.handle_error(e),
          }
        }
        Err(e) => app.handle_error(format!("Error parsing payload: {:}", e).into()),
      }
    }
    Err(e) => app.handle_error(format!("Error parsing header: {:}", e).into()),
  }
}

pub fn encoding_key_from_secret(alg: &Algorithm, secret_string: &str) -> JWTResult<EncodingKey> {
  let secret = get_secret(alg, secret_string)?;

  match alg {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => Ok(EncodingKey::from_secret(&secret)),
    Algorithm::RS256
    | Algorithm::RS384
    | Algorithm::RS512
    | Algorithm::PS256
    | Algorithm::PS384
    | Algorithm::PS512 => match secret_string.ends_with(".pem") {
      true => EncodingKey::from_rsa_pem(&secret).map_err(Error::into),
      false => Ok(EncodingKey::from_rsa_der(&secret)),
    },
    Algorithm::ES256 | Algorithm::ES384 => match secret_string.ends_with(".pem") {
      true => EncodingKey::from_ec_pem(&secret).map_err(Error::into),
      false => Ok(EncodingKey::from_ec_der(&secret)),
    },
    Algorithm::EdDSA => match secret_string.ends_with(".pem") {
      true => EncodingKey::from_ed_pem(&secret).map_err(Error::into),
      false => Ok(EncodingKey::from_ed_der(&secret)),
    },
  }
}
