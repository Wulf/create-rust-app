use serde::{Deserialize, Serialize};

use crate::auth::{schema::*, UTC};
use crate::database::Connection;
use crate::diesel::*;

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = role_permissions)]
pub struct RolePermission {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub role: String,
    pub permission: String,
    pub created_at: UTC,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[diesel(table_name = role_permissions)]
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
    pub fn create(db: &mut Connection, item: &RolePermissionChangeset) -> QueryResult<Self> {
        use crate::auth::schema::role_permissions::dsl::*;

        insert_into(role_permissions)
            .values(item)
            .get_result::<RolePermission>(db)
    }

    #[cfg(feature = "database_sqlite")]
    pub fn create_many(
        db: &mut Connection,
        items: Vec<RolePermissionChangeset>,
    ) -> QueryResult<usize> {
        use crate::auth::schema::role_permissions::dsl::*;

        insert_into(role_permissions).values(items).execute(db)
    }

    #[cfg(not(feature = "database_sqlite"))]
    pub fn create_many(
        db: &mut Connection,
        items: Vec<RolePermissionChangeset>,
    ) -> QueryResult<Vec<Self>> {
        use crate::auth::schema::role_permissions::dsl::*;

        insert_into(role_permissions)
            .values(items)
            .get_results::<RolePermission>(db)
    }

    pub fn read(
        db: &mut Connection,
        item_role: String,
        item_permission: String,
    ) -> QueryResult<Self> {
        use crate::auth::schema::role_permissions::dsl::*;

        role_permissions
            .filter(role.eq(item_role).and(permission.eq(item_permission)))
            .first::<RolePermission>(db)
    }

    pub fn read_all(db: &mut Connection, item_role: String) -> QueryResult<Vec<Self>> {
        use crate::auth::schema::role_permissions::dsl::*;

        role_permissions
            .filter(role.eq(item_role))
            .order(created_at)
            .load::<RolePermission>(db)
    }

    pub fn delete(
        db: &mut Connection,
        item_role: String,
        item_permission: String,
    ) -> QueryResult<usize> {
        use crate::auth::schema::role_permissions::dsl::*;

        diesel::delete(
            role_permissions.filter(role.eq(item_role).and(permission.eq(item_permission))),
        )
        .execute(db)
    }

    pub fn delete_many(
        db: &mut Connection,
        item_role: String,
        item_permissions: Vec<String>,
    ) -> QueryResult<usize> {
        use crate::auth::schema::role_permissions::dsl::*;

        diesel::delete(
            role_permissions
                .filter(role.eq(item_role))
                .filter(permission.eq_any(item_permissions)),
        )
        .execute(db)
    }

    pub fn delete_all(db: &mut Connection, item_role: &str) -> QueryResult<usize> {
        use crate::auth::schema::role_permissions::dsl::*;

        diesel::delete(role_permissions.filter(role.eq(item_role))).execute(db)
    }
}
