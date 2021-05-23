use indoc::indoc;

pub fn generate_service(file_name: &str, model_name: &str, table_name: &str) -> String {
  let contents_template: &str = indoc! {"
    use crate::models::$FILE_NAME::{$MODEL_NAME, $MODEL_NAMEChangeset};
    use crate::models::{ID, PaginationParams};
    use crate::Pool;
    
    use actix_web::{delete, get, post, put, Error as AWError};
    use actix_web::{web, HttpResponse};
    
    #[get(\"\")]
    async fn index(
      pool: web::Data<Pool>,
      web::Query(info): web::Query<PaginationParams>
    ) -> Result<HttpResponse, AWError> {
      let db = pool.get().unwrap();
    
      Ok($MODEL_NAME::read_all(&db, &info)
        .map(|items| HttpResponse::Ok().json(items))
        .map_err(|_| HttpResponse::InternalServerError())?)
    }
    
    #[get(\"/{id}\")]
    async fn read(
      pool: web::Data<Pool>,
      web::Path(item_id): web::Path<ID>
    ) -> Result<HttpResponse, AWError> {
      let db = pool.get().unwrap();
    
      Ok($MODEL_NAME::read(&db, item_id)
        .map(|item| HttpResponse::Found().json(item))
        .map_err(|_| HttpResponse::NotFound())?)
    }
    
    #[post(\"\")]
    async fn create(
      pool: web::Data<Pool>,
      web::Json(item): web::Json<$MODEL_NAMEChangeset>
    ) -> Result<HttpResponse, AWError> {
      let db = pool.get().unwrap();
    
      Ok($MODEL_NAME::create(&db, &item)
        .map(|item| HttpResponse::Created().json(item))
        .map_err(|_| HttpResponse::InternalServerError())?)
    }
    
    #[put(\"/{id}\")]
    async fn update(
      pool: web::Data<Pool>,
      web::Path(item_id): web::Path<ID>,
      web::Json(item): web::Json<$MODEL_NAMEChangeset>
    ) -> Result<HttpResponse, AWError> {
      let db = pool.get().unwrap();
    
      Ok($MODEL_NAME::update(&db, item_id, &item)
        .map(|item| HttpResponse::Ok().json(item))
        .map_err(|_| HttpResponse::InternalServerError())?)
    }
    
    #[delete(\"/{id}\")]
    async fn destroy(
        pool: web::Data<Pool>,
        web::Path(item_id): web::Path<ID>,
    ) -> Result<HttpResponse, AWError> {
        let db = pool.get().unwrap();
    
        Ok($MODEL_NAME::delete(&db, item_id)
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
  "};

    let contents = String::from(contents_template).replace("$MODEL_NAME", model_name).replace("$TABLE_NAME", table_name).replace("$FILE_NAME", file_name);

    contents
}