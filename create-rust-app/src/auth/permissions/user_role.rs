use diesel::QueryResult;
use serde::{Deserialize, Serialize};

use crate::auth::schema::*;
use crate::diesel::*;
use crate::{
    auth::{user::User, Utc, ID},
    database::Connection,
};

#[tsync::tsync]
#[derive(
    Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Associations, AsChangeset,
)]
#[diesel(table_name = user_roles, belongs_to(User))]
/// Rust struct modeling an entry in the user_roles table
pub struct UserRole {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub user_id: ID,
    pub role: String,
    pub created_at: Utc,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[diesel(table_name = user_roles)]
/// Rust struct modeling mutable data in an entry in the user_roles table
pub struct UserRoleChangeset {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    Don't include non-mutable columns
    (ex: id, created_at/updated_at)
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub user_id: ID,
    pub role: String,
}

impl UserRole {
    /// Create an entry in [`db`](`Connection`)'s user_roles table that has the data stored in [`item`](`UserRoleChangeset`)
    pub fn create(db: &mut Connection, item: &UserRoleChangeset) -> QueryResult<Self> {
        use crate::auth::schema::user_roles::dsl::*;

        insert_into(user_roles)
            .values(item)
            .get_result::<UserRole>(db)
    }

    #[cfg(feature = "database_sqlite")]
    /// Create an entry in [`db`](`Connection`)'s user_roles table for each [element](`UserRoleChangeset`) in `items`
    pub fn create_many(db: &mut Connection, items: Vec<UserRoleChangeset>) -> QueryResult<usize> {
        use crate::auth::schema::user_roles::dsl::*;

        insert_into(user_roles).values(items).execute(db)
    }

    #[cfg(not(feature = "database_sqlite"))]
    /// Create an entry in [`db`](`Connection`)'s user_roles table for each [element](`UserRoleChangeset`) in `items`
    pub fn create_many(
        db: &mut Connection,
        items: Vec<UserRoleChangeset>,
    ) -> QueryResult<Vec<Self>> {
        use crate::auth::schema::user_roles::dsl::*;

        insert_into(user_roles)
            .values(items)
            .get_results::<UserRole>(db)
    }

    /// Read from [`db`](`Connection`), querying for an entry in the user_roles table that has
    /// (`item_user_id`,`item_role`) as it's primary keys
    pub fn read(db: &mut Connection, item_user_id: ID, item_role: String) -> QueryResult<Self> {
        use crate::auth::schema::user_roles::dsl::*;

        user_roles
            .filter(user_id.eq(item_user_id).and(role.eq(item_role)))
            .first::<UserRole>(db)
    }

    /// Read from [`db`](`Connection`), querying for every entry in the user_roles table that has
    /// `item_user_id` as one of its primary keys
    pub fn read_all(db: &mut Connection, item_user_id: ID) -> QueryResult<Vec<Self>> {
        use crate::auth::schema::user_roles::dsl::*;

        user_roles
            .filter(user_id.eq(item_user_id))
            .order(created_at)
            .load::<UserRole>(db)
    }

    /// Delete the entry in [`db`](`Connection`)'s user_roles table that has
    /// (`item_user_id`,`item_role`) as it's primary keys
    pub fn delete(db: &mut Connection, item_user_id: ID, item_role: String) -> QueryResult<usize> {
        use crate::auth::schema::user_roles::dsl::*;

        diesel::delete(user_roles.filter(user_id.eq(item_user_id).and(role.eq(item_role))))
            .execute(db)
    }

    /// Delete every entry in [`db`](`Connection`)'s user_roles table that has
    /// `item_user_id`, and an element of`item_roles` as it's primary keys
    pub fn delete_many(
        db: &mut Connection,
        item_user_id: ID,
        item_roles: Vec<String>,
    ) -> QueryResult<usize> {
        use crate::auth::schema::user_roles::dsl::*;

        diesel::delete(user_roles.filter(user_id.eq(item_user_id).and(role.eq_any(item_roles))))
            .execute(db)
    }
}
