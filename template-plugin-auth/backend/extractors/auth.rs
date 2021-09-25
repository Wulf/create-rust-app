use jsonwebtoken::Validation;
use jsonwebtoken::DecodingKey;
use crate::models::permissions::Permission;
use crate::services::auth::AccessTokenClaims;
use actix_http::http::HeaderValue;
use actix_web::dev::Payload;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::{FromRequest, HttpRequest, HttpResponse};
use derive_more::{Display, Error};
use futures_util::future::{ready, Ready};
use crate::models::ID;
use serde_json::json;
use jsonwebtoken::decode;

#[derive(Debug, Clone, Default)]
pub struct Config();

impl Config {}

#[derive(Debug, Clone)]
pub struct Auth {
  pub user_id: ID,
  pub permissions: Vec<Permission>
}

impl Auth {
  pub fn has_permission(&self, permission: &str) -> bool {
    self.permissions.iter().find(|perm| perm.permission == permission).is_some()
  }
}

#[derive(Debug, Display, Error)]
#[display(fmt = "Unauthorized, reason: {}", self.reason)]
pub struct AuthError {
  reason: String,
}

impl ResponseError for AuthError {
  fn error_response(&self) -> HttpResponse {
    // HttpResponse::Unauthorized().json(self.reason.as_str())
    // println!("error_response");
    HttpResponse::build(StatusCode::UNAUTHORIZED).body(json!({
      "message": self.reason.as_str()
    }))
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

    if auth_header_opt.is_none() {
      return ready(Err(AuthError {
        reason: "Authorization header required".to_string(),
      }));
    }

    let access_token_str = auth_header_opt.unwrap().to_str().unwrap_or("");

    let access_token = decode::<AccessTokenClaims>(
      access_token_str,
      &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
      &Validation::default()
    );

    if access_token.is_err() {
      return ready(Err(AuthError {
        reason: "Invalid access token".to_string()
      }))
    }

    let access_token = access_token.unwrap();

    if !access_token.claims.token_type.eq_ignore_ascii_case("access_token") {
      return ready(Err(AuthError {
        reason: "Invalid access token".to_string()
      }))
    }
  
    let user_id = access_token.claims.sub;
    let permissions = access_token.claims.permissions;

    return ready(
      Ok(Auth {
        user_id: user_id,
        permissions
      })
    );
  }
}
