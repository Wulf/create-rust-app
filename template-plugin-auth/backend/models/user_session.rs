use crate::diesel::*;
use crate::schema::*;

use crate::models::user::User;
use crate::models::{PaginationParams, ID, UTC};
use crate::DB;
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
#[table_name = "user_sessions"]
#[belongs_to(User)]
pub struct UserSession {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub id: ID,

    pub user_id: ID,
    pub refresh_token: String,
    pub device: Option<String>,

    pub created_at: UTC,
    pub updated_at: UTC,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[table_name = "user_sessions"]
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
    pub fn create(db: &DB, item: &UserSessionChangeset) -> QueryResult<Self> {
        use crate::schema::user_sessions::dsl::*;

        insert_into(user_sessions)
            .values(item)
            .get_result::<UserSession>(db)
    }

    pub fn read(db: &DB, item_id: ID) -> QueryResult<Self> {
        use crate::schema::user_sessions::dsl::*;

        user_sessions
            .filter(id.eq(item_id))
            .first::<UserSession>(db)
    }

    pub fn find_by_refresh_token(db: &DB, item_refresh_token: &str) -> QueryResult<Self> {
        use crate::schema::user_sessions::dsl::*;

        user_sessions
            .filter(refresh_token.eq(item_refresh_token))
            .first::<UserSession>(db)
    }

    pub fn read_all(db: &DB, pagination: &PaginationParams, user_id: ID) -> QueryResult<Vec<Self>> {
        use crate::schema::user_sessions::dsl::*;

        user_sessions
            .filter(user_id.eq(user_id))
            .order(created_at)
            .limit(pagination.page_size)
            .offset(
                pagination.page
                    * std::cmp::min(pagination.page_size, PaginationParams::MAX_PAGE_SIZE as i64),
            )
            .load::<UserSession>(db)
    }

    pub fn count_all(db: &DB, user_id: ID) -> QueryResult<i64> {
        use crate::schema::user_sessions::dsl::*;

        user_sessions
            .filter(user_id.eq(user_id))
            .count()
            .get_result(db)
    }

    pub fn update(db: &DB, item_id: ID, item: &UserSessionChangeset) -> QueryResult<Self> {
        use crate::schema::user_sessions::dsl::*;

        diesel::update(user_sessions.filter(id.eq(item_id)))
            .set(item)
            .get_result(db)
    }

    pub fn delete(db: &DB, item_id: ID) -> QueryResult<usize> {
        use crate::schema::user_sessions::dsl::*;

        diesel::delete(user_sessions.filter(id.eq(item_id))).execute(db)
    }

    pub fn delete_all_for_user(db: &DB, item_user_id: ID) -> QueryResult<usize> {
        use crate::schema::user_sessions::dsl::*;

        diesel::delete(user_sessions.filter(user_id.eq(item_user_id))).execute(db)
    }
}
