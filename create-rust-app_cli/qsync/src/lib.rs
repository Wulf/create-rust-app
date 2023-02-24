extern crate syn;

mod hook;
mod params;
mod processor;
mod utils;
pub use processor::process;

/// the #[qsync] attribute macro which marks structs and types to be translated into queries
pub use qsync_macro::qsync;
