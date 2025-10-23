// @generated automatically by Diesel CLI.

diesel::table! {
    files (id) {
        id -> Text,
        mime -> Text,
        hash -> Text,
        owner_id -> Text,
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

diesel::allow_tables_to_appear_in_same_query!(files, usuarios,);
