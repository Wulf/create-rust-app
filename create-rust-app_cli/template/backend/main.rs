#[macro_use]
extern crate diesel;

use poem::endpoint::{StaticFilesEndpoint};
use poem::{
    listener::TcpListener,
    middleware::{AddData, CookieJarManager},
    EndpointExt, Route, Server,
};

mod extractors;
mod mail;
mod models;
mod schema;
mod services;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let data = create_rust_app::setup();

    let mut api = Route::new()
        .nest("/todos", services::todo::api());

    let mut app = Route::new().nest("/api", api);

    #[cfg(not(debug_assertions))]
    {
        app = app.at(
            "*",
            StaticFilesEndpoint::new("./frontend/build").index_file("index.html"),
        );
    }

    Server::new(TcpListener::bind("0.0.0.0:8080"))
        .run(
            app.with(AddData::new(data.mailer))
                .with(AddData::new(data.database))
                .with(create_rust_app::Logger)
                .with(CookieJarManager::new())
                .catch_error(create_rust_app::default_404_handler),
        )
        .await
}
