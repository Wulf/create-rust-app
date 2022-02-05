extern crate argonautica;

// Auth guard / extractor

#[cfg(feature = "backend_actix-web")]
mod extractor_actixweb;
#[cfg(feature = "backend_actix-web")]
pub use extractor_actixweb::Auth;

#[cfg(feature = "backend_poem")]
mod extractor_poem;
#[cfg(feature = "backend_poem")]
pub use extractor_poem::Auth;

// api endpoint definitions

#[cfg(feature = "backend_actix-web")]
mod endpoints_actixweb;
#[cfg(feature = "backend_actix-web")]
pub use endpoints_actixweb::endpoints;

#[cfg(feature = "backend_poem")]
mod endpoints_poem;
#[cfg(feature = "backend_poem")]
pub use endpoints_poem::api;

mod mail;
mod permissions;
mod user;
mod user_session;

pub use permissions::Permission;


#[tsync::tsync]
type ID = i32;

#[tsync::tsync]
type UTC = chrono::DateTime<chrono::Utc>;

#[tsync::tsync]
#[derive(serde::Deserialize)]
pub struct PaginationParams {
    pub page: i64,
    pub page_size: i64,
}

impl PaginationParams {
    const MAX_PAGE_SIZE: u16 = 100;
}

#[tsync::tsync]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UserSessionJson {
    pub id: ID,
    pub device: Option<String>,
    pub created_at: UTC,
    pub updated_at: UTC,
}

#[tsync::tsync]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UserSessionResponse {
    pub sessions: Vec<UserSessionJson>,
    pub num_pages: i64
}

#[tsync::tsync]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AccessTokenClaims {
    pub exp: usize,
    pub sub: ID,
    pub token_type: String,
    pub permissions: Vec<Permission>
}
