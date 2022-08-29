use actix_web::{delete, Error, get, HttpResponse, post, put, Result, web::{Data, Json, Path, Query}};
use create_rust_app::Database;
use crate::{models, models::todo::{Todo, TodoChangeset, PaginationParams}};

#[get("")]
async fn index(
    db: Data<Database>,
    Query(info): Query<PaginationParams>,
) -> HttpResponse {
    let mut db = db.pool.get().unwrap();

    let result = models::todo::read_all(&mut db, &info);

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
    let mut db = db.pool.get().unwrap();

    let result = models::todo::read(&mut db, item_id.into_inner());

    if result.is_ok() {
        HttpResponse::Ok().json(result.unwrap())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("")]
async fn create(
    db: Data<Database>,
    Json(item): Json<TodoChangeset>,
) -> Result<HttpResponse, Error> {
    let mut db = db.pool.get().unwrap();

    let result: Todo = models::todo::create(&mut db, &item).expect("Creation error");

    Ok(HttpResponse::Created().json(result))
}

#[put("/{id}")]
async fn update(
    db: Data<Database>,
    item_id: Path<i32>,
    Json(item): Json<TodoChangeset>,
) -> HttpResponse {
    let mut db = db.pool.get().unwrap();

    let result = models::todo::update(&mut db, item_id.into_inner(), &item);

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
    let mut db = db.pool.get().unwrap();

    let result = models::todo::delete(&mut db, item_id.into_inner());

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
