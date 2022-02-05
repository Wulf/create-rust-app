use actix_web::{
    web::{self, Data, Json},
    HttpResponse,
};

use diesel::{sql_types::Text, sql_query, QueryableByName, RunQueryDsl};
use diesel_migrations::{any_pending_migrations, run_pending_migrations};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{Auth, Permission},
    Database,
};

#[derive(Debug, Deserialize, QueryableByName)]
struct MyQueryResult {
    #[sql_type = "Text"]
    json: String,
}

#[derive(Serialize, Deserialize)]
struct HealthCheckResponse {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct MySqlQuery {
    query: String,
}

pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
    return scope
        .route(
            "/db/query",
            web::post().to(|db: Data<Database>, body: Json<MySqlQuery>| {
                let q = format!("SELECT json_agg(q) as json FROM ({}) q;", body.query);
                let db = db.pool.get().unwrap();

                let rows = sql_query(q.as_str()).get_result::<MyQueryResult>(&db);
                if rows.is_err() {
                    return HttpResponse::InternalServerError().finish();
                }

                let result = rows.unwrap().json;

                HttpResponse::Ok().body(result)
            }),
        )
        .route(
            "/auth/has-system-role",
            web::get().to(|auth: Auth| HttpResponse::Ok().json(auth.has_permission("system"))),
        )
        .route(
            "/auth/add-system-role",
            web::get().to(|db: Data<Database>, auth: Auth| {
                let db = db.pool.clone().get().unwrap();

                HttpResponse::Ok()
                    .json(Permission::assign_role(&db, auth.user_id, "system").unwrap())
            }),
        )
        .route(
            "/db/is-connected",
            web::get().to(|db: Data<Database>| {
                let db = db.pool.clone().get().unwrap();

                let is_connected = diesel::sql_query("SELECT 1;").execute(&db);

                println!("{:#?}", is_connected);

                if is_connected.is_err() {
                    return HttpResponse::Ok().json(false);
                }

                HttpResponse::Ok().json(true)
            }),
        )
        .route(
            "/db/needs-migration",
            web::get().to(|db: Data<Database>| {
                let db = db.pool.clone().get().unwrap();

                // This will run the necessary migrations.
                let has_migrations_pending = any_pending_migrations(&db).unwrap();

                if has_migrations_pending {
                    return HttpResponse::Ok().json(true);
                }

                HttpResponse::Ok().json(false)
            }),
        )
        .route(
            "/db/migrate",
            web::get().to(|db: Data<Database>| {
                let db = db.pool.clone().get().unwrap();

                // This will run the necessary migrations.
                let has_migrations_pending = any_pending_migrations(&db).unwrap();

                if !has_migrations_pending {
                    return HttpResponse::Ok().json(false);
                }

                let op = run_pending_migrations(&db);

                if op.is_err() {
                    println!("{:#?}", op.err());
                    return HttpResponse::InternalServerError().json(false);
                }

                HttpResponse::Ok().json(true)
            }),
        )
        .route(
            "/health",
            web::get().to(|| {
                HttpResponse::Ok().json(HealthCheckResponse {
                    message: "healthy".to_string(),
                })
            }),
        );
}
