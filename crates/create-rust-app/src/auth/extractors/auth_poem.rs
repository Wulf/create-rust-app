use poem::{
    async_trait, http::HeaderValue, http::StatusCode, Error, FromRequest, Request, RequestBody,
    Result,
};
use std::collections::HashSet;

use super::auth::Auth;
use crate::auth::{permissions::Permission, AccessTokenClaims, ID};
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use std::iter::FromIterator;

#[async_trait]
impl<'a> FromRequest<'a> for Auth {
    /// extracts [`Auth`] from the given [`req`](`Request`)
    async fn from_request(req: &'a Request, _: &mut RequestBody) -> Result<Self> {
        let auth_header_opt: Option<&HeaderValue> = req.headers().get("Authorization");

        if auth_header_opt.is_none() {
            return Err(Error::from_string(
                "Authorization header required",
                StatusCode::UNAUTHORIZED,
            ));
        }

        let access_token_str = auth_header_opt.unwrap().to_str().unwrap_or("");

        if !access_token_str.starts_with("Bearer ") {
            return Err(Error::from_string(
                "Invalid authorization header",
                StatusCode::UNAUTHORIZED,
            ));
        }

        let access_token = decode::<AccessTokenClaims>(
            access_token_str.trim_start_matches("Bearer "),
            &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
            &Validation::default(),
        );

        if access_token.is_err() {
            return Err(Error::from_string(
                "Invalid access token",
                StatusCode::UNAUTHORIZED,
            ));
        }

        let access_token = access_token.unwrap();

        if !access_token
            .claims
            .token_type
            .eq_ignore_ascii_case("access_token")
        {
            return Err(Error::from_string(
                "Invalid access token",
                StatusCode::UNAUTHORIZED,
            ));
        }

        let user_id = access_token.claims.sub;
        let permissions: HashSet<Permission> =
            HashSet::from_iter(access_token.claims.permissions.iter().cloned());
        let roles: HashSet<String> = HashSet::from_iter(access_token.claims.roles.iter().cloned());

        return Ok(Auth {
            user_id,
            roles,
            permissions,
        });
    }
}
