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
#[diesel(table_name=user_permissions,belongs_to(User))]
/// Rust struct modeling an entry in the user_permissions table
pub struct UserPermission {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub user_id: ID,
    pub permission: String,
    pub created_at: Utc,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[diesel(table_name=user_permissions)]
/// Rust struct modeling mutable data in an entry in the user_permissions table
pub struct UserPermissionChangeset {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    Don't include non-mutable columns
    (ex: id, created_at/updated_at)
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub user_id: ID,
    pub permission: String,
}

/// CRUD functions for [`UserPermission`]
impl UserPermission {
    /// Create an entry in [`db`](`Connection`)'s user_permissions table that has the data stored in [`item`](`UserPermissionChangeset`)
    pub fn create(db: &mut Connection, item: &UserPermissionChangeset) -> QueryResult<Self> {
        use crate::auth::schema::user_permissions::dsl::*;

        insert_into(user_permissions)
            .values(item)
            .get_result::<UserPermission>(db)
    }

    #[cfg(feature = "database_sqlite")]
    /// Create an entry in [`db`](`Connection`)'s user_permissions table for each [element](`UserPermissionChangeset`) in `items`
    pub fn create_many(
        db: &mut Connection,
        items: Vec<UserPermissionChangeset>,
    ) -> QueryResult<usize> {
        use crate::auth::schema::user_permissions::dsl::*;

        insert_into(user_permissions).values(items).execute(db)
    }

    #[cfg(not(feature = "database_sqlite"))]
    /// Create an entry in [`db`](`Connection`)'s user_permissions table for each [element](`UserPermissionChangeset`) in `items`
    pub fn create_many(
        db: &mut Connection,
        items: Vec<UserPermissionChangeset>,
    ) -> QueryResult<Self> {
        use crate::auth::schema::user_permissions::dsl::*;

        insert_into(user_permissions)
            .values(items)
            .get_result::<UserPermission>(db)
    }

    /// Read from [`db`](`Connection`), querying for an entry in the user_permissions table that has
    /// (`item_user_id`,`item_permission`) as it's primary keys
    pub fn read(
        db: &mut Connection,
        item_user_id: ID,
        item_permission: String,
    ) -> QueryResult<Self> {
        use crate::auth::schema::user_permissions::dsl::*;

        user_permissions
            .filter(user_id.eq(item_user_id).and(permission.eq(item_permission)))
            .first::<UserPermission>(db)
    }

    /// Read from [`db`](`Connection`), querying for every entry in the user_permissions table that has
    /// `item_user_id` as one of its primary keys
    pub fn read_all(db: &mut Connection, item_user_id: ID) -> QueryResult<Vec<Self>> {
        use crate::auth::schema::user_permissions::dsl::*;

        user_permissions
            .filter(user_id.eq(item_user_id))
            .order(created_at)
            .load::<UserPermission>(db)
    }

    /// Delete the entry in [`db`](`Connection`)'s user_permissions table that has
    /// (`item_user_id`,`item_permission`) as it's primary keys
    pub fn delete(
        db: &mut Connection,
        item_user_id: ID,
        item_permission: String,
    ) -> QueryResult<usize> {
        use crate::auth::schema::user_permissions::dsl::*;

        diesel::delete(
            user_permissions.filter(user_id.eq(item_user_id).and(permission.eq(item_permission))),
        )
        .execute(db)
    }

    /// Delete every entry in [`db`](`Connection`)'s user_permissions table that has
    /// `item_user_id`, and an element of`item_permissions` as it's primary keys
    pub fn delete_many(
        db: &mut Connection,
        item_user_id: ID,
        item_permissions: Vec<String>,
    ) -> QueryResult<usize> {
        use crate::auth::schema::user_permissions::dsl::*;

        diesel::delete(
            user_permissions
                .filter(user_id.eq(item_user_id))
                .filter(permission.eq_any(item_permissions)),
        )
        .execute(db)
    }

    /// Delete the entry in [`db`](`Connection`)'s user_permissions table that has
    /// `item_user_id` as one of it's primary keys
    pub fn delete_all(db: &mut Connection, item_user_id: ID) -> QueryResult<usize> {
        use crate::auth::schema::user_permissions::dsl::*;

        diesel::delete(user_permissions.filter(user_id.eq(item_user_id))).execute(db)
    }
}
