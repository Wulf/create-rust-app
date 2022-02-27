use crate::diesel::*;
use crate::schema::*;
use create_rust_app::Database;

use chrono::{DateTime, Utc};
use actix_web::{Result, web::{Path, Json, Data, Query}, HttpResponse, delete, get, post, put, Error, Responder};
use serde::{Deserialize, Serialize};

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Identifiable, AsChangeset, Clone, Queryable)]
#[table_name = "todos"]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[tsync::tsync]
#[derive(Debug, Insertable, Serialize, Deserialize, AsChangeset)]
#[table_name = "todos"]
pub struct TodoJson {
    pub text: String,
}

const MAX_PAGE_SIZE: u16 = 100;

#[derive(Deserialize)]
pub struct IndexRequest {
    page: i64,
    page_size: i64,
}

#[get("")]
async fn index(
    db: Data<Database>,
    Query(info): Query<IndexRequest>,
) -> HttpResponse {
    let db = db.pool.get().unwrap();

    use crate::schema::todos::dsl::*;
    let result = todos
        .order(created_at)
        .limit(10)
        .offset(info.page * std::cmp::max(info.page_size, MAX_PAGE_SIZE as i64))
        .load::<Todo>(&db);

    if result.is_ok() {
        HttpResponse::Ok().json(result.unwrap())
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[get("/{id}")]
async fn read(
    db: Data<Database>,
    item_id: Path<i32>,
) -> HttpResponse {
    let db = db.pool.get().unwrap();

    use crate::schema::todos::dsl::*;
    let result = todos.filter(id.eq(item_id.into_inner())).first::<Todo>(&db);

    if result.is_ok() {
        HttpResponse::Ok().json(result.unwrap())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("")]
async fn create(
    db: Data<Database>,
    Json(item): Json<TodoJson>,
) -> Result<HttpResponse, Error> {
    let db = db.pool.get().unwrap();

    use crate::schema::todos::dsl::*;

    let result: Todo = insert_into(todos)
        .values(&item)
        .get_result::<Todo>(&db)
        .expect("Creation error");

    Ok(HttpResponse::Created().json(result))
}

#[put("/{id}")]
async fn update(
    db: Data<Database>,
    item_id: Path<i32>,
    Json(item): Json<TodoJson>,
) -> HttpResponse {
    let db = db.pool.get().unwrap();

    use crate::schema::todos::dsl::*;

    let result = diesel::update(todos.filter(id.eq(item_id.into_inner())))
        .set(&TodoJson {
            ..item
        })
        .execute(&db);

    if result.is_ok() {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[delete("/{id}")]
async fn destroy(
    db: Data<Database>,
    item_id: Path<i32>,
) -> HttpResponse {
    let db = db.pool.get().unwrap();

    use crate::schema::todos::dsl::*;

    let result = diesel::delete(todos.filter(id.eq(item_id.into_inner()))).execute(&db);

    if result.is_ok() {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
    return scope
        .service(index)
        .service(read)
        .service(create)
        .service(update)
        .service(destroy);
}
