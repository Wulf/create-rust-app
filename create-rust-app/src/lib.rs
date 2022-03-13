#[cfg(all(feature = "backend_actix-web", feature = "backend_poem"))]
compile_error!("feature \"backend_actix-web\" and feature \"backend_poem\" cannot be enabled at the same time");

// #[cfg(not(any(feature = "backend_poem", feature = "backend_actix-web")))]
// compile_error!("Please enable one of the backend features (options: 'backend_actix-web', 'backend-poem')");

#[macro_use]
extern crate diesel;

#[cfg(feature = "plugin_auth")]
pub mod auth;

#[cfg(all(feature = "plugin_dev", debug_assertions))]
pub mod dev;

mod database;
pub use database::{Database, Connection};

#[cfg(feature = "backend_poem")]
mod logger;
#[cfg(feature = "backend_poem")]
pub use logger::Logger as PoemLogger;

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
        if dotenv::dotenv().is_err() {
            panic!("ERROR: Could not load environment variables from dotenv file");
        }

        #[cfg(feature = "backend_actix-web")]
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
        // diesel_migrations::embed_migrations!();
    }

    #[cfg(feature="plugin_auth")]
    if std::env::var("SECRET_KEY").is_err() {
        panic!("No SECRET_KEY environment variable set!");
    }

    if std::env::var("DATABASE_URL").is_err() {
        panic!("No DATABASE_URL environment variable set!");
    }

    Mailer::check_environment_variables();

    AppData {
        mailer: Mailer::new(),
        database: Database::new(),
    }
}


#[cfg(feature = "backend_actix-web")]
use actix_web;

#[cfg(feature = "backend_actix-web")]
pub async fn not_found() -> actix_web::HttpResponse {
    actix_web::HttpResponse::NotFound().finish()
}

#[cfg(feature = "backend_poem")]
use poem;

#[cfg(feature = "backend_poem")]
pub async fn not_found(_: poem::error::NotFoundError) -> poem::Response {
    let json = serde_json::json!({
        "success": false,
        "message": "Invalid endpoint"
    });

    poem::Response::builder()
        .status(poem::http::StatusCode::NOT_FOUND)
        .header("Content-Type", "application/json")
        .body(json.to_string())
}
