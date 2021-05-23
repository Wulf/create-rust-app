use crate::Pool;
use actix_http::http::HeaderValue;
use actix_web::dev::Payload;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::{FromRequest, HttpRequest, HttpResponse};
use derive_more::{Display, Error};
use futures_util::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use crate::models::ID;

#[derive(Debug, Clone, Default)]
pub struct Config();

impl Config {}

#[derive(Debug, Clone)]
pub struct Auth {
  pub email: String
}

#[derive(Debug, Display, Error)]
#[display(fmt = "Unauthorized, reason: {}", self.reason)]
pub struct AuthError {
  reason: String,
}

impl ResponseError for AuthError {
  fn error_response(&self) -> HttpResponse {
    HttpResponse::Unauthorized().json(self.reason.as_str())
  }

  fn status_code(&self) -> StatusCode {
    StatusCode::UNAUTHORIZED
  }
}

impl FromRequest for Auth {
  type Config = Config;
  type Future = Ready<Result<Self, Self::Error>>;
  type Error = AuthError;

  fn from_request(req: &HttpRequest, _payload: &mut Payload) -> <Self as FromRequest>::Future {
    let auth_header_opt: Option<&HeaderValue> = req.headers().get("Authorization");

    if auth_header_opt.is_none() || auth_header_opt.unwrap().len() < 1 {
      return ready(Err(AuthError {
        reason: "Invalid authorization header".to_string(),
      }));
    }

    let auth_header = auth_header_opt.unwrap().to_str().unwrap_or("").to_string();

    return ready(
      Ok(Auth {
        email: "".to_owned()
      })
    );
  }
}
