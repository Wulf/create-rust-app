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
/// roles and permissions available to a User
///
/// use to control what users are and are not allowed to do
pub struct Auth {
    pub user_id: ID,
    pub roles: HashSet<String>,
    pub permissions: HashSet<Permission>,
}

impl Auth {
    /// does the user with the id [`self.user_id`](`ID`) have the given `permission`
    pub fn has_permission(&self, permission: String) -> bool {
        self.permissions.contains(&Permission {
            permission,
            from_role: String::new(),
        })
    }

    /// does the user with the id [`self.user_id`](`ID`) have all of the given `perms`
    pub fn has_all_permissions(&self, perms: Vec<String>) -> bool {
        perms.iter().all(|p| self.has_permission(p.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have any of the given `perms`
    pub fn has_any_permission(&self, perms: Vec<String>) -> bool {
        perms.iter().any(|p| self.has_permission(p.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have the given `role`
    pub fn has_role(&self, role: String) -> bool {
        self.roles.contains(&role)
    }

    /// does the user with the id [`self.user_id`](`ID`) have all of the given `roles`
    pub fn has_all_roles(&self, roles: Vec<String>) -> bool {
        roles.iter().all(|r| self.has_role(r.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have any of the given `roles`
    pub fn has_any_roles(&self, roles: Vec<String>) -> bool {
        roles.iter().any(|r| self.has_role(r.to_string()))
    }
}

#[derive(Debug, Display, Error)]
#[display(fmt = "Unauthorized, reason: {}", self.reason)]
/// custom error type for Authorization related errors
pub struct AuthError {
    reason: String,
}

impl ResponseError for AuthError {
    /// builds an [`HttpResponse`] for [`self`](`AuthError`)
    fn error_response(&self) -> HttpResponse {
        // HttpResponse::Unauthorized().json(self.reason.as_str())
        // println!("error_response");
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
        let auth_header_opt: Option<&HeaderValue> = req.headers().get("Authorization");

        if auth_header_opt.is_none() {
            return ready(Err(AuthError {
                reason: "Authorization header required".to_string(),
            }));
        }

        let access_token_str = auth_header_opt.unwrap().to_str().unwrap_or("");

        if !access_token_str.starts_with("Bearer ") {
            return ready(Err(AuthError {
                reason: "Invalid authorization header".to_string(),
            }));
        }

        let access_token = decode::<AccessTokenClaims>(
            access_token_str.trim_start_matches("Bearer "),
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
