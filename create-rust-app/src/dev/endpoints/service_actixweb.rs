use crate::{dev::controller, dev::controller::MySqlQuery, Database};
use actix_web::{
    post,
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

pub fn endpoints(scope: Scope) -> Scope {
    scope.service(query_db)
}
