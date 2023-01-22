use poem::{
    async_trait, http::HeaderValue, http::StatusCode, Error, FromRequest, Request, RequestBody,
    Result,
};
use std::collections::HashSet;

use crate::auth::{permissions::Permission, AccessTokenClaims, ID};
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
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
            permission: permission.to_string(),
            from_role: String::new(),
        })
    }

    /// does the user with the id [`self.user_id`](`ID`) have all of the given `perms`
    pub fn has_all_permissions(&self, perms: Vec<String>) -> bool {
        perms.iter().all(|p| self.has_permission(p.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have any of the given `perms`
    pub fn has_any_permissions(&self, perms: Vec<String>) -> bool {
        perms.iter().any(|p| self.has_permission(p.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have the given `role`
    pub fn has_role(&self, role: String) -> bool {
        self.roles.contains(&role.to_string())
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
