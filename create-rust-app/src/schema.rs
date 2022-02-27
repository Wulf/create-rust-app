// TODO: split this into a schema file for each plugin individually
// in this case, this should be schema_auth.rs

#[cfg(feature = "plugin_auth")]
table! {
  role_permissions (role) {
      role -> Text,
      permission -> Text,
      created_at -> Timestamptz,
  }
}

#[cfg(feature = "plugin_auth")]
table! {
  user_permissions (user_id, permission) {
      user_id -> Int4,
      permission -> Text,
      created_at -> Timestamptz,
  }
}

#[cfg(feature = "plugin_auth")]
table! {
  user_roles (user_id, role) {
      user_id -> Int4,
      role -> Text,
      created_at -> Timestamptz,
  }
}

#[cfg(feature = "plugin_auth")]
table! {
  user_sessions (id) {
      id -> Int4,
      user_id -> Int4,
      refresh_token -> Text,
      device -> Nullable<Text>,
      created_at -> Timestamptz,
      updated_at -> Timestamptz,
  }
}

#[cfg(feature = "plugin_auth")]
table! {
  users (id) {
      id -> Int4,
      email -> Text,
      hash_password -> Text,
      activated -> Bool,
      created_at -> Timestamptz,
      updated_at -> Timestamptz,
  }
}

#[cfg(feature = "plugin_auth")]
joinable!(user_permissions -> users (user_id));
#[cfg(feature = "plugin_auth")]
joinable!(user_roles -> users (user_id));
#[cfg(feature = "plugin_auth")]
joinable!(user_sessions -> users (user_id));

#[cfg(feature = "plugin_auth")]
allow_tables_to_appear_in_same_query!(
    role_permissions,
    user_permissions,
    user_roles,
    user_sessions,
    users,
);
