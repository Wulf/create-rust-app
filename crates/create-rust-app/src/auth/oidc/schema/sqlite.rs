table! {
  users (id) {
      id -> Int4,
  }
}

table! {
  user_oauth2_links (id) {
      id -> Int4,
      provider -> Text,
      csrf_token -> Text,
      nonce -> Text,
      pkce_secret -> Text,
      refresh_token -> Nullable<Text>,
      access_token -> Nullable<Text>,
      subject_id -> Nullable<Text>,
      user_id -> Nullable<Int4>,
      created_at -> Timestamptz,
    }
}

joinable!(user_oauth2_links -> users (user_id));

allow_tables_to_appear_in_same_query!(users, user_oauth2_links);
