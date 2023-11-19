use super::auth::Auth;
use crate::auth::{permissions::Permission, AccessTokenClaims};
use actix_web::dev::Payload;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::{FromRequest, HttpRequest, HttpResponse};
use derive_more::{Display, Error};
use futures::future::{ready, Ready};
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use serde_json::json;
use std::collections::HashSet;

#[derive(Debug, Display, Error)]
#[display(fmt = "Unauthorized ({status:?}), reason: {reason:?}")]
/// custom error type for Authorization related errors
pub struct AuthError {
    reason: String,
    status: StatusCode,
}

impl AuthError {
    #[must_use]
    pub const fn new(reason: String, status: StatusCode) -> Self {
        Self { reason, status }
    }

    #[must_use]
    pub const fn reason(reason: String) -> Self {
        Self {
            reason,
            status: StatusCode::UNAUTHORIZED,
        }
    }
}

impl ResponseError for AuthError {
    /// builds an [`HttpResponse`] for [`self`](`AuthError`)
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(
            json!({
              "message": self.reason.as_str()
            })
            .to_string(),
        )
    }

    /// return the [`StatusCode`] associated with an [`AuthError`]
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

impl FromRequest for Auth {
    type Future = Ready<Result<Self, Self::Error>>;
    type Error = AuthError;

    /// extracts [`Auth`] from the given [`req`](`HttpRequest`)
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> <Self as FromRequest>::Future {
        let access_token_str = match req.headers().get("Authorization").map(|h| h.to_str()) {
            Some(Ok(auth_header)) if auth_header.starts_with("Bearer ") => auth_header,
            Some(_) => {
                return ready(Err(AuthError::reason(
                    "Invalid authorization header".to_string(),
                )))
            }
            None => {
                return ready(Err(AuthError::reason(
                    "Authorization header required".to_string(),
                )))
            }
        };

        let access_token = match decode::<AccessTokenClaims>(
            access_token_str.trim_start_matches("Bearer "),
            &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
            &Validation::default(),
        ) {
            Ok(token) if token.claims.token_type.eq_ignore_ascii_case("access_token") => token,
            _ => return ready(Err(AuthError::reason("Invalid access token".to_string()))),
        };

        let user_id = access_token.claims.sub;
        let permissions: HashSet<Permission> =
            access_token.claims.permissions.iter().cloned().collect();
        let roles: HashSet<String> = access_token.claims.roles.iter().cloned().collect();

        ready(Ok(Self {
            user_id,
            roles,
            permissions,
        }))
    }
}
