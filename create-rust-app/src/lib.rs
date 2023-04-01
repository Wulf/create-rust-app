#[cfg(all(feature = "backend_actix-web", feature = "backend_poem"))]
compile_error!(
    "feature \"backend_actix-web\" and feature \"backend_poem\" cannot be enabled at the same time"
);

#[cfg(all(feature = "database_sqlite", feature = "database_postgres"))]
compile_error!(
    "feature \"database_sqlite\" and feature \"database_postgres\" cannot be enabled at the same time"
);

// #[cfg(not(any(feature = "backend_poem", feature = "backend_actix-web")))]
// compile_error!("Please enable one of the backend features (options: 'backend_actix-web', 'backend-poem')");

mod util;
pub use util::*;

#[macro_use]
extern crate diesel;

#[cfg(feature = "plugin_auth")]
pub mod auth;

#[cfg(feature = "plugin_tasks")]
pub mod tasks;

#[cfg(all(feature = "plugin_dev", debug_assertions))]
pub mod dev;
#[cfg(all(feature = "plugin_dev", debug_assertions))]
pub use dev::setup_development;

mod database;
pub use database::{Connection, Database, Pool};

#[cfg(feature = "backend_poem")]
mod logger;
#[allow(deprecated)] // deprecated; we're going to roll out better logging soon. Use your own tracing setup for now!
#[cfg(feature = "backend_poem")]
pub use logger::Logger as PoemLogger;

#[cfg(feature = "plugin_storage")]
mod storage;
#[cfg(feature = "plugin_storage")]
pub use storage::{Attachment, AttachmentBlob, AttachmentData, Storage};

mod mailer;
pub use mailer::Mailer;

// #[cfg(debug_assertions)]
// #[macro_use]
// extern crate diesel_migrations;

#[derive(Clone)]
///
pub struct AppData {
    /// wrapper for SMTP mailing server accessed by chosen web framework
    ///
    /// see [`Mailer`]
    pub mailer: Mailer,
    /// db agnostic wrapper for databases accessed by chosen web framework
    ///
    /// see [`Database`]
    pub database: Database,
    #[cfg(feature = "plugin_storage")]
    /// wrapper for Amazon S3 cloud file storage service accessed by chosen web framework
    ///
    /// see [`Storage`]
    pub storage: Storage,
}

#[cfg(debug_assertions)]
fn load_env_vars() {
    static START: std::sync::Once = std::sync::Once::new();

    START.call_once(|| {
        dotenv::dotenv().unwrap_or_else(|_| {
            panic!("ERROR: Could not load environment variables from dotenv file");
        });
    });
}

/// ensures required environment variables are present,
///  
/// initialize a [`Mailer`], [`Database`], and [`Storage`] (is `Storage` plugin was enabled ("plugin_storage" feature enabled))
///
/// and wraps them in a [`AppData`] struct that is then returned
pub fn setup() -> AppData {
    // Only load dotenv in development
    #[cfg(debug_assertions)]
    {
        load_env_vars();

        // #[cfg(feature = "backend_actix-web")]
        // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    #[cfg(feature = "plugin_auth")]
    if std::env::var("SECRET_KEY").is_err() {
        panic!("No SECRET_KEY environment variable set!");
    }

    if std::env::var("DATABASE_URL").is_err() {
        panic!("No DATABASE_URL environment variable set!");
    }

    AppData {
        mailer: Mailer::new(),
        database: Database::new(),
        #[cfg(feature = "plugin_storage")]
        storage: Storage::new(),
    }
}

#[cfg(feature = "backend_poem")]
use poem;

#[cfg(feature = "backend_poem")]
/// TODO: documentation
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
