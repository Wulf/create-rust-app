use poem::{
    handler,
    http::StatusCode,
    post,
    web::{Data, Json},
    Error, IntoResponse, Result, Route,
};

use crate::dev::{controller, controller::MySqlQuery};

use crate::Database;

#[handler]
async fn query(db: Data<&Database>, body: Json<MySqlQuery>) -> Result<impl IntoResponse> {
    controller::query_db(db.0, &body)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
}

pub fn api() -> Route {
    Route::new().at("/db/query", post(query))
}
