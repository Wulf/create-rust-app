use crate::logger::register_service_msg;
use crate::BackendFramework;
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

pub fn create(
    backend: BackendFramework,
    resource_name: &str,
    service_api_fn: &str,
    base_endpoint_path: &str,
    include_qsync_attr: bool,
) -> Result<()> {
    let resource = match backend {
        BackendFramework::ActixWeb => generate_actix(resource_name, include_qsync_attr),
        BackendFramework::Poem => generate_poem(resource_name),
    };

    crate::fs::add_rust_file(
        "backend/services",
        resource.config.file_name.as_str(),
        resource.file_contents.as_str(),
    )?;

    match backend {
        BackendFramework::ActixWeb => {
            let name = resource.config.file_name.as_str();
            let service_entry =
                &format!("services::{name}::endpoints(web::scope(\"{base_endpoint_path}\"))");
            register_actix(name, service_entry)?;
        }
        BackendFramework::Poem => register_poem(
            &resource.config.file_name,
            service_api_fn,
            base_endpoint_path,
        )?,
    };

    Ok(())
}

fn config(service_name: &str) -> ServiceConfig {
    let model_name = service_name.to_pascal_case();
    let file_name = model_name.to_snake_case();

    ServiceConfig {
        model_name,
        file_name,
    }
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
        config,
        file_contents: contents,
    }
}

