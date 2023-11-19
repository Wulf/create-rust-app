use std::collections::HashSet;

use crate::auth::{Permission, ID};

#[allow(clippy::module_name_repetitions)]
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
    #[must_use]
    pub fn has_permission(&self, permission: String) -> bool {
        self.permissions.contains(&Permission {
            permission,
            from_role: String::new(),
        })
    }

    /// does the user with the id [`self.user_id`](`ID`) have all of the given `perms`
    #[must_use]
    pub fn has_all_permissions(&self, perms: impl AsRef<[String]>) -> bool {
        perms
            .as_ref()
            .iter()
            .all(|p| self.has_permission(p.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have any of the given `perms`
    #[must_use]
    pub fn has_any_permission(&self, perms: impl AsRef<[String]>) -> bool {
        perms
            .as_ref()
            .iter()
            .any(|p| self.has_permission(p.to_string()))
    }

    /// does the user with the id [`self.user_id`](`ID`) have the given `role`
    #[must_use]
    pub fn has_role(&self, role: impl AsRef<str>) -> bool {
        self.roles.contains(role.as_ref())
    }

    /// does the user with the id [`self.user_id`](`ID`) have all of the given `roles`
    #[must_use]
    pub fn has_all_roles(&self, roles: impl AsRef<[String]>) -> bool {
        roles.as_ref().iter().all(|r| self.has_role(r))
    }

    /// does the user with the id [`self.user_id`](`ID`) have any of the given `roles`
    pub fn has_any_roles(&self, roles: impl AsRef<[String]>) -> bool {
        roles.as_ref().iter().any(|r| self.has_role(r))
    }
}
