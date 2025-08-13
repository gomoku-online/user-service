diesel::table! {
    users (user_id) {
        #[sql_name = "USER_ID"]
        user_id -> BigInt,

        #[sql_name = "USER_NICKNAME"]
        user_nickname -> Text,

        #[sql_name = "ACTIVE"]
        active -> Bool,

        #[sql_name = "CREATED_AT"]
        created_at -> Timestamp,

        #[sql_name = "UPDATED_AT"]
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_auths (user_auth_id) {
        #[sql_name = "USER_AUTH_ID"]
        user_auth_id -> BigInt,

        #[sql_name = "USER_ID"]
        user_id -> BigInt,

        #[sql_name = "AUTH_PROVIDER"]
        auth_provider -> Text,

        #[sql_name = "AUTH_ID"]
        auth_id -> Text,

        #[sql_name = "CREATED_AT"]
        created_at -> Timestamp,

        #[sql_name = "UPDATED_AT"]
        updated_at -> Timestamp,
    }
}

diesel::joinable!(user_auths -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    user_auths,
);
