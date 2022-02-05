#[cfg(feature = "backend_poem")]
mod endpoints_poem;
#[cfg(feature = "backend_poem")]
pub use endpoints_poem::api;

#[cfg(feature = "backend_actix-web")]
mod endpoints_actixweb;
#[cfg(feature = "backend_actix-web")]
pub use endpoints_actixweb::endpoints;
