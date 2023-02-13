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
    match controller::query_db(db.0, &body) {
        Ok(result) => Ok(result),
        Err(_) => Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

pub fn api() -> Route {
    Route::new().at("/db/query", post(query))
}
