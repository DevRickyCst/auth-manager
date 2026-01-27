use axum::{http::StatusCode, Json};
use crate::dto::responses::ErrorResponse;

#[allow(dead_code)]
pub type ApiResponse<T> = Result<(StatusCode, Json<T>), (StatusCode, Json<ErrorResponse>)>;

#[allow(dead_code)]
pub fn error_response(status: StatusCode, error: &str, message: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: error.to_string(),
            message,
        }),
    )
}

#[allow(dead_code)]
pub fn success_response<T>(status: StatusCode, data: T) -> (StatusCode, Json<T>) {
    (status, Json(data))
}

#[allow(dead_code)]
pub fn handle_result<T, E: ToString>(
    result: Result<T, E>, 
    ok_status: StatusCode, 
    error_msg: &str
) -> ApiResponse<T> {
    match result {
        Ok(data) => Ok(success_response(ok_status, data)),
        Err(e) => Err(error_response(StatusCode::INTERNAL_SERVER_ERROR, error_msg, e.to_string())),
    }
}