use crate::diesel::*;
use crate::schema::*;
use crate::Pool;

use chrono::{DateTime, Utc};
use actix_web::{delete, get, post, put, Error};
use actix_web::{web, HttpResponse};
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
    pool: web::Data<Pool>,
    web::Query(info): web::Query<IndexRequest>,
) -> Result<HttpResponse, Error> {
    let db = pool.get().unwrap();

    use crate::schema::todos::dsl::*;
    let result = todos
        .order(created_at)
        .limit(10)
        .offset(info.page * std::cmp::max(info.page_size, MAX_PAGE_SIZE as i64))
        .load::<Todo>(&db);

    Ok(result
        .map(|items| HttpResponse::Ok().json(items))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

#[get("/{id}")]
async fn read(
    pool: web::Data<Pool>,
    web::Path(item_id): web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let db = pool.get().unwrap();

    use crate::schema::todos::dsl::*;
    let result = todos.filter(id.eq(item_id)).first::<Todo>(&db);

    Ok(result
        .map(|item| HttpResponse::Ok().json(item))
        .map_err(|_| HttpResponse::NotFound())?)
}

#[post("")]
async fn create(
    pool: web::Data<Pool>,
    web::Json(item): web::Json<TodoJson>,
) -> Result<HttpResponse, Error> {
    let db = pool.get().unwrap();

    use crate::schema::todos::dsl::*;

    let result: Todo = insert_into(todos)
        .values(&item)
        .get_result::<Todo>(&db)
        .expect("Creation error");

    Ok(HttpResponse::Created().json(result))
}

#[put("/{id}")]
async fn update(
    pool: web::Data<Pool>,
    web::Path(item_id): web::Path<i32>,
    web::Json(item): web::Json<TodoJson>,
) -> Result<HttpResponse, Error> {
    let db = pool.get().unwrap();

    use crate::schema::todos::dsl::*;

    let result = diesel::update(todos.filter(id.eq(item_id)))
        .set(&TodoJson {
            ..item
        })
        .execute(&db);

    Ok(result
        .map(|_| HttpResponse::Ok().finish())
        .map_err(|_| HttpResponse::InternalServerError().finish())?)
}

#[delete("/{id}")]
async fn destroy(
    pool: web::Data<Pool>,
    web::Path(item_id): web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let db = pool.get().unwrap();

    use crate::schema::todos::dsl::*;

    let result = diesel::delete(todos.filter(id.eq(item_id))).execute(&db);
    println!("{:?}", result);
    Ok(result
        .map(|_| HttpResponse::Ok().finish())
        .map_err(|_| HttpResponse::InternalServerError().finish())?)
}

pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
    return scope
        .service(index)
        .service(read)
        .service(create)
        .service(update)
        .service(destroy);
}
