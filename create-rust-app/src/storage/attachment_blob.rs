use diesel::dsl::any;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};

use crate::Connection;
use crate::diesel::*;
use crate::storage::{ID, schema, UTC};

use super::schema::*;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, Associations, AsChangeset)]
#[table_name = "attachment_blobs"]
pub struct AttachmentBlob {
    pub id: ID,

    pub key: String,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub byte_size: i64,
    pub checksum: String,
    pub service_name: String,

    pub created_at: UTC,
}
#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[table_name = "attachment_blobs"]
pub struct AttachmentBlobChangeset {
    pub key: String,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub byte_size: i64,
    pub checksum: String,
    pub service_name: String
}

impl AttachmentBlob {
    pub fn create(db: &Connection, item: &AttachmentBlobChangeset) -> QueryResult<Self> {
        use super::schema::attachment_blobs::dsl::*;

        insert_into(attachment_blobs)
            .values(item)
            .get_result::<AttachmentBlob>(db)
    }

    pub fn find_by_id(db: &Connection, item_id: ID) -> QueryResult<Self> {
        use super::schema::attachment_blobs::dsl::*;

        attachment_blobs.filter(id.eq(item_id)).first::<AttachmentBlob>(db)
    }

    pub fn find_all_by_id(db: &Connection, item_ids: Vec<ID>) -> QueryResult<Vec<Self>> {
        use super::schema::attachment_blobs::dsl::*;

        attachment_blobs.filter(id.eq(any(item_ids))).load::<AttachmentBlob>(db)
    }

    // fn read_all(db: &Connection, pagination: &PaginationParams) -> QueryResult<Vec<Self>> {
    //     use super::schema::attachment_blobs::dsl::*;
    //
    //     attachment_blobs
    //         .order(created_at)
    //         .limit(pagination.page_size)
    //         .offset(pagination.page * std::cmp::max(pagination.page_size, 100))
    //         .load::<AttachmentBlob>(db)
    // }

    // fn update(db: &Connection, item_id: ID, item: &AttachmentBlobChangeset) -> QueryResult<Self> {
    //     use super::schema::attachment_blobs::dsl::*;
    //
    //     diesel::update(attachment_blobs.filter(id.eq(item_id)))
    //         .set(item)
    //         .get_result(db)
    // }

    pub fn delete(db: &Connection, item_id: ID) -> QueryResult<usize> {
        let query = schema::attachment_blobs::table.filter(schema::attachment_blobs::id.eq(item_id));

        diesel::delete(query).execute(db)
    }

    pub fn delete_all(db: &Connection, item_ids: Vec<ID>) -> QueryResult<usize> {
        let query = schema::attachment_blobs::table.filter(schema::attachment_blobs::id.eq(any(item_ids)));

        diesel::delete(query).execute(db)
    }
}
