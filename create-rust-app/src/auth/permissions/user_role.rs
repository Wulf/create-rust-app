use diesel::dsl::any;
use crate::diesel::*;
use crate::auth::schema::*;

use crate::{
    auth::{user::User, ID, UTC},
    database::Connection,
};
use diesel::QueryResult;
use serde::{Deserialize, Serialize};

#[tsync::tsync]
#[derive(
    Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Associations, AsChangeset,
)]
#[table_name = "user_roles"]
#[belongs_to(User)]
pub struct UserRole {
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    Add columns here in the same order as the schema
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub user_id: ID,
    pub role: String,
    pub created_at: UTC,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[table_name = "user_roles"]
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
    pub fn create(db: &Connection, item: &UserRoleChangeset) -> QueryResult<Self> {
        use crate::auth::schema::user_roles::dsl::*;

        insert_into(user_roles)
            .values(item)
            .get_result::<UserRole>(db)
    }

    pub fn create_many(db: &Connection, items: Vec<UserRoleChangeset>) -> QueryResult<Vec<Self>> {
        use crate::auth::schema::user_roles::dsl::*;

        insert_into(user_roles)
            .values(items)
            .get_results::<UserRole>(db)
    }

    pub fn read(db: &Connection, item_user_id: ID, item_role: String) -> QueryResult<Self> {
        use crate::auth::schema::user_roles::dsl::*;

        user_roles
            .filter(user_id.eq(item_user_id).and(role.eq(item_role)))
            .first::<UserRole>(db)
    }

    pub fn read_all(db: &Connection, item_user_id: ID) -> QueryResult<Vec<Self>> {
        use crate::auth::schema::user_roles::dsl::*;

        user_roles
            .filter(user_id.eq(item_user_id))
            .order(created_at)
            .load::<UserRole>(db)
    }

    pub fn delete(db: &Connection, item_user_id: ID, item_role: String) -> QueryResult<usize> {
        use crate::auth::schema::user_roles::dsl::*;

        diesel::delete(user_roles.filter(user_id.eq(item_user_id).and(role.eq(item_role))))
            .execute(db)
    }

    pub fn delete_many(db: &Connection, item_user_id: ID, item_roles: Vec<String>) -> QueryResult<usize> {
        use crate::auth::schema::user_roles::dsl::*;

        diesel::delete(user_roles.filter(user_id.eq(item_user_id).and(role.eq(any(item_roles)))))
            .execute(db)
    }
}
