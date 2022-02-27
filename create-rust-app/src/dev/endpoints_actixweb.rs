use actix_web::{get, post, web::{self, Data, Json}, Scope, HttpResponse, Responder, Result};

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

pub fn endpoints(scope: Scope) -> Scope {
    scope
        .service(query_db)
        .service(check_system_role)
        .service(add_system_role)
        .service(is_connected)
        .service(needs_migration)
        .service(migrate_db)
        .service(health)
}

#[post("/db/query")]
async fn query_db(db: Data<Database>, body: Json<MySqlQuery>) -> HttpResponse {
    let q = format!("SELECT json_agg(q) as json FROM ({}) q;", body.query);
    let db = db.pool.get().unwrap();

    let rows = sql_query(q.as_str()).get_result::<MyQueryResult>(&db);
    if rows.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    let result = rows.unwrap().json;

    HttpResponse::Ok().body(result)
}

#[get("/auth/has-system-role")]
async fn check_system_role(auth: Auth) -> HttpResponse {
    HttpResponse::Ok().json(auth.has_permission("system".to_string()))
}

#[get("/auth/add-system-role")]
async fn add_system_role(db: Data<Database>, auth: Auth) -> HttpResponse {
    let db = db.pool.clone().get().unwrap();

    HttpResponse::Ok()
        .json(Permission::assign_role(&db, auth.user_id, "system").unwrap())
}

#[get("/db/is-connected")]
async fn is_connected(db: Data<Database>) -> HttpResponse {
    let db = db.pool.clone().get().unwrap();

    let is_connected = diesel::sql_query("SELECT 1;").execute(&db);

    println!("{:#?}", is_connected);

    if is_connected.is_err() {
        return HttpResponse::Ok().json(false);
    }

    HttpResponse::Ok().json(true)
}

#[get("/db/needs-migration")]
async fn needs_migration(db: Data<Database>) -> HttpResponse {
    let db = db.pool.clone().get().unwrap();

    // This will run the necessary migrations.
    let has_migrations_pending = any_pending_migrations(&db).unwrap();

    if has_migrations_pending {
        return HttpResponse::Ok().json(true);
    }

    HttpResponse::Ok().json(false)
}

#[get("/db/migrate")]
async fn migrate_db(db: Data<Database>) -> HttpResponse {
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
}

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthCheckResponse {
        message: "healthy".to_string(),
    })
}