fn generate_actix(service_name: &str, include_qsync_attr: bool) -> Service {
    let config = config(service_name);
    let contents_template: &str = indoc! {r#"
    use actix_web::{delete, get, post, put};
    use actix_web::{
        HttpResponse,
        web::{Data, Json, Path, Query},
    };
    use create_rust_app::Database;
    use diesel::OptionalExtension;
    use qsync::qsync;
    use serde::Deserialize;
    use tsync::tsync;

    use crate::models::$TABLE_NAME::{$MODEL_NAME, Create$MODEL_NAME, Update$MODEL_NAME};
    
    #[tsync]
    #[derive(Deserialize)]
    struct List$MODEL_NAMERequest {
        page: i64,
        page_size: i64,
    }
    
    $LIST_QSYNC_ATTR#[get("")]
    async fn list(
      db: Data<Database>,
      info: Query<List$MODEL_NAMERequest>
    ) -> HttpResponse {
      let mut db = db.pool.get().unwrap();
    
      let results = $MODEL_NAME::paginate(&mut db, info.page, info.page_size);

      match results {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(_) => HttpResponse::InternalServerError().finish(),
      }
    }
    
    $READ_QSYNC_ATTR#[get("/{id}")]
    async fn read(
      db: Data<Database>,
      item_id: Path<i32>
    ) -> HttpResponse {
        let mut db = db.pool.get().unwrap();

        let result = $MODEL_NAME::read(&mut db, item_id.into_inner()).optional();

        match result {
            Ok(result) => match result {
                Some(item) => HttpResponse::Ok().json(item),
                None => HttpResponse::NotFound().finish(),
            },
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }
    
    $CREATE_QSYNC_ATTR#[post("")]
    async fn create(
      db: Data<Database>,
      item: Json<Create$MODEL_NAME>
    ) -> HttpResponse {
        let mut db = db.pool.get().unwrap();

        let result = $MODEL_NAME::create(&mut db, &item);
    
        match result {
            Ok(result) => HttpResponse::Ok().json(result),
            Err(_) => HttpResponse::InternalServerError().finish()
        }
    }
    
    $UPDATE_QSYNC_ATTR#[put("/{id}")]
    async fn update(
      db: Data<Database>,
      item_id: Path<i32>,
      item: Json<Update$MODEL_NAME>
    ) -> HttpResponse {
        let mut db = db.pool.get().unwrap();

        let result = $MODEL_NAME::update(&mut db, item_id.into_inner(), &item);
    
        match result {
            Ok(result) => HttpResponse::Ok().json(result),
            Err(_) => HttpResponse::InternalServerError().finish()
        }
    }
    
    $DESTROY_QSYNC_ATTR#[delete("/{id}")]
    async fn destroy(db: Data<Database>, item_id: Path<i32>) -> HttpResponse {
        let mut db = db.pool.get().unwrap();
    
        let result = $MODEL_NAME::delete(&mut db, item_id.into_inner());
    
        match result {
            Ok(result) => match result {
                0 => HttpResponse::NotFound().finish(),
                usize => HttpResponse::Ok().json(usize)
            },
            Err(_) => HttpResponse::InternalServerError().finish()
        }
    }
    
    pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
      return scope
        .service(list)
        .service(read)
        .service(create)
        .service(update)
        .service(destroy);
    }
  "#};

    let destroy_qsync_attr = "#[qsync(return_type=\"number\")]\n";
    let update_qsync_attr = "#[qsync(return_type=\"$MODEL_NAME\")]\n";
    let create_qsync_attr = "#[qsync(return_type=\"$MODEL_NAME\")]\n";
    let read_qsync_attr = "#[qsync(return_type=\"$MODEL_NAME\")]\n";
    let list_qsync_attr = "#[qsync(return_type=\"PaginationResult<$MODEL_NAME>\")]\n";

    let contents = String::from(contents_template)
        .replace(
            "$DESTROY_QSYNC_ATTR",
            if include_qsync_attr {
                destroy_qsync_attr
            } else {
                ""
            },
        )
        .replace(
            "$UPDATE_QSYNC_ATTR",
            if include_qsync_attr {
                update_qsync_attr
            } else {
                ""
            },
        )
        .replace(
            "$CREATE_QSYNC_ATTR",
            if include_qsync_attr {
                create_qsync_attr
            } else {
                ""
            },
        )
        .replace(
            "$READ_QSYNC_ATTR",
            if include_qsync_attr {
                read_qsync_attr
            } else {
                ""
            },
        )
        .replace(
            "$LIST_QSYNC_ATTR",
            if include_qsync_attr {
                list_qsync_attr
            } else {
                ""
            },
        )
        .replace("$MODEL_NAME", config.model_name.as_str())
        .replace("$TABLE_NAME", config.file_name.to_plural().as_str());

    Service {
        config,
        file_contents: format!("{}\n", contents.trim()),
    }
}

/// use fs::replace instead and also fs::append for the services/mod.rs entry
// #[deprecated]
pub fn register_poem(
    name: &str,
    service_api_fn: &str,
    service_base_endpoint_path: &str,
) -> Result<()> {
    register_service_msg(name);
    let main_file_path = PathBuf::from("backend/main.rs");
    if main_file_path.exists() && main_file_path.is_file() {
        let mut main_file_contents = std::fs::read_to_string(&main_file_path)?;

        main_file_contents = main_file_contents.replace(
            "let mut api_routes = Route::new();",
            &format!(
                "let mut api_routes = Route::new();\n\t\tapi_routes = api_routes.nest(\"{service_base_endpoint_path}\", {service_api_fn});",
            ),
        );
        std::fs::write(main_file_path, main_file_contents)?;
    }

    Ok(())
}

/// use fs::replace instead and also fs::append for the services/mod.rs entry
// #[deprecated]
pub fn register_actix(name: &str, service: &str) -> Result<()> {
    register_service_msg(name);
    let main_file_path = PathBuf::from("backend/main.rs");
    if main_file_path.exists() && main_file_path.is_file() {
        let mut main_file_contents = std::fs::read_to_string(&main_file_path)?;
        main_file_contents = main_file_contents.replace(
            r#"let mut api_scope = web::scope("/api");"#,
            &format!(
                r#"let mut api_scope = web::scope("/api");
        api_scope = api_scope.service({service});"#
            ),
        );
        std::fs::write(main_file_path, main_file_contents)?;
    }

    Ok(())
}
