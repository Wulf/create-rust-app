table! {
  role_permissions (role, permission) {
      role -> Text,
      permission -> Text,
      created_at -> Timestamp,
  }
}

table! {
  user_permissions (user_id, permission) {
      user_id -> Integer,
      permission -> Text,
      created_at -> Timestamp,
  }
}

table! {
  user_roles (user_id, role) {
      user_id -> Integer,
      role -> Text,
      created_at -> Timestamp,
  }
}

table! {
  user_sessions (id) {
      id -> Integer,
      user_id -> Integer,
      refresh_token -> Text,
      device -> Nullable<Text>,
      created_at -> Timestamp,
  }
}

table! {
  users (id) {
      id -> Integer,
      email -> Text,
      hash_password -> Text,
      activated -> Bool,
      created_at -> Timestamp,
  }
}

joinable!(user_permissions -> users (user_id));
joinable!(user_roles -> users (user_id));
joinable!(user_sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    role_permissions,
    user_permissions,
    user_roles,
    user_sessions,
    users,
);
