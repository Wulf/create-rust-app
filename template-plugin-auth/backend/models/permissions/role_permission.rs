use crate::diesel::*;
use crate::schema::*;

use crate::models::*;
use crate::DB;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[table_name = "role_permissions"]
pub struct RolePermission {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub role: String,
    pub permission: String,
    pub created_at: UTC,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[table_name = "role_permissions"]
pub struct RolePermissionChangeset {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    Don't include non-mutable columns
    (ex: id, created_at/updated_at)
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub role: String,
    pub permission: String,
}

impl RolePermission {
    pub fn create(db: &DB, item: &RolePermissionChangeset) -> QueryResult<Self> {
        use crate::schema::role_permissions::dsl::*;

        insert_into(role_permissions)
            .values(item)
            .get_result::<RolePermission>(db)
    }

    pub fn read(db: &DB, item_role: String, item_permission: String) -> QueryResult<Self> {
        use crate::schema::role_permissions::dsl::*;

        role_permissions
            .filter(role.eq(item_role).and(permission.eq(item_permission)))
            .first::<RolePermission>(db)
    }

    pub fn read_all(db: &DB, item_role: String) -> QueryResult<Vec<Self>> {
        use crate::schema::role_permissions::dsl::*;

        role_permissions
            .filter(role.eq(item_role))
            .order(created_at)
            .load::<RolePermission>(db)
    }

    pub fn delete(db: &DB, item_role: String, item_permission: String) -> QueryResult<usize> {
        use crate::schema::role_permissions::dsl::*;

        diesel::delete(
            role_permissions.filter(role.eq(item_role).and(permission.eq(item_permission))),
        )
        .execute(db)
    }
}
