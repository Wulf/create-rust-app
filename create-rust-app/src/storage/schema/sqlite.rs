table! {
  attachment_blobs (id) {
      id -> Integer,
      key -> Text,
      file_name -> Text,
      content_type -> Nullable<Text>,
      byte_size -> BigInt,
      checksum -> Text,
      service_name -> Text,
      created_at -> Timestamp,
  }
}

table! {
  attachments (id) {
      id -> Integer,
      name -> Text,
      record_type -> Text,
      record_id -> Integer,
      blob_id -> Integer,
      created_at -> Timestamp,
  }
}

joinable!(attachments -> attachment_blobs (blob_id));

allow_tables_to_appear_in_same_query!(attachment_blobs, attachments,);
