#[cfg(feature = "backend_actix-web")]
mod service_actixweb;
#[cfg(feature = "backend_actix-web")]
pub use service_actixweb::endpoints;
#[cfg(feature = "backend_actix-web")]
pub use service_actixweb::ApiDoc;

#[cfg(feature = "backend_poem")]
mod service_poem;
#[cfg(feature = "backend_poem")]
pub use service_poem::api;
