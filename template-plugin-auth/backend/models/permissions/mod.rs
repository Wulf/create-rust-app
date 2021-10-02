mod role_permission;
mod user_permission;
mod user_role;

use diesel::{
    sql_query,
    sql_types::{Integer, Text},
    RunQueryDsl,
};

use crate::{
    models::permissions::user_permission::{UserPermission, UserPermissionChangeset},
    DB,
};
use serde::{Deserialize, Serialize};

use self::{
    role_permission::{RolePermission, RolePermissionChangeset},
    user_role::{UserRole, UserRoleChangeset},
};

use super::ID;
use anyhow::Result;

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, QueryableByName, Clone)]
pub struct Permission {
    #[sql_type = "Text"]
    pub from_role: String,

    #[sql_type = "Text"]
    pub permission: String,
}

impl Permission {
    // pub fn is_granted_to_user(db: &DB, user_id: ID, permission: String) -> Result<bool> {
    //   let permissions = Permission::for_user(&db, user_id)?;
    //   let user_has_permission = permissions.iter().any(|perm| perm.permission == permission);

    //   Ok(user_has_permission)
    // }

    pub fn grant_to_user(db: &DB, user_id: ID, permission: &str) -> Result<bool> {
        let granted = UserPermission::create(
            &db,
            &UserPermissionChangeset {
                permission: permission.to_string(),
                user_id,
            },
        );

        Ok(granted.is_ok())
    }

    pub fn grant_to_role(db: &DB, role: &str, permission: &str) -> Result<bool> {
        let granted = RolePermission::create(
            &db,
            &RolePermissionChangeset {
                permission: permission.to_string(),
                role: role.to_string(),
            },
        );

        Ok(granted.is_ok())
    }

    pub fn revoke_from_user(db: &DB, user_id: ID, permission: &str) -> Result<bool> {
        let deleted = UserPermission::delete(&db, user_id, permission.to_string());

        Ok(deleted.is_ok())
    }

    pub fn revoke_from_role(db: &DB, user_id: ID, permission: &str) -> Result<bool> {
        let deleted = UserPermission::delete(&db, user_id, permission.to_string());

        Ok(deleted.is_ok())
    }

    pub fn assign_role(db: &DB, user_id: ID, role: &str) -> Result<bool> {
        let assigned = UserRole::create(
            &db,
            &UserRoleChangeset {
                user_id: user_id,
                role: role.to_string(),
            },
        );

        Ok(assigned.is_ok())
    }

    pub fn unassign_role(db: &DB, user_id: ID, role: &str) -> Result<bool> {
        let unassigned = UserRole::delete(&db, user_id, role.to_string());

        Ok(unassigned.is_ok())
    }

    pub fn for_user(db: &DB, user_id: ID) -> Result<Vec<Permission>> {
        let permissions = sql_query(
            r#"
      SELECT 
        permission AS permission,
        NULL AS from_role
      FROM user_permissions
      WHERE user_permissions.user_id = $1

      UNION

      SELECT
        permission AS permission,
        user_roles.role AS form_role
      FROM user_roles
      INNER JOIN role_permissions ON user_roles.role = role_permissions.role
      WHERE user_roles.user_id = $2
      "#,
        );

        let permissions = permissions
            .bind::<Integer, _>(user_id)
            .bind::<Integer, _>(user_id)
            .get_results::<Permission>(db)?;

        Ok(permissions)
    }
}
