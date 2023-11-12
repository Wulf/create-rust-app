use serde::{Deserialize, Serialize};

use crate::diesel::{
    insert_into, AsChangeset, ExpressionMethods, Identifiable, Insertable, QueryDsl, QueryResult,
    Queryable, RunQueryDsl,
};
use crate::storage::{schema, schema::attachment_blobs, Utc, ID};
use crate::Connection;

#[allow(clippy::module_name_repetitions)]
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

#[allow(clippy::module_name_repetitions)]
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
    /// Create an entry in [`db`](`Connection`)'s `attachment_blobs` table using the data in [`item`](`AttachmentBlobChangeset`)
    ///
    /// # Errors
    /// * Diesel error
    pub fn create(db: &mut Connection, item: &AttachmentBlobChangeset) -> QueryResult<Self> {
        use super::schema::attachment_blobs::dsl::attachment_blobs;

        insert_into(attachment_blobs)
            .values(item)
            .get_result::<Self>(db)
    }

    /// Read from [`db`](`Connection`), querying for an entry in the `attachment_blobs` table who's primary key matches [`item_id`](`ID`)
    ///
    /// # Errors
    /// * Diesel error
    pub fn find_by_id(db: &mut Connection, item_id: ID) -> QueryResult<Self> {
        use super::schema::attachment_blobs::dsl::attachment_blobs;

        attachment_blobs
            .filter(schema::attachment_blobs::id.eq(item_id))
            .first::<Self>(db)
    }

    /// Read from [`db`](`Connection`), querying for an entry in the `attachment_blobs` table for each [`item_id`](`ID`) in `item_ids`
    ///
    /// # Errors
    /// * Diesel error
    pub fn find_all_by_id(db: &mut Connection, item_ids: Vec<ID>) -> QueryResult<Vec<Self>> {
        use super::schema::attachment_blobs::dsl::attachment_blobs;

        attachment_blobs
            .filter(schema::attachment_blobs::id.eq_any(item_ids))
            .load::<Self>(db)
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

    /// Delete the entry in [`db`](`Connection`)'s `attachment_blobs` table who's primary key matches [`item_id`](`ID`)
    ///
    /// # Errors
    /// * Diesel error
    pub fn delete(db: &mut Connection, item_id: ID) -> QueryResult<usize> {
        let query =
            schema::attachment_blobs::table.filter(schema::attachment_blobs::id.eq(item_id));

        diesel::delete(query).execute(db)
    }

    /// Delete every entry in [`db`](`Connection`)'s `attachment_blobs` table who's primary key matches [`item_id`](`ID`)
    ///
    /// # Errors
    /// * Diesel error
    pub fn delete_all(db: &mut Connection, item_ids: Vec<ID>) -> QueryResult<usize> {
        let query =
            schema::attachment_blobs::table.filter(schema::attachment_blobs::id.eq_any(item_ids));

        diesel::delete(query).execute(db)
    }
}
