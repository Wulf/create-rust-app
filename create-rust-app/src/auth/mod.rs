extern crate argonautica;
use serde::{Serialize, Deserialize};

// Auth guard / extractor
mod extractors;
pub use extractors::*;

// api endpoint definitions
pub mod controller;
mod endpoints;
pub use endpoints::*;

mod mail;
mod permissions;
mod schema;
mod user;
mod user_session;

pub use user::{User, UserChangeset};
pub use user_session::{UserSession, UserSessionChangeset};
pub use permissions::{Role, Permission, UserPermission, UserPermissionChangeset, RolePermission, RolePermissionChangeset};

#[tsync::tsync]
type ID = i32;

#[tsync::tsync]
type UTC = chrono::DateTime<chrono::Utc>;

#[tsync::tsync]
#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: i64,
    pub page_size: i64,
}

impl PaginationParams {
    const MAX_PAGE_SIZE: u16 = 100;
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionJson {
    pub id: ID,
    pub device: Option<String>,
    pub created_at: UTC,
    pub updated_at: UTC,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionResponse {
    pub sessions: Vec<UserSessionJson>,
    pub num_pages: i64
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub exp: usize,
    pub sub: ID,
    pub token_type: String,
    pub roles: Vec<String>,
    pub permissions: Vec<Permission>
}
