use crate::diesel::*;
use crate::schema::*;

use super::{PaginationParams, ID, UTC};
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
#[table_name = "users"]
pub struct User {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub id: ID,

    pub email: String,
    pub hash_password: String,
    pub activated: bool,

    pub created_at: UTC,
    pub updated_at: UTC,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[table_name = "users"]
pub struct UserChangeset {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    Don't include non-mutable columns
    (ex: id, created_at/updated_at)
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub email: String,
    pub hash_password: String,
    pub activated: bool,
}

impl User {
    pub fn create(db: &Connection, item: &UserChangeset) -> QueryResult<Self> {
        use crate::schema::users::dsl::*;

        insert_into(users).values(item).get_result::<User>(db)
    }

    pub fn read(db: &Connection, item_id: ID) -> QueryResult<Self> {
        use crate::schema::users::dsl::*;

        users.filter(id.eq(item_id)).first::<User>(db)
    }

    pub fn find_by_email(db: &Connection, item_email: String) -> QueryResult<Self> {
        use crate::schema::users::dsl::*;

        users.filter(email.eq(item_email)).first::<User>(db)
    }

    pub fn read_all(db: &Connection, pagination: &PaginationParams) -> QueryResult<Vec<Self>> {
        use crate::schema::users::dsl::*;

        users
            .order(created_at)
            .limit(pagination.page_size)
            .offset(
                pagination.page
                    * std::cmp::max(pagination.page_size, PaginationParams::MAX_PAGE_SIZE as i64),
            )
            .load::<User>(db)
    }

    pub fn update(db: &Connection, item_id: ID, item: &UserChangeset) -> QueryResult<Self> {
        use crate::schema::users::dsl::*;

        diesel::update(users.filter(id.eq(item_id)))
            .set(item)
            .get_result(db)
    }

    pub fn delete(db: &Connection, item_id: ID) -> QueryResult<usize> {
        use crate::schema::users::dsl::*;

        diesel::delete(users.filter(id.eq(item_id))).execute(db)
    }
}
