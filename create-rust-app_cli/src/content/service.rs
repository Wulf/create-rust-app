use crate::utils::logger::message;
use anyhow::Result;
use indoc::indoc;
use inflector::Inflector;
use std::path::PathBuf;
use crate::BackendFramework;

struct Service {
    pub config: ServiceConfig,
    pub file_contents: String,
}

struct ServiceConfig {
    pub model_name: String,
    pub file_name: String,
}

pub fn create(backend: BackendFramework, resource_name: &str, service_api_fn: &str, base_endpoint_path: &str) -> Result<()> {
    let resource = match backend {
        BackendFramework::ActixWeb => generate_actix(resource_name),
        BackendFramework::Poem => generate_poem(resource_name)
    };

    crate::fs::add_rust_file(
        "backend/services",
        resource.config.file_name.as_str(),
        resource.file_contents.as_str(),
    )?;

    match backend {
        BackendFramework::ActixWeb => {
            let name = resource.config.file_name.as_str();
            let service_entry = &format!("services::{}::endpoints(web::scope(\"{}\"))", name, base_endpoint_path);
            register_actix(name, service_entry)?;
        },
        BackendFramework::Poem => register_poem(&resource.config.file_name, service_api_fn, base_endpoint_path)?
    };

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

fn generate_poem(service_name: &str) -> Service {
    let config = config(service_name);
    let contents_template: &str = indoc! {"\
    use create_rust_app::Database;
    use diesel::NotFound;
    use poem::{get, Route, handler, Result, IntoResponse, Response};
    use poem::error::InternalServerError;
    use poem::http::StatusCode;
    use poem::web::{Data, Json, Path, Query};
    use crate::models::$FILE_NAME::{$MODEL_NAME, $MODEL_NAMEChangeset};
    use crate::models::{PaginationParams, ID};


    #[handler]
    async fn index(
        db: Data<&Database>,
        Query(info): Query<PaginationParams>,
    ) -> Result<impl IntoResponse> {
        let db = db.pool.get().unwrap();

        Ok($MODEL_NAME::read_all(&db, &info)
            .map(|items| Json(items).with_status(StatusCode::OK)
            .map_err(|_| InternalServerError)?)
    }

    #[handler]
    async fn read(
        db: Data<&Database>,
        Path(item_id): Path<ID>,
    ) -> Result<impl IntoResponse> {
        let db = db.pool.get().unwrap();

        Ok($MODEL_NAME::read(&db, item_id)
            .map(|item| Json(item).with_status(StatusCode::FOUND))
            .map_err(|_| NotFound)?)
    }

    #[handler]
    async fn create(
        db: Data<&Database>,
        Json(item): Json<$MODEL_NAMEChangeset>,
    ) -> Result<impl IntoResponse> {
        let db = db.pool.get().unwrap();

        Ok($MODEL_NAME::create(&db, &item)
            .map(|item| Json(item).with_status(StatusCode::CREATED))
            .map_err(|_| InternalServerError)?)
    }

    #[handler]
    async fn update(
        db: Data<&Database>,
        Path(item_id): Path<ID>,
        Json(item): Json<$MODEL_NAMEChangeset>,
    ) -> Result<impl IntoResponse> {
        let db = db.pool.get().unwrap();

        Ok($MODEL_NAME::update(&db, item_id, &item)
            .map(|item| Json(item))
            .map_err(|_| InternalServerError)?)
    }

    #[handler]
    async fn destroy(
        db: Data<&Database>,
        Path(item_id): Path<ID>,
    ) -> Result<impl IntoResponse> {
        let db = db.pool.get().unwrap();

        Ok($MODEL_NAME::delete(&db, item_id)
            .map(|_| Response::builder().status(StatusCode::NO_CONTENT))
            .map_err(|_| InternalServerError)?)
    }

    pub fn api() -> Route {
        Route::new()
            .at(\"/\", get(index).post(create))
            .at(\"/:id\", get(read).put(update).delete(destroy))
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

fn generate_actix(service_name: &str) -> Service {
    let config = config(service_name);
    let contents_template: &str = indoc! {"\
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

/// use fs::replace instead and also fs::append for the services/mod.rs entry
#[deprecated]
pub fn register_poem(name: &str, service_api_fn: &str, service_base_endpoint_path: &str) -> Result<()> {
    message(&format!("Registering service {}", name));
    let main_file_path = PathBuf::from("backend/main.rs");
    if main_file_path.exists() && main_file_path.is_file() {
        let mut main_file_contents = std::fs::read_to_string(&main_file_path)?;

        main_file_contents = main_file_contents.replace("let mut api = Route::new()", &format!("let mut api = Route::new()\n\t\t.nest(\"{}\", {})", service_base_endpoint_path, service_api_fn));
        std::fs::write(main_file_path, main_file_contents)?;
    }

    Ok(())
}

/// use fs::replace instead and also fs::append for the services/mod.rs entry
#[deprecated]
pub fn register_actix(name: &str, service: &str) -> Result<()> {
    message(&format!("Registering service {}", name));
    let main_file_path = PathBuf::from("backend/main.rs");
    if main_file_path.exists() && main_file_path.is_file() {
        let mut main_file_contents = std::fs::read_to_string(&main_file_path)?;
        main_file_contents = main_file_contents.replace("web::scope(\"/api\")", &format!("web::scope(\"/api\")\n            .service({})", service));
        std::fs::write(main_file_path, main_file_contents)?;
    }

    Ok(())
}