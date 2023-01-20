mod role_permission;
mod user_permission;
mod user_role;

pub use role_permission::{RolePermission, RolePermissionChangeset};
use std::hash::{Hash, Hasher};
pub use user_permission::{UserPermission, UserPermissionChangeset};
pub use user_role::{UserRole, UserRoleChangeset};

use crate::database::Connection;
use anyhow::Result;
use diesel::{
    sql_query,
    sql_types::{Integer, Text},
    RunQueryDsl,
};
use serde::{Deserialize, Serialize};

use crate::auth::ID;

pub struct Role;

#[derive(Debug, Serialize, Deserialize, QueryableByName, Clone)]
struct RoleQueryRow {
    #[diesel(sql_type=Text)]
    role: String,
}

impl Role {
    /// assign `role` to the User whose id is [`user_id`](`ID`)
    ///
    /// Returns true if successful
    pub fn assign(db: &mut Connection, user_id: ID, role: &str) -> Result<bool> {
        let assigned = UserRole::create(
            db,
            &UserRoleChangeset {
                user_id,
                role: role.to_string(),
            },
        );

        Ok(assigned.is_ok())
    }

    /// assigns every role in `roles` to the User whose id is [`user_id`](`ID`)
    ///
    /// returns true if successful
    pub fn assign_many(db: &mut Connection, user_id: ID, roles: Vec<String>) -> Result<bool> {
        let assigned = UserRole::create_many(
            db,
            roles
                .into_iter()
                .map(|r| UserRoleChangeset { user_id, role: r })
                .collect::<Vec<_>>(),
        );

        Ok(assigned.is_ok())
    }

    /// unassigns `role` from the User whose id is [`user_id`](`ID`)
    ///
    /// returns true if successful
    pub fn unassign(db: &mut Connection, user_id: ID, role: &str) -> Result<bool> {
        let unassigned = UserRole::delete(db, user_id, role.to_string());

        Ok(unassigned.is_ok())
    }

    /// unassigns every role in `roles` from the User whose id is [`user_id`](`ID`)
    ///
    /// returns true if successful
    pub fn unassign_many(db: &mut Connection, user_id: ID, roles: Vec<String>) -> Result<bool> {
        let unassigned = UserRole::delete_many(db, user_id, roles);

        Ok(unassigned.is_ok())
    }

    /// returns a vector containing every role assigned to the User whose id is [`user_id`](`ID`)
    pub fn fetch_all(db: &mut Connection, user_id: ID) -> Result<Vec<String>> {
        let roles = sql_query("SELECT role FROM user_roles WHERE user_id = $1");

        let roles = roles
            .bind::<Integer, _>(user_id)
            .get_results::<RoleQueryRow>(db)?;

        let roles = roles.into_iter().map(|r| r.role).collect();

        Ok(roles)
    }
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, QueryableByName, Clone)]
pub struct Permission {
    #[diesel(sql_type=Text)]
    /// the role this permission is coming from
    pub from_role: String,

    #[diesel(sql_type=Text)]
    /// the permission itself
    pub permission: String,
}

impl Hash for Permission {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.permission.as_str().hash(state);
    }
}

impl PartialEq for Permission {
    fn eq(&self, other: &Self) -> bool {
        self.permission.eq(&other.permission)
    }
}
impl Eq for Permission {}

impl Permission {
    // pub fn is_granted_to_user(db: &mut Connection, user_id: ID, permission: String) -> Result<bool> {
    //   let permissions = Permission::for_user(&db, user_id)?;
    //   let user_has_permission = permissions.iter().any(|perm| perm.permission == permission);

    //   Ok(user_has_permission)
    // }

    /// grants `permission` to the User whose id is [`user_id`](`ID`)
    ///
    /// returns true if successful
    pub fn grant_to_user(db: &mut Connection, user_id: ID, permission: &str) -> Result<bool> {
        let granted = UserPermission::create(
            db,
            &UserPermissionChangeset {
                permission: permission.to_string(),
                user_id,
            },
        );

        Ok(granted.is_ok())
    }

