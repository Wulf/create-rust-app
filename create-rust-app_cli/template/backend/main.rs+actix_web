extern crate diesel;

use actix_files::{Files};
use actix_web::{App, HttpServer, web};
use actix_web::middleware::{Compress, Logger, TrailingSlash, NormalizePath};
use actix_web::web::Data;
use create_rust_app::AppConfig;

mod schema;
mod services;
mod models;
mod mail;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)] create_rust_app::setup_development().await;
    let app_data = create_rust_app::setup();
    simple_logger::init_with_env().unwrap();

    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Compress::default())
            .wrap(NormalizePath::new(TrailingSlash::MergeOnly))
            .wrap(Logger::default());

        app = app.app_data(Data::new(app_data.database.clone()));
        app = app.app_data(Data::new(app_data.mailer.clone()));
        app = app.app_data(Data::new(AppConfig {
            app_url: std::env::var("APP_URL").unwrap(),
        }));

        let mut api_scope = web::scope("/api");
        api_scope = api_scope.service(services::todo::endpoints(web::scope("/todos")));

        #[cfg(debug_assertions)]
        {
            /* Development-only routes */
        }

        app = app.service(api_scope);
        app = app.default_service(web::get().to(create_rust_app::render_views));
        app
    }).bind("0.0.0.0:3000")?.run().await
}
