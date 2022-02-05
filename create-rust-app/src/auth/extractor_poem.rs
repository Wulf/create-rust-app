use poem::{async_trait, http::HeaderValue, http::StatusCode, Error, FromRequest, Request, RequestBody, Result};

use super::{permissions::Permission, AccessTokenClaims, ID};
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;

#[derive(Debug, Clone)]
pub struct Auth {
    pub user_id: ID,
    pub permissions: Vec<Permission>,
}

impl Auth {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions
            .iter()
            .find(|perm| perm.permission == permission)
            .is_some()
    }
}

#[async_trait]
impl<'a> FromRequest<'a> for Auth {
    async fn from_request(req: &'a Request, _: &mut RequestBody) -> Result<Self> {
        let auth_header_opt: Option<&HeaderValue> = req.headers().get("Authorization");

        if auth_header_opt.is_none() {
            return Err(Error::from_string(
                "Authorization header required",
                StatusCode::UNAUTHORIZED,
            ));
        }

        let access_token_str = auth_header_opt.unwrap().to_str().unwrap_or("");

        let access_token = decode::<AccessTokenClaims>(
            access_token_str,
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
        let permissions = access_token.claims.permissions;

        return Ok(Auth {
            user_id: user_id,
            permissions,
        });
    }
}
