use serde::{Deserialize, Serialize};

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

pub use permissions::{
    Permission, Role, RolePermission, RolePermissionChangeset, UserPermission,
    UserPermissionChangeset,
};
pub use user::{User, UserChangeset};
pub use user_session::{UserSession, UserSessionChangeset};

#[tsync::tsync]
type ID = i32;

#[tsync::tsync]
#[cfg(not(feature = "database_sqlite"))]
type Utc = chrono::DateTime<chrono::Utc>;
#[cfg(feature = "database_sqlite")]
type Utc = chrono::NaiveDateTime;

#[tsync::tsync]
#[derive(Deserialize)]
/// Rust struct that provides the information needed to allow 
/// pagination of results for requests that have a lot of results
/// 
/// often times, GET requests to a REST API will have a lot of 
/// results to return, pagination allows the server to break up
/// those results into smaller chunks that can be more easily 
/// sent to, and used by, the client
pub struct PaginationParams {
    pub page: i64,
    pub page_size: i64,
}

impl PaginationParams {
    const MAX_PAGE_SIZE: u16 = 100;
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Rust struct representation of a entry from the databases user_session table 
/// serialized into Json
pub struct UserSessionJson {
    pub id: ID,
    pub device: Option<String>,
    pub created_at: Utc,
    #[cfg(not(feature = "database_sqlite"))]
    pub updated_at: Utc,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Rust struct representation of the 
/// backends JSON response to a GET request at the /sessions endpoint
pub struct UserSessionResponse {
    pub sessions: Vec<UserSessionJson>,
    pub num_pages: i64,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize)]
/// TODO: documentation
pub struct AccessTokenClaims {
    pub exp: usize,
    pub sub: ID,
    pub token_type: String,
    pub roles: Vec<String>,
    pub permissions: Vec<Permission>,
}
