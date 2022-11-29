use crate::{
    auth::Auth,
    dev::controller,
    dev::controller::{HealthCheckResponse, MySqlQuery},
    Database,
};
use actix_web::{
    get, post,
    web::{Data, Json},
    HttpResponse, Scope,
};
use std::ops::Deref;

#[post("/db/query")]
async fn query_db(db: Data<Database>, body: Json<MySqlQuery>) -> HttpResponse {
    match controller::query_db(&db, body.deref()) {
        Ok(result) => HttpResponse::Ok().body(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/auth/has-system-role")]
async fn check_system_role(auth: Auth) -> HttpResponse {
    HttpResponse::Ok().json(controller::check_system_role(&auth))
}

#[get("/auth/add-system-role")]
async fn add_system_role(db: Data<Database>, auth: Auth) -> HttpResponse {
    HttpResponse::Ok().json(controller::add_system_role(&db, &auth))
}

#[get("/db/is-connected")]
async fn is_connected(db: Data<Database>) -> HttpResponse {
    HttpResponse::Ok().json(controller::is_connected(&db))
}

#[get("/db/needs-migration")]
async fn needs_migration(db: Data<Database>) -> HttpResponse {
    HttpResponse::Ok().json(controller::needs_migration(&db))
}

#[get("/db/migrate")]
async fn migrate_db(db: Data<Database>) -> HttpResponse {
    HttpResponse::Ok().json(controller::migrate_db(&db))
}

#[get("/health")]
async fn health() -> HttpResponse {
    controller::health();
    HttpResponse::Ok().json(HealthCheckResponse {
        message: "healthy".to_string(),
    })
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
