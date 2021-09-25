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
use mail::Mailer;

#[cfg(debug_assertions)]
#[macro_use]
extern crate diesel_migrations;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DB = PooledConnection<ConnectionManager<PgConnection>>;

#[cfg(not(debug_assertions))]
async fn app_index() -> actix_web::Result<NamedFile, actix_web::Error> {
    Ok(NamedFile::open("./frontend/build/index.html")?)
}

#[cfg(debug_assertions)]
async fn development_index(req: web::HttpRequest) -> actix_web::Result<NamedFile, actix_web::Error> {
    Ok(NamedFile::open(".cargo/admin/dist/admin.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    if dotenv::dotenv().is_err() {
        panic!("ERROR: Could not load environment variables from dotenv file");
    }

    diesel_migrations::embed_migrations!();
    
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mailer = Mailer::new();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable expected.");
    let database_pool = Pool::builder()
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .unwrap();

    HttpServer::new(move || {
        let app = App::new();

        let app = app.wrap(Logger::default());
        let app = app.data(database_pool.clone());
        let app = app.data(mailer.clone());

        let mut api_scope = web::scope("/api")
            .service(services::auth::endpoints(web::scope("/auth")))
            .service(services::todos_service::endpoints(web::scope("/todos")));

        #[cfg(debug_assertions)]
        {
            // Mount development-only routes
            api_scope = api_scope.service(services::development::endpoints(web::scope("/development")));
        };

        let mut app = app.service(api_scope);

        #[cfg(not(debug_assertions))]
        {
            app = app.service(Files::new("*", "./frontend/build").index_file("index.html").default_handler(web::get().to(app_index)));
        }
        
        #[cfg(debug_assertions)] {
            app = app
                .service(
                    Files::new("*", ".cargo/admin/dist").index_file("admin.html").default_handler(web::get().to(development_index))
                );
        }

        let app = app.default_service(web::route().to(|| HttpResponse::NotFound()));
        app
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
