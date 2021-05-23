#[macro_use]
extern crate diesel;

mod schema;
mod services;
mod util;
mod models;
mod extractors;
mod mail;

use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::HttpResponse;
use actix_web::{web, App, HttpServer};
use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use diesel::PgConnection;
use env_logger::Env;
use serde::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DB = PooledConnection<ConnectionManager<PgConnection>>;

async fn app_index() -> actix_web::Result<NamedFile, actix_web::Error> {
    Ok(NamedFile::open("./app/build/index.html")?)
}

#[derive(Serialize, Deserialize)]
struct HealthCheckResponse {
    message: String
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(not(debug_assertions))]
    env::set_var("RUST_BACKTRACE", "1");

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable expected.");
    let database_pool = Pool::builder()
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(database_pool.clone())
            .service(
                web::scope("/api")
                    .service(services::todos_service::endpoints(web::scope("/todos")))
                    .route("/health", web::get().to(|| HttpResponse::Ok().json(HealthCheckResponse { message: "healthy".to_string() })))
            )
            .service(
                Files::new("*", "./app/build")
                    .index_file("index.html")
                    .default_handler(web::get().to(app_index))
            )
            .default_service(web::route().to(|| HttpResponse::NotFound()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
