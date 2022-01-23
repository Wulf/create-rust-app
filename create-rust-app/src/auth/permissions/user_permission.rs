use crate::diesel::*;
use crate::schema::*;

use crate::{
    auth::{User, ID, UTC},
    database::Connection,
};
use diesel::QueryResult;
use serde::{Deserialize, Serialize};

#[tsync::tsync]
#[derive(
    Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Associations, AsChangeset,
)]
#[table_name = "user_permissions"]
#[belongs_to(User)]
pub struct UserPermission {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub user_id: ID,
    pub permission: String,
    pub created_at: UTC,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[table_name = "user_permissions"]
pub struct UserPermissionChangeset {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    Don't include non-mutable columns
    (ex: id, created_at/updated_at)
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub user_id: ID,
    pub permission: String,
}

impl UserPermission {
    pub fn create(db: &Connection, item: &UserPermissionChangeset) -> QueryResult<Self> {
        use crate::schema::user_permissions::dsl::*;

        insert_into(user_permissions)
            .values(item)
            .get_result::<UserPermission>(db)
    }

    pub fn read(db: &Connection, item_user_id: ID, item_permission: String) -> QueryResult<Self> {
        use crate::schema::user_permissions::dsl::*;

        user_permissions
            .filter(user_id.eq(item_user_id).and(permission.eq(item_permission)))
            .first::<UserPermission>(db)
    }

    pub fn read_all(db: &Connection, item_user_id: ID) -> QueryResult<Vec<Self>> {
        use crate::schema::user_permissions::dsl::*;

        user_permissions
            .filter(user_id.eq(item_user_id))
            .order(created_at)
            .load::<UserPermission>(db)
    }

    pub fn delete(
        db: &Connection,
        item_user_id: ID,
        item_permission: String,
    ) -> QueryResult<usize> {
        use crate::schema::user_permissions::dsl::*;

        diesel::delete(
            user_permissions.filter(user_id.eq(item_user_id).and(permission.eq(item_permission))),
        )
        .execute(db)
    }
}
