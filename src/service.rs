use crate::logger::message;
use anyhow::Result;
use indoc::indoc;
use inflector::Inflector;
use std::path::PathBuf;

struct Service {
  pub config: ServiceConfig,
  pub file_contents: String,
}

struct ServiceConfig {
  pub model_name: String,
  pub file_name: String,
}

pub fn create(resource_name: &str, base_endpoint_path: &str) -> Result<()> {
  let resource = generate(resource_name);
  crate::fs::add_rust_file(
    "src/services",
    resource.config.file_name.as_str(),
    resource.file_contents.as_str(),
  )?;

  crate::service::register_service(resource.config.file_name.as_str(), base_endpoint_path)?;

  Ok(())
}

fn config(service_name: &str) -> ServiceConfig {
  let model_name = service_name.to_pascal_case();
  let file_name = model_name.to_snake_case();

  return ServiceConfig {
    model_name: model_name,
    file_name: file_name,
  };
}

fn generate(service_name: &str) -> Service {
  let config = config(service_name);
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

  let contents = String::from(contents_template)
    .replace("$MODEL_NAME", config.model_name.as_str())
    .replace("$FILE_NAME", config.file_name.as_str());

  Service {
    config: config,
    file_contents: contents,
  }
}

pub fn register_service(service_file_name: &str, service_base_endpoint_path: &str) -> Result<()> {
  message(&format!("Registering service {}", service_file_name));
  let main_file_path = PathBuf::from("src/main.rs");
  if main_file_path.exists() && main_file_path.is_file() {
    let mut main_file_contents = std::fs::read_to_string(&main_file_path)?;

    main_file_contents = main_file_contents.replace("web::scope(\"/api\")", &format!("web::scope(\"/api\")\n                    .service(services::{}::endpoints(web::scope(\"{}\")))", service_file_name, service_base_endpoint_path));
    std::fs::write(main_file_path, main_file_contents)?;
  }

  Ok(())
}
