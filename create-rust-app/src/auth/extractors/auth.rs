use std::collections::HashSet;

use crate::auth::{Permission, ID};

#[derive(Debug, Clone)]
/// roles and permissions available to a User
///
/// use to control what users are and are not allowed to do
pub struct Auth {
    pub user_id: ID,
    pub roles: HashSet<String>,
    pub permissions: HashSet<Permission>,
}

impl Auth {
    /// does the user with the id [`self.user_id`](`ID`) have the given `permission`
    pub fn has_permission(&self, permission: String) -> bool {
        self.permissions.contains(&Permission {
            permission,
            from_role: String::new(),
        })
    }

    /// does the user with the id [`self.user_id`](`ID`) have all of the given `perms`
    pub fn has_all_permissions(&self, perms: Vec<String>) -> bool {
        perms.iter().all(|p| self.has_permission(p.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have any of the given `perms`
    pub fn has_any_permission(&self, perms: Vec<String>) -> bool {
        perms.iter().any(|p| self.has_permission(p.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have the given `role`
    pub fn has_role(&self, role: String) -> bool {
        self.roles.contains(&role)
    }

    /// does the user with the id [`self.user_id`](`ID`) have all of the given `roles`
    pub fn has_all_roles(&self, roles: Vec<String>) -> bool {
        roles.iter().all(|r| self.has_role(r.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have any of the given `roles`
    pub fn has_any_roles(&self, roles: Vec<String>) -> bool {
        roles.iter().any(|r| self.has_role(r.to_string()))
    }
}