    /// grant `permission` to `role`
    ///
    /// returns true if successful
    pub fn grant_to_role(db: &mut Connection, role: &str, permission: &str) -> Result<bool> {
        let granted = RolePermission::create(
            db,
            &RolePermissionChangeset {
                permission: permission.to_string(),
                role: role.to_string(),
            },
        );

        Ok(granted.is_ok())
    }

    /// grants every permission in `permissions` to `role`
    ///
    /// returns true if successful
    pub fn grant_many_to_role(
        db: &mut Connection,
        role: String,
        permissions: Vec<String>,
    ) -> Result<bool> {
        let granted = RolePermission::create_many(
            db,
            permissions
                .into_iter()
                .map(|permission| RolePermissionChangeset {
                    permission,
                    role: role.clone(),
                })
                .collect::<Vec<_>>(),
        );

        Ok(granted.is_ok())
    }

    /// grants every permission in `permissions` to `role`
    ///
    /// returns true if successful
    pub fn grant_many_to_user(
        db: &mut Connection,
        user_id: i32,
        permissions: Vec<String>,
    ) -> Result<bool> {
        let granted = UserPermission::create_many(
            db,
            permissions
                .into_iter()
                .map(|permission| UserPermissionChangeset {
                    permission,
                    user_id,
                })
                .collect::<Vec<_>>(),
        );

        Ok(granted.is_ok())
    }

    /// revokes `permission` from the User whose id is [`user_id`](`ID`)
    ///
    /// returns true if successful
    pub fn revoke_from_user(db: &mut Connection, user_id: ID, permission: &str) -> Result<bool> {
        let deleted = UserPermission::delete(db, user_id, permission.to_string());

        Ok(deleted.is_ok())
    }

    /// revokes `permission` from `role`
    ///
    /// returns true if successful
    pub fn revoke_from_role(db: &mut Connection, role: String, permission: String) -> Result<bool> {
        let deleted = RolePermission::delete(db, role, permission);

        Ok(deleted.is_ok())
    }

    /// revokes every permission in `permissions` from the User whose id is [`user_id`](`ID`)
    ///
    /// returns true if successful
    pub fn revoke_many_from_user(
        db: &mut Connection,
        user_id: ID,
        permissions: Vec<String>,
    ) -> Result<bool> {
        let deleted = UserPermission::delete_many(db, user_id, permissions);

        Ok(deleted.is_ok())
    }

    /// revokes every permission in `permissions` from `role`
    ///
    /// returns true if successful
    pub fn revoke_many_from_role(
        db: &mut Connection,
        role: String,
        permissions: Vec<String>,
    ) -> Result<bool> {
        let deleted = RolePermission::delete_many(db, role, permissions);

        Ok(deleted.is_ok())
    }

    /// revokes every permission granted to `role`
    ///
    /// returns true if successful
    pub fn revoke_all_from_role(db: &mut Connection, role: &str) -> Result<bool> {
        let deleted = RolePermission::delete_all(db, role);

        Ok(deleted.is_ok())
    }

    /// revokes every permission granted to the User whose id is [`user_id`](`ID`)
    ///
    /// returns true if successful
    pub fn revoke_all_from_user(db: &mut Connection, user_id: i32) -> Result<bool> {
        let deleted = UserPermission::delete_all(db, user_id);

        Ok(deleted.is_ok())
    }

    /// returns every permission granted to the User whose id is [`user_id`](`ID`)
    pub fn fetch_all(db: &mut Connection, user_id: ID) -> Result<Vec<Permission>> {
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
      WHERE user_roles.user_id = $1
      "#,
        );

        let permissions = permissions
            .bind::<Integer, _>(user_id)
            .get_results::<Permission>(db)?;

        Ok(permissions)
    }
}
