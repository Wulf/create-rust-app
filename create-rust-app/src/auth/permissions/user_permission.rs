use diesel::QueryResult;
use serde::{Deserialize, Serialize};

use crate::auth::schema::*;
use crate::diesel::*;
use crate::{
    auth::{user::User, ID, UTC},
    database::Connection,
};

#[tsync::tsync]
#[derive(
    Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Associations, AsChangeset,
)]
#[diesel(table_name=user_permissions,belongs_to(User))]
pub struct UserPermission {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub user_id: ID,
    pub permission: String,
    pub created_at: UTC,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[diesel(table_name=user_permissions)]
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
    pub fn create(db: &mut Connection, item: &UserPermissionChangeset) -> QueryResult<Self> {
        use crate::auth::schema::user_permissions::dsl::*;

        insert_into(user_permissions)
            .values(item)
            .get_result::<UserPermission>(db)
    }

    #[cfg(feature = "database_sqlite")]
    pub fn create_many(
        db: &mut Connection,
        items: Vec<UserPermissionChangeset>,
    ) -> QueryResult<usize> {
        use crate::auth::schema::user_permissions::dsl::*;

        insert_into(user_permissions).values(items).execute(db)
    }

    #[cfg(not(feature = "database_sqlite"))]
    pub fn create_many(
        db: &mut Connection,
        items: Vec<UserPermissionChangeset>,
    ) -> QueryResult<Self> {
        use crate::auth::schema::user_permissions::dsl::*;

        insert_into(user_permissions)
            .values(items)
            .get_result::<UserPermission>(db)
    }

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

    pub fn read_all(db: &mut Connection, item_user_id: ID) -> QueryResult<Vec<Self>> {
        use crate::auth::schema::user_permissions::dsl::*;

        user_permissions
            .filter(user_id.eq(item_user_id))
            .order(created_at)
            .load::<UserPermission>(db)
    }

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

    pub fn delete_all(db: &mut Connection, item_user_id: ID) -> QueryResult<usize> {
        use crate::auth::schema::user_permissions::dsl::*;

        diesel::delete(user_permissions.filter(user_id.eq(item_user_id))).execute(db)
    }
}
