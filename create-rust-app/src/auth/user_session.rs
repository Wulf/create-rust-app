use super::schema::*;
use crate::diesel::*;

use super::user::User;
use super::{PaginationParams, Utc, ID};
use crate::database::Connection;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};

#[tsync::tsync]
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Queryable,
    Insertable,
    Identifiable,
    Associations,
    AsChangeset,
)]
#[diesel(table_name=user_sessions, belongs_to(User))]
pub struct UserSession {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub id: ID,

    pub user_id: ID,
    pub refresh_token: String,
    pub device: Option<String>,

    pub created_at: Utc,
    #[cfg(not(feature = "database_sqlite"))]
    pub updated_at: Utc,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[diesel(table_name=user_sessions)]
pub struct UserSessionChangeset {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    Don't include non-mutable columns
    (ex: id, created_at/updated_at)
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub user_id: ID,
    pub refresh_token: String,
    pub device: Option<String>,
}

impl UserSession {
    /// Create an entry in [`db`](`Connection`)'s `user_sessions` table using the data in [`item`](`UserSessionChangeset`)
    pub fn create(db: &mut Connection, item: &UserSessionChangeset) -> QueryResult<Self> {
        use super::schema::user_sessions::dsl::*;

        insert_into(user_sessions)
            .values(item)
            .get_result::<UserSession>(db)
    }

    /// Read from [`db`](`Connection`), querying for an entry in the `user_sessions`
    /// who's primary key matches [`item_id`](`ID`)
    pub fn read(db: &mut Connection, item_id: ID) -> QueryResult<Self> {
        use super::schema::user_sessions::dsl::*;

        user_sessions
            .filter(id.eq(item_id))
            .first::<UserSession>(db)
    }

    /// Query [`db`](`Connection`)'s `user_sessions` table for an entry
    /// who's `refresh_token` matches the given `item_refresh_token`
    pub fn find_by_refresh_token(
        db: &mut Connection,
        item_refresh_token: &str,
    ) -> QueryResult<Self> {
        use super::schema::user_sessions::dsl::*;

        user_sessions
            .filter(refresh_token.eq(item_refresh_token))
            .first::<UserSession>(db)
    }

    /// Read from [`db`](`Connection`), return entries of the `user_sessions` table,
    /// paginated according to [`pagination`](`PaginationParams`)
    pub fn read_all(
        db: &mut Connection,
        pagination: &PaginationParams,
        item_user_id: ID,
    ) -> QueryResult<Vec<Self>> {
        use super::schema::user_sessions::dsl::*;

        user_sessions
            .filter(user_id.eq(item_user_id))
            .order(created_at)
            .limit(pagination.page_size)
            .offset(
                pagination.page
                    * std::cmp::min(pagination.page_size, PaginationParams::MAX_PAGE_SIZE as i64),
            )
            .load::<UserSession>(db)
    }

    /// Query [`db`](`Connection`) for all entries in the `user_sessions` table
    /// who's `user_id` matches the given [`item_user_id`]
    pub fn count_all(db: &mut Connection, item_user_id: ID) -> QueryResult<i64> {
        use super::schema::user_sessions::dsl::*;

        user_sessions
            .filter(user_id.eq(item_user_id))
            .count()
            .get_result(db)
    }

    /// Update the entry in [`db`](`Connection`)'s `user_sessions` table who's primary key matches
    /// [`item_id`](`ID`), with the data in [`item`](`UserSessionChangeset`)
    pub fn update(
        db: &mut Connection,
        item_id: ID,
        item: &UserSessionChangeset,
    ) -> QueryResult<Self> {
        use super::schema::user_sessions::dsl::*;

        diesel::update(user_sessions.filter(id.eq(item_id)))
            .set(item)
            .get_result(db)
    }

    /// Delete the entry in [`db`](`Connection`)'s `user_sessions` table who's
    /// primary key matches [`item_id`](`ID`)
    pub fn delete(db: &mut Connection, item_id: ID) -> QueryResult<usize> {
        use super::schema::user_sessions::dsl::*;

        diesel::delete(user_sessions.filter(id.eq(item_id))).execute(db)
    }

    /// Delete all entries in [`db`](`Connection`)'s `user_sessions` table who's
    /// 'user_id' matches [`item_user_id`](`ID`)
    pub fn delete_all_for_user(db: &mut Connection, item_user_id: ID) -> QueryResult<usize> {
        use super::schema::user_sessions::dsl::*;

        diesel::delete(user_sessions.filter(user_id.eq(item_user_id))).execute(db)
    }
}
