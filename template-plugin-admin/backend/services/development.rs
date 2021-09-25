use actix_web::{HttpResponse, web::{self, Data, Json}};
use diesel::{RunQueryDsl};
use postgres::NoTls;
use serde::{Deserialize, Serialize};
use diesel_migrations::run_pending_migrations;

use crate::{Pool, extractors::auth::Auth, models::permissions::Permission};

use diesel_migrations::any_pending_migrations;

#[derive(Serialize, Deserialize)]
struct HealthCheckResponse {
    message: String
}

#[derive(Serialize, Deserialize)]
struct SqlQuery {
  query: String
}

pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
  return scope
    .route("/db/query", web::post().to(|body: Json<SqlQuery>| {
      let q = format!("SELECT json_agg(q) FROM ({}) q;", body.query);
      let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable expected.");
      let mut pg = postgres::Client::connect(&database_url, NoTls).unwrap();
      let result = pg.query(q.as_str(), &[]).unwrap();
      let result: serde_json::Value = result[0].get(0);
      HttpResponse::Ok().json(result)
    }))
    .route("/auth/has-system-role", web::get().to(|auth: Auth| {
      HttpResponse::Ok().json(auth.has_permission("system"))
    }))
    .route("/auth/add-system-role", web::get().to(|pool: Data<Pool>, auth: Auth| {
      let db = pool.clone().get().unwrap();

      HttpResponse::Ok().json(Permission::assign_role(&db, auth.user_id, "system").unwrap())
    }))
    .route("/db/is-connected", web::get().to(|pool: Data<Pool>| {
      let db = pool.clone().get().unwrap();

      let is_connected = diesel::sql_query("SELECT 1;").execute(&db);

      println!("{:#?}", is_connected);

      if is_connected.is_err() {
        return HttpResponse::Ok().json(false);
      }
      
      HttpResponse::Ok().json(true)
    }))
    .route("/db/needs-migration", web::get().to(|pool: Data<Pool>| {
      let db = pool.clone().get().unwrap();

      // This will run the necessary migrations.
      let has_migrations_pending = any_pending_migrations(&db).unwrap();

      if has_migrations_pending {
        return HttpResponse::Ok().json(true);
      }
      
      HttpResponse::Ok().json(false)
    }))
    .route("/db/migrate", web::get().to(|pool: Data<Pool>| {
      let db = pool.clone().get().unwrap();

      // This will run the necessary migrations.
      let has_migrations_pending = any_pending_migrations(&db).unwrap();

      if !has_migrations_pending {
        return HttpResponse::Ok().json(false);
      }
      
      let op = run_pending_migrations(&db);

      if op.is_err() {
        println!("{:#?}",op.err());
        return HttpResponse::InternalServerError().json(false)
      }
      
      HttpResponse::Ok().json(true)
    }))
    .route("/health", web::get().to(|| HttpResponse::Ok().json(HealthCheckResponse { message: "healthy".to_string() })))
}