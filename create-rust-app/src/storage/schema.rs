table! {
    attachment_blobs (id) {
        id -> Int4,
        key -> Text,
        file_name -> Nullable<Text>,
        content_type -> Nullable<Text>,
        byte_size -> Int8,
        checksum -> Text,
        service_name -> Text,
        created_at -> Timestamptz,
    }
}

table! {
    attachments (id) {
        id -> Int4,
        name -> Text,
        record_type -> Text,
        record_id -> Int4,
        blob_id -> Int4,
        created_at -> Timestamptz,
    }
}

joinable!(attachments -> attachment_blobs (blob_id));

allow_tables_to_appear_in_same_query!(
    attachment_blobs,
    attachments,
);
