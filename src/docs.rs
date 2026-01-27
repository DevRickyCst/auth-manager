use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::health,
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::refresh_token,
        crate::handlers::auth::logout,
        crate::handlers::user::get_current_user,
        crate::handlers::user::get_user_by_id,
        crate::handlers::user::delete_user,
        crate::handlers::user::change_password,
    ),
    components(schemas(
        crate::dto::requests::RegisterRequest,
        crate::dto::requests::LoginRequest,
        crate::dto::requests::ChangePasswordRequest,
        crate::dto::responses::UserResponse,
        crate::dto::responses::PublicLoginResponse,
        crate::dto::responses::RefreshTokenResponse,
        crate::error::ErrorResponse,
    )),
    tags(
        (name = "Health"),
        (name = "Auth"),
        (name = "Users"),
    )
)]
pub struct ApiDoc;
