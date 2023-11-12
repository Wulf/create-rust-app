//!
//! This package contains all the utility functions that
//! are exposed directly as `create_rust_app::`<utilty-fn>.
//!

#[cfg(feature = "backend_actix-web")]
mod actix_web_utils;

#[cfg(feature = "backend_actix-web")]
pub use actix_web_utils::*;

#[cfg(feature = "backend_poem")]
mod poem_utils;

#[cfg(feature = "backend_poem")]
pub use poem_utils::*;

/// expose `template_utils` for all backends.
mod template_utils;

pub mod net;
