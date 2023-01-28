// TODO: add .gitignore (and other ignore files) parsing functinality
// TODO: add "create-module" functionality (so generated types can be under a specified namespace like Rust.MyType)
extern crate syn;

pub mod hook;
pub mod params;
pub mod processor;
pub use processor::process;

