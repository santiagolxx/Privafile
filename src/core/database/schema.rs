// @generated automatically by Diesel CLI.

diesel::table! {
    chunks (id) {
        id -> Text,
        file_id -> Text,
        chunk_index -> Integer,
        hash -> Text,
        size -> Integer,
        status -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    files (id) {
        id -> Text,
        mime -> Text,
        hash -> Text,
        owner_id -> Text,
        status -> Text,
        total_size -> Nullable<Integer>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    usuarios (id) {
        id -> Text,
        username -> Text,
        password -> Text,
        b64_pubkey -> Nullable<Text>,
    }
}

diesel::joinable!(chunks -> files (file_id));
diesel::joinable!(files -> usuarios (owner_id));

diesel::allow_tables_to_appear_in_same_query!(chunks, files, usuarios,);
