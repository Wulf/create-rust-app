use diesel::result::Error;
use diesel::QueryResult;
//use md5;
//use mime_guess;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::diesel::*;
use crate::storage::attachment_blob::AttachmentBlobChangeset;
use crate::storage::{schema, AttachmentBlob, Utc, ID};
use crate::Connection;

use super::{schema::*, Storage};

#[derive(
    Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, AsChangeset,
)]
#[diesel(table_name=attachments)]
pub struct Attachment {
    pub id: ID,

    pub name: String,
    pub record_type: String,
    pub record_id: ID,
    pub blob_id: ID,

    pub created_at: Utc,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[diesel(table_name=attachments)]
pub struct AttachmentChangeset {
    pub name: String,
    pub record_type: String,
    pub record_id: ID,
    pub blob_id: ID,
}

pub struct AttachmentData {
    pub data: Vec<u8>,
    pub file_name: Option<String>,
}

impl Attachment {
    /// in actix_web we don't need to support send+sync handlers, so we can use the &mut Connection directly.
    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "backend_actix-web")]
    pub async fn attach(
        db: &mut Connection,
        storage: &Storage,
        name: String,
        record_type: String,
        record_id: ID,
        data: AttachmentData,
        allow_multiple: bool,
        overwrite_existing: bool,
    ) -> Result<String, String> {
        let checksum = format!("{:x}", md5::compute(&data.data));
        let file_name = data.file_name.clone();
        let content_type = if file_name.is_some() {
            mime_guess::from_path(file_name.unwrap()).first_raw()
        } else {
            None
        }
        .map(|t| t.to_string());
        let key = Uuid::new_v4().to_string();

        if !allow_multiple {
            let existing =
                Attachment::find_for_record(db, name.clone(), record_type.clone(), record_id);

            if existing.is_ok() {
                // one already exists, we need to delete it
                if overwrite_existing {
                    Attachment::detach(db, storage, existing.unwrap().id).await.map_err(|_| {
                        format!("Could not detach the existing attachment for '{name}' attachment on '{record_type}'", name=name.clone(), record_type=record_type.clone())
                    })?;
                } else {
                    // throw the error
                    return Err(format!("Only 1 attachment is allowed for '{name}' type attachments on '{record_type}'", name=name.clone(), record_type=record_type.clone()));
                }
            }
        }

        let attached = diesel::connection::Connection::transaction::<Self, Error, _>(db, |db| {
            let blob = AttachmentBlob::create(
                db,
                &AttachmentBlobChangeset {
                    byte_size: data.data.len() as i64,
                    service_name: "s3".to_string(),
                    key: key.clone(),
                    checksum: checksum.clone(),
                    content_type: content_type.clone(),
                    file_name: data.file_name.clone().unwrap_or_default(),
                },
            )?;

            let attached = Attachment::create(
                db,
                &AttachmentChangeset {
                    blob_id: blob.id,
                    record_id,
                    record_type,
                    name,
                },
            )?;

            Ok(attached)
        })
        .map_err(|err| err.to_string())?;

        let upload_result = storage
            .upload(
                key.clone(),
                data.data,
                content_type.clone().unwrap_or_default(),
                checksum.clone(),
            )
            .await
            .map(|_| key);

        if upload_result.is_err() {
            // attempt to delete the attachment
            // if it fails, it fails
            Attachment::detach(db, storage, attached.id).await?;
        }

        upload_result
    }

    /// in poem, we need to pass in the pool itself because the Connection is not Send+Sync which poem handlers require
    #[cfg(feature = "backend_poem")]
    pub async fn attach(
        pool: std::sync::Arc<crate::database::Pool>,
        storage: &Storage,
        name: String,
        record_type: String,
        record_id: ID,
        data: AttachmentData,
        allow_multiple: bool,
        overwrite_existing: bool,
    ) -> Result<String, String> {
        let mut db = pool.clone().get().unwrap();

        let checksum = format!("{:x}", md5::compute(&data.data));
        let file_name = data.file_name.clone();
        let content_type = if file_name.is_some() {
            mime_guess::from_path(file_name.unwrap()).first_raw()
        } else {
            None
        }
        .map(|t| t.to_string());
        let key = Uuid::new_v4().to_string();

        if !allow_multiple {
            let existing =
                Attachment::find_for_record(&mut db, name.clone(), record_type.clone(), record_id);

            if existing.is_ok() {
                // one already exists, we need to delete it
                if overwrite_existing {
                    Attachment::detach(pool.clone(), &storage, existing.unwrap().id).await.map_err(|_| {
                        format!("Could not detach the existing attachment for '{name}' attachment on '{record_type}'", name=name.clone(), record_type=record_type.clone())
                    })?;
                } else {
                    // throw the error
                    return Err(format!("Only 1 attachment is allowed for '{name}' type attachments on '{record_type}'", name=name.clone(), record_type=record_type.clone()));
                }
            }
        }

        let attached =
            diesel::connection::Connection::transaction::<Self, Error, _>(&mut db, |db| {
                let blob = AttachmentBlob::create(
                    db,
                    &AttachmentBlobChangeset {
                        byte_size: data.data.len() as i64,
                        service_name: "s3".to_string(),
                        key: key.clone(),
                        checksum: checksum.clone(),
                        content_type: content_type.clone(),
                        file_name: data.file_name.clone().unwrap_or(String::new()),
                    },
                )?;

                let attached = Attachment::create(
                    db,
                    &AttachmentChangeset {
                        blob_id: blob.id,
                        record_id,
                        record_type,
                        name,
                    },
                )?;

                Ok(attached)
            })
            .map_err(|err| err.to_string())?;

        let upload_result = storage
            .upload(
                key.clone(),
                data.data,
                content_type.clone().unwrap_or("".to_string()),
                checksum.clone(),
            )
            .await
            .map(|_| key);

        if upload_result.is_err() {
            // attempt to delete the attachment
            // if it fails, it fails
            Attachment::detach(pool.clone(), storage, attached.id)
                .await
                .map_err(|err| err.to_string())?;
        }

        upload_result
    }

    /// in actix_web we don't need to support send+sync handlers, so we can use the &mut Connection directly.
    #[cfg(feature = "backend_actix-web")]
    pub async fn detach(db: &mut Connection, storage: &Storage, item_id: ID) -> Result<(), String> {
        let attached =
            Attachment::find_by_id(db, item_id).map_err(|_| "Could not load attachment")?;
        let blob = AttachmentBlob::find_by_id(db, attached.blob_id)
            .map_err(|_| "Could not load attachment blob")?;

        let delete_result = storage.delete(blob.key.clone()).await;

        if delete_result.is_err() {
            // we continue even if there's an error deleting the actual object
            // todo: make this more robust by checking why it failed to delete the object
            //       => is it because it didn't exist?
            println!("{}", delete_result.err().unwrap());
        }

        diesel::connection::Connection::transaction::<(), Error, _>(db, |db| {
            // delete the attachment first because it references the blobs
            Attachment::delete(db, attached.id)?;
            AttachmentBlob::delete(db, blob.id)?;

            Ok(())
        })
        .map_err(|err| err.to_string())?;

        Ok(())
    }

    /// in poem, we need to pass in the pool itself because the Connection is not Send+Sync which poem handlers require
    #[cfg(feature = "backend_poem")]
    pub async fn detach(
        pool: std::sync::Arc<crate::database::Pool>,
        storage: &Storage,
        item_id: ID,
    ) -> Result<(), String> {
        let mut db = pool.get().unwrap();

        let attached =
            Attachment::find_by_id(&mut db, item_id).map_err(|_| "Could not load attachment")?;
        let blob = AttachmentBlob::find_by_id(&mut db, attached.blob_id)
            .map_err(|_| "Could not load attachment blob")?;

        let delete_result = storage.delete(blob.key.clone()).await;

        if delete_result.is_err() {
            // we continue even if there's an error deleting the actual object
            // todo: make this more robust by checking why it failed to delete the object
            //       => is it because it didn't exist?
            println!("{}", delete_result.err().unwrap());
        }

        diesel::connection::Connection::transaction::<(), Error, _>(&mut db, |db| {
            // delete the attachment first because it references the blobs
            Attachment::delete(db, attached.id)?;
            AttachmentBlob::delete(db, blob.id)?;

            Ok(())
        })
        .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub async fn detach_all(
        db: &mut Connection,
        storage: &Storage,
        name: String,
        record_type: String,
        record_id: ID,
    ) -> Result<(), String> {
        let attached = Attachment::find_all_for_record(db, name, record_type, record_id)
            .map_err(|_| "Could not load attachments")?;
        let attached_ids = attached
            .iter()
            .map(|attached| attached.id)
            .collect::<Vec<_>>();
        let blob_ids = attached
            .iter()
            .map(|attached| attached.blob_id)
            .collect::<Vec<_>>();
        let blobs = AttachmentBlob::find_all_by_id(db, blob_ids.clone())
            .map_err(|_| "Could not load attachment blobs")?;
        let keys = blobs
            .iter()
            .map(|blob| blob.key.to_string())
            .collect::<Vec<_>>();

        let delete_result = storage.delete_many(keys).await;

        if delete_result.is_err() {
            // we continue even if there's an error deleting the actual object
            // todo: make this more robust by checking why it failed to delete the objects
            //       => is it because it didn't exist?
            println!("{}", delete_result.err().unwrap());
        }

        diesel::connection::Connection::transaction::<(), Error, _>(db, |db| {
            // delete the attachments first because they reference the blobs
            Attachment::delete_all(db, attached_ids)?;
            AttachmentBlob::delete_all(db, blob_ids)?;

            Ok(())
        })
        .map_err(|err| err.to_string())?;

        Ok(())
    }

    fn create(db: &mut Connection, item: &AttachmentChangeset) -> QueryResult<Self> {
        use super::schema::attachments::dsl::*;

        insert_into(attachments)
            .values(item)
            .get_result::<Attachment>(db)
    }

    fn find_by_id(db: &mut Connection, item_id: ID) -> QueryResult<Self> {
        schema::attachments::table
            .filter(schema::attachments::id.eq(item_id))
            .first(db)
    }

    pub fn find_for_record(
        db: &mut Connection,
        item_name: String,
        item_record_type: String,
        item_record_id: ID,
    ) -> QueryResult<Self> {
        schema::attachments::table
            .filter(schema::attachments::name.eq(item_name))
            .filter(schema::attachments::record_type.eq(item_record_type))
            .filter(schema::attachments::record_id.eq(item_record_id))
            .first::<Self>(db)
    }

    pub fn find_all_for_record(
        db: &mut Connection,
        item_name: String,
        item_record_type: String,
        item_record_id: ID,
    ) -> QueryResult<Vec<Self>> {
        schema::attachments::table
            .filter(schema::attachments::name.eq(item_name))
            .filter(schema::attachments::record_type.eq(item_record_type))
            .filter(schema::attachments::record_id.eq(item_record_id))
            .get_results::<Self>(db)
    }

    pub fn find_all_for_records(
        db: &mut Connection,
        item_name: String,
        item_record_type: String,
        item_record_ids: Vec<ID>,
    ) -> QueryResult<Vec<Self>> {
        schema::attachments::table
            .filter(schema::attachments::name.eq(item_name))
            .filter(schema::attachments::record_type.eq(item_record_type))
            .filter(schema::attachments::record_id.eq_any(item_record_ids))
            .get_results::<Self>(db)
    }

    // fn update(db: &mut Connection, item_id: ID, item: &AttachmentChangeset) -> QueryResult<Self> {
    //     use super::schema::attachments::dsl::*;
    //
    //     diesel::update(attachments.filter(id.eq(item_id)))
    //         .set(item)
    //         .get_result(db)
    // }

    fn delete(db: &mut Connection, item_id: ID) -> QueryResult<usize> {
        use super::schema::attachments::dsl::*;

        diesel::delete(attachments.filter(schema::attachments::id.eq(item_id))).execute(db)
    }

    fn delete_all(db: &mut Connection, item_ids: Vec<ID>) -> QueryResult<usize> {
        use super::schema::attachments::dsl::*;

        diesel::delete(attachments.filter(schema::attachments::id.eq_any(item_ids))).execute(db)
    }
}
