table! {
    todos (id) {
        id -> Int4,
        text -> Text,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}
