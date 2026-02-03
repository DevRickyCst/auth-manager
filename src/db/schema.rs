// @generated automatically by Diesel CLI.

diesel::table! {
    login_attempts (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        success -> Bool,
        attempted_at -> Timestamptz,
        user_agent -> Nullable<Text>,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_identities (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 20]
        provider -> Varchar,
        #[max_length = 255]
        provider_user_id -> Varchar,
        #[max_length = 255]
        email -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 100]
        username -> Varchar,
        #[max_length = 255]
        password_hash -> Nullable<Varchar>,
        email_verified -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_login_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(login_attempts -> users (user_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(user_identities -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    login_attempts,
    refresh_tokens,
    user_identities,
    users,
);
