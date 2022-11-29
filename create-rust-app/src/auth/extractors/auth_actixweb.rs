use crate::auth::{permissions::Permission, AccessTokenClaims, ID};
use actix_http::header::HeaderValue;
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
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct Auth {
    pub user_id: ID,
    pub roles: HashSet<String>,
    pub permissions: HashSet<Permission>,
}

impl Auth {
    pub fn has_permission(&self, permission: String) -> bool {
        self.permissions.contains(&Permission {
            permission,
            from_role: String::new(),
        })
    }

    pub fn has_all_permissions(&self, perms: Vec<String>) -> bool {
        perms.iter().all(|p| self.has_permission(p.to_string()))
    }

    pub fn has_any_permission(&self, perms: Vec<String>) -> bool {
        perms.iter().any(|p| self.has_permission(p.to_string()))
    }

    pub fn has_role(&self, permission: String) -> bool {
        self.roles.contains(&permission)
    }

    pub fn has_all_roles(&self, roles: Vec<String>) -> bool {
        roles.iter().all(|r| self.has_role(r.to_string()))
    }

    pub fn has_any_roles(&self, roles: Vec<String>) -> bool {
        roles.iter().any(|r| self.has_role(r.to_string()))
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
        HttpResponse::build(StatusCode::UNAUTHORIZED).body(
            json!({
              "message": self.reason.as_str()
            })
            .to_string(),
        )
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

impl FromRequest for Auth {
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
            &Validation::default(),
        );

        if access_token.is_err() {
            return ready(Err(AuthError {
                reason: "Invalid access token".to_string(),
            }));
        }

        let access_token = access_token.unwrap();

        if !access_token
            .claims
            .token_type
            .eq_ignore_ascii_case("access_token")
        {
            return ready(Err(AuthError {
                reason: "Invalid access token".to_string(),
            }));
        }

        let user_id = access_token.claims.sub;
        let permissions: HashSet<Permission> =
            HashSet::from_iter(access_token.claims.permissions.iter().cloned());
        let roles: HashSet<String> = HashSet::from_iter(access_token.claims.roles.iter().cloned());

        ready(Ok(Auth {
            user_id,
            roles,
            permissions,
        }))
    }
}
