use auth_manager_api::{AppResponse as ApiResponse, StatusCode as ApiStatusCode};
use axum::{
    Json,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Backend wrapper for auth-manager-api's AppResponse that adds Axum integration.
///
/// This type wraps the WASM-compatible ApiResponse and provides:
/// - Axum's IntoResponse trait implementation
/// - HTTP header support
/// - Status code conversion
///
/// # Examples
///
/// ```rust
/// use crate::response::AppResponse;
///
/// // Simple response
/// AppResponse::ok(user_data)
///
/// // Response with headers
/// let mut headers = HeaderMap::new();
/// headers.insert("X-Custom", "value".parse().unwrap());
/// AppResponse::ok(data).with_headers(headers)
///
/// // Created response
/// AppResponse::created(new_resource)
///
/// // No content
/// AppResponse::no_content()
/// ```
pub struct AppResponse<T> {
    inner: ApiResponse<T>,
    headers: Option<HeaderMap>,
}

impl<T> AppResponse<T>
where
    T: Serialize,
{
    /// Creates a new response wrapping the API response
    pub fn new(inner: ApiResponse<T>) -> Self {
        Self {
            inner,
            headers: None,
        }
    }

    /// Adds HTTP headers to the response
    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    // === Convenience constructors ===

    /// 200 OK with data
    pub fn ok(data: T) -> Self {
        Self::new(ApiResponse::ok(data))
    }

    /// 201 Created with data
    pub fn created(data: T) -> Self {
        Self::new(ApiResponse::created(data))
    }

    /// 202 Accepted with data
    #[allow(dead_code)]
    pub fn accepted(data: T) -> Self {
        Self::new(ApiResponse::accepted(data))
    }
}

impl AppResponse<()> {
    /// 204 No Content
    pub fn no_content() -> Self {
        Self::new(ApiResponse::no_content())
    }
}

/// Converts API StatusCode to Axum's StatusCode
fn convert_status(api_status: ApiStatusCode) -> StatusCode {
    match api_status {
        ApiStatusCode::Ok => StatusCode::OK,
        ApiStatusCode::Created => StatusCode::CREATED,
        ApiStatusCode::Accepted => StatusCode::ACCEPTED,
        ApiStatusCode::NoContent => StatusCode::NO_CONTENT,
        ApiStatusCode::BadRequest => StatusCode::BAD_REQUEST,
        ApiStatusCode::Unauthorized => StatusCode::UNAUTHORIZED,
        ApiStatusCode::Forbidden => StatusCode::FORBIDDEN,
        ApiStatusCode::NotFound => StatusCode::NOT_FOUND,
        ApiStatusCode::Conflict => StatusCode::CONFLICT,
        ApiStatusCode::UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
        ApiStatusCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Implements Axum's IntoResponse trait for our wrapper
impl<T> IntoResponse for AppResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status = convert_status(self.inner.status);

        let mut response = match self.inner.data {
            Some(data) => (status, Json(data)).into_response(),
            None => status.into_response(),
        };

        if let Some(headers) = self.headers {
            response.headers_mut().extend(headers);
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        message: String,
    }

    #[test]
    fn test_ok_response() {
        let data = TestData {
            message: "success".to_string(),
        };
        let response = AppResponse::ok(data);
        assert_eq!(response.inner.status, ApiStatusCode::Ok);
    }

    #[test]
    fn test_created_response() {
        let data = TestData {
            message: "created".to_string(),
        };
        let response = AppResponse::created(data);
        assert_eq!(response.inner.status, ApiStatusCode::Created);
    }

    #[test]
    fn test_no_content_response() {
        let response = AppResponse::no_content();
        assert_eq!(response.inner.status, ApiStatusCode::NoContent);
        assert!(response.inner.data.is_none());
    }

    #[test]
    fn test_response_with_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Custom-Header", "value".parse().unwrap());

        let data = TestData {
            message: "with headers".to_string(),
        };
        let response = AppResponse::ok(data).with_headers(headers);
        assert!(response.headers.is_some());
    }

    #[test]
    fn test_status_conversion() {
        assert_eq!(convert_status(ApiStatusCode::Ok), StatusCode::OK);
        assert_eq!(convert_status(ApiStatusCode::Created), StatusCode::CREATED);
        assert_eq!(
            convert_status(ApiStatusCode::BadRequest),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            convert_status(ApiStatusCode::Unauthorized),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            convert_status(ApiStatusCode::InternalServerError),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
