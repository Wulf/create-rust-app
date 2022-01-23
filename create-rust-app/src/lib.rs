#[macro_use]
extern crate diesel;

#[cfg(feature = "plugin-auth")]
pub mod schema;

#[cfg(feature = "plugin-auth")]
pub mod auth;

#[cfg(all(feature = "plugin-dev", debug_assertions))]
pub mod dev;

mod database;
pub use database::{Database, Connection};

mod logger;
pub use logger::Logger;

mod mailer;
pub use mailer::Mailer;

// #[cfg(debug_assertions)]
// #[macro_use]
// extern crate diesel_migrations;

#[derive(Clone)]
pub struct AppData {
    pub mailer: Mailer,
    pub database: Database,
}

pub fn setup() -> AppData {
    // Only load dotenv in development
    #[cfg(debug_assertions)]
    {
        if std::env::var_os("RUST_LOG").is_none() {
            std::env::set_var("RUST_LOG", "poem=debug");
        }

        if dotenv::dotenv().is_err() {
            panic!("ERROR: Could not load environment variables from dotenv file");
        }

        // diesel_migrations::embed_migrations!();
    }

    tracing_subscriber::fmt::init();

    Mailer::check_environment_variables();

    AppData {
        mailer: Mailer::new(),
        database: Database::new(),
    }
}

pub async fn default_404_handler(_: poem::error::NotFoundError) -> poem::Response {
    let json = serde_json::json!({
        "success": false,
        "message": "Invalid endpoint"
    });

    poem::Response::builder()
        .status(poem::http::StatusCode::NOT_FOUND)
        .header("Content-Type", "application/json")
        .body(json.to_string())
}
