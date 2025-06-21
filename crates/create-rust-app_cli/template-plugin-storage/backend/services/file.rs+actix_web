use actix_multipart::Multipart;
use actix_web::{HttpResponse, ResponseError};
use actix_web::web::{Data, Path};
use serde::Serialize;
use create_rust_app::{Attachment, AttachmentBlob, AttachmentData, Database, Storage};
use futures_util::StreamExt as _;

#[derive(Serialize)]
#[tsync::tsync]
struct FileInfo {
    pub id: i32,
    pub key: String,
    pub name: String,
    pub url: Option<String>,
}

#[actix_web::get("")]
async fn all(db: Data<Database>, storage: Data<Storage>) -> HttpResponse {
    let mut db = db.get_connection().unwrap();
    let files = Attachment::find_all_for_record(&mut db, "file".to_string(), "NULL".to_string(), 0).unwrap_or_default();
    let blob_ids = files.iter().map(|f| f.blob_id).collect::<Vec<_>>();
    let blobs = AttachmentBlob::find_all_by_id(&mut db, blob_ids).unwrap_or_default();

    let mut files = blobs.iter().enumerate().map(|b| FileInfo {
        id: files[b.0].id,
        key: b.1.clone().key,
        name: b.1.clone().file_name,
        url: None,
    }).collect::<Vec<FileInfo>>();

    for info in files.iter_mut() {
        let uri = storage.download_uri(info.key.clone(), None).await;
        if uri.is_err() {
            return HttpResponse::InternalServerError().json(uri.err().unwrap());
        }
        let uri = uri.unwrap();
        info.url = Some(uri);
    }

    HttpResponse::Ok().json(files)
}

#[actix_web::delete("/{id}")]
async fn delete(db: Data<Database>, storage: Data<Storage>, file_id: Path<i32>) -> HttpResponse {
    let mut db = db.get_connection().unwrap();
    let file_id = file_id.into_inner();

    let detach_op = Attachment::detach(&mut db, &storage, file_id).await;

    if detach_op.is_err() {
        return HttpResponse::InternalServerError().json(detach_op.err().unwrap());
    }

    HttpResponse::Ok().finish()
}

#[actix_web::post("")]
async fn create(db: Data<Database>, store: Data<Storage>, mut payload: Multipart) -> HttpResponse {
    let mut db = db.get_connection().unwrap();

    while let Some(item) = payload.next().await {
        let mut field = if item.is_ok() {
            item.unwrap()
        } else {
            let err = item.err().unwrap();
            return err.error_response();
        };

        let content_disposition = field.content_disposition();
        let file_name = content_disposition.get_filename().map(|f| f.to_string());
        let field_name = content_disposition
            .get_name().unwrap();

        match field_name {
            "file" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk.unwrap()[..]);
                }

                let attached_req = Attachment::attach(&mut db, &store, "file".to_string(), "NULL".to_string(), 0, AttachmentData {
                    data,
                    file_name
                }, true, false).await;

                if attached_req.is_err() {
                    return HttpResponse::InternalServerError().json(attached_req.err().unwrap());
                }
            },
            _ => {}
        }
    }

    HttpResponse::Ok().finish()
}

pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
    return scope
        .service(create)
        .service(all)
        .service(delete);
}
