use serde::{Deserialize, Serialize};

use crate::diesel::*;
use crate::storage::{schema, schema::*, Utc, ID};
use crate::Connection;

#[derive(
    Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, AsChangeset,
)]
#[diesel(table_name = attachment_blobs)]
pub struct AttachmentBlob {
    pub id: ID,

    pub key: String,
    pub file_name: String,
    pub content_type: Option<String>,
    pub byte_size: i64,
    pub checksum: String,
    pub service_name: String,

    pub created_at: Utc,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[diesel(table_name = attachment_blobs)]
pub struct AttachmentBlobChangeset {
    pub key: String,
    pub file_name: String,
    pub content_type: Option<String>,
    pub byte_size: i64,
    pub checksum: String,
    pub service_name: String,
}

impl AttachmentBlob {
    pub fn create(db: &mut Connection, item: &AttachmentBlobChangeset) -> QueryResult<Self> {
        use super::schema::attachment_blobs::dsl::*;

        insert_into(attachment_blobs)
            .values(item)
            .get_result::<AttachmentBlob>(db)
    }

    pub fn find_by_id(db: &mut Connection, item_id: ID) -> QueryResult<Self> {
        use super::schema::attachment_blobs::dsl::*;

        attachment_blobs
            .filter(schema::attachment_blobs::id.eq(item_id))
            .first::<AttachmentBlob>(db)
    }

    pub fn find_all_by_id(db: &mut Connection, item_ids: Vec<ID>) -> QueryResult<Vec<Self>> {
        use super::schema::attachment_blobs::dsl::*;

        attachment_blobs
            .filter(schema::attachment_blobs::id.eq_any(item_ids))
            .load::<AttachmentBlob>(db)
    }

    // fn read_all(db: &mut Connection, pagination: &PaginationParams) -> QueryResult<Vec<Self>> {
    //     use super::schema::attachment_blobs::dsl::*;
    //
    //     attachment_blobs
    //         .order(created_at)
    //         .limit(pagination.page_size)
    //         .offset(pagination.page * std::cmp::max(pagination.page_size, 100))
    //         .load::<AttachmentBlob>(db)
    // }

    // fn update(db: &mut Connection, item_id: ID, item: &AttachmentBlobChangeset) -> QueryResult<Self> {
    //     use super::schema::attachment_blobs::dsl::*;
    //
    //     diesel::update(attachment_blobs.filter(id.eq(item_id)))
    //         .set(item)
    //         .get_result(db)
    // }

    pub fn delete(db: &mut Connection, item_id: ID) -> QueryResult<usize> {
        let query =
            schema::attachment_blobs::table.filter(schema::attachment_blobs::id.eq(item_id));

        diesel::delete(query).execute(db)
    }

    pub fn delete_all(db: &mut Connection, item_ids: Vec<ID>) -> QueryResult<usize> {
        let query =
            schema::attachment_blobs::table.filter(schema::attachment_blobs::id.eq_any(item_ids));

        diesel::delete(query).execute(db)
    }
}
