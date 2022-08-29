use argonautica::config::Version::_0x10;
use poem::{
    get, handler,
    http::StatusCode,
    post,
    web::{Data, Json},
    Error, IntoResponse, Result, Route,
};

use diesel::{sql_query, sql_types::Text, QueryableByName, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::dev::{
    controller,
    controller::{HealthCheckResponse, MyQueryResult, MySqlQuery},
};

use crate::auth::Role;
use crate::{
    auth::{Auth, Permission},
    Database,
};

#[handler]
async fn health() -> Result<impl IntoResponse> {
    controller::health();
    Ok(Json(HealthCheckResponse {
        message: "healthy".to_string(),
    })
    .with_status(StatusCode::OK)
    .into_response())
}

#[handler]
async fn migrate(db: Data<&Database>) -> Result<impl IntoResponse> {
    if controller::migrate_db(db.0) {
        Ok(Json(true).with_status(StatusCode::OK).into_response())
    } else {
        Ok(Json(false)
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response())
    }
}

#[handler]
async fn needs_migration(db: Data<&Database>) -> Result<impl IntoResponse> {
    Ok(Json(controller::needs_migration(db.0)))
}

#[handler]
async fn query(db: Data<&Database>, body: Json<MySqlQuery>) -> Result<impl IntoResponse> {
    match controller::query_db(db.0, &body) {
        Ok(result) => Ok(result),
        Err(_) => Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

#[handler]
async fn is_connected(db: Data<&Database>) -> Result<impl IntoResponse> {
    Ok(Json(controller::is_connected(db.0)))
}

#[handler]
async fn has_system_role(auth: Auth) -> Result<impl IntoResponse> {
    Ok(Json(controller::check_system_role(&auth)))
}

#[handler]
async fn add_system_role(auth: Auth, db: Data<&Database>) -> Result<impl IntoResponse> {
    Ok(Json(controller::add_system_role(db.0, &auth)))
}

pub fn api() -> Route {
    Route::new()
        .at("/health", get(health))
        .at("/db/migrate", get(migrate))
        .at("/db/needs-migration", get(needs_migration))
        .at("/db/query", post(query))
        .at("/db/is-connected", get(is_connected))
        .at("/auth/add-system-role", get(add_system_role))
        .at("/auth/has-system-role", get(has_system_role))
}
