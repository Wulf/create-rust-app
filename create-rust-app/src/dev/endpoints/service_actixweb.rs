use crate::{dev::controller, dev::controller::MySqlQuery, Database};
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Scope,
};

#[post("/db/query")]
async fn query_db(db: Data<Database>, body: Json<MySqlQuery>) -> HttpResponse {
    controller::query_db(&db, &body).map_or_else(
        |_| HttpResponse::InternalServerError().finish(),
        |result| HttpResponse::Ok().body(result),
    )
}

#[must_use]
pub fn endpoints(scope: Scope) -> Scope {
    scope.service(query_db)
}
