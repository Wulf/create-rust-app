use poem::{get, handler, http::StatusCode, post, web::{Json, Data}, Error, IntoResponse, Result, Route};

use diesel::{sql_types::Text, sql_query, QueryableByName, RunQueryDsl};
use diesel_migrations::{any_pending_migrations, run_pending_migrations};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{Auth, Permission},
    Database,
};

#[derive(Serialize, Deserialize)]
struct HealthCheckResponse {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct MySqlQuery {
    query: String,
}


#[derive(Debug, Deserialize, QueryableByName)]
struct MyQueryResult {
    #[sql_type = "Text"]
    json: String,
}


#[handler]
async fn health() -> Result<impl IntoResponse> {
    Ok(Json(HealthCheckResponse {
        message: "healthy".to_string(),
    })
        .with_status(StatusCode::OK)
        .into_response())
}

#[handler]
async fn migrate(db: Data<&Database>) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    // This will run the necessary migrations.
    let has_migrations_pending = any_pending_migrations(&db).unwrap();

    if !has_migrations_pending {
        return Ok(Json(false).into_response());
    }

    let op = run_pending_migrations(&db);

    if op.is_err() {
        println!("{:#?}", op.err());
        return Ok(Json(false)
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response());
    }

    Ok(Json(true).with_status(StatusCode::OK).into_response())
}

#[handler]
async fn needs_migration(db: Data<&Database>) -> Result<impl IntoResponse> {
    let db = db.pool.clone().get().unwrap();

    // This will run the necessary migrations.
    let has_migrations_pending = any_pending_migrations(&db).unwrap();

    if has_migrations_pending {
        return Ok(Json(true));
    }

    Ok(Json(false))
}

#[handler]
async fn query(db: Data<&Database>, body: Json<MySqlQuery>) -> Result<impl IntoResponse> {
    let q = format!("SELECT json_agg(q) as json FROM ({}) q;", body.query);
    let db = db.pool.get().unwrap();

    let rows = sql_query(q.as_str()).get_result::<MyQueryResult>(&db);
    if rows.is_err() {
        return Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR));
    }

    let result = rows.unwrap().json;

    Ok(result)
}

#[handler]
async fn is_connected(db: Data<&Database>) -> Result<impl IntoResponse> {
    let db = db.pool.clone().get().unwrap();

    let is_connected = diesel::sql_query("SELECT 1;").execute(&db);

    println!("{:#?}", is_connected);

    if is_connected.is_err() {
        return Ok(Json(false));
    }

    Ok(Json(true))
}

#[handler]
async fn has_system_role(auth: Auth) -> Result<impl IntoResponse> {
    Ok(Json(auth.has_permission("system")))
}

#[handler]
async fn add_system_role(auth: Auth, db: Data<&Database>) -> Result<impl IntoResponse> {
    let db = db.pool.clone().get().unwrap();

    Ok(Json(
        Permission::assign_role(&db, auth.user_id, "system").unwrap(),
    ))
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
