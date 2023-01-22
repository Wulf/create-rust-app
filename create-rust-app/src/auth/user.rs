/// Create an entry in [`db`](`Connection`)'s `users` table using the data in [`item`](`UserChangeset`)

/// Read from [`db`](`Connection`), querying for an entry in the `users`
/// who's primary key matches [`item_id`](`ID`)

/// Queries [`db`](`Connection`)'s `users` table for an entry
/// with an email that matches the given `item_email`

/// Read from [`db`](`Connection`), return entries of the `users` table,
/// paginated according to [`pagination`](`PaginationParams`)

/// Update the entry in [`db`](`Connection`)'s `users` table who's primary key matches
/// [`item_id`](`ID`), with the data in [`item`](`UserChangeset`)

/// Delete the entry in [`db`](`Connection`)'s `users` table who's
/// primary key matches [`item_id`](`ID`)
use super::schema::*;
use crate::diesel::*;

use super::{PaginationParams, Utc, ID};
use crate::database::Connection;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};

#[tsync::tsync]
#[derive(
    Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, AsChangeset,
)]
#[diesel(table_name=users)]
pub struct User {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub id: ID,

    pub email: String,
    pub hash_password: String,
    pub activated: bool,

    pub created_at: Utc,
    #[cfg(not(feature = "database_sqlite"))]
    pub updated_at: Utc,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[diesel(table_name=users)]
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
    /// Create an entry in [`db`](`Connection`)'s `users` table using the data in [`item`](`UserChangeset`)
    pub fn create(db: &mut Connection, item: &UserChangeset) -> QueryResult<Self> {
        use super::schema::users::dsl::*;

        insert_into(users).values(item).get_result::<User>(db)
    }

    /// Read from [`db`](`Connection`), querying for an entry in the `users`
    /// who's primary key matches [`item_id`](`ID`)
    pub fn read(db: &mut Connection, item_id: ID) -> QueryResult<Self> {
        use super::schema::users::dsl::*;

        users.filter(id.eq(item_id)).first::<User>(db)
    }

    /// Queries [`db`](`Connection`)'s `users` table for an entry
    /// with an email that matches the given `item_email`
    pub fn find_by_email(db: &mut Connection, item_email: String) -> QueryResult<Self> {
        use super::schema::users::dsl::*;

        users.filter(email.eq(item_email)).first::<User>(db)
    }

    /// Read from [`db`](`Connection`), return entries of the `users` table,
    /// paginated according to [`pagination`](`PaginationParams`)
    pub fn read_all(db: &mut Connection, pagination: &PaginationParams) -> QueryResult<Vec<Self>> {
        use super::schema::users::dsl::*;

        users
            .order(created_at)
            .limit(pagination.page_size)
            .offset(
                pagination.page
                    * std::cmp::max(pagination.page_size, PaginationParams::MAX_PAGE_SIZE as i64),
            )
            .load::<User>(db)
    }

    /// Update the entry in [`db`](`Connection`)'s `users` table who's primary key matches
    /// [`item_id`](`ID`), with the data in [`item`](`UserChangeset`)
    pub fn update(db: &mut Connection, item_id: ID, item: &UserChangeset) -> QueryResult<Self> {
        use super::schema::users::dsl::*;

        diesel::update(users.filter(id.eq(item_id)))
            .set(item)
            .get_result(db)
    }

    /// Delete the entry in [`db`](`Connection`)'s `users` table who's
    /// primary key matches [`item_id`](`ID`)
    pub fn delete(db: &mut Connection, item_id: ID) -> QueryResult<usize> {
        use super::schema::users::dsl::*;

        diesel::delete(users.filter(id.eq(item_id))).execute(db)
    }
}
