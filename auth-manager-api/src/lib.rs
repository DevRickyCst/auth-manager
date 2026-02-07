//! # auth-manager-api
//!
//! Shared API types for the auth-manager service.
//! This crate is designed to be WASM-compatible and can be used in both
//! backend (Rust) and frontend (WASM/TypeScript via wasm-bindgen) applications.
//!
//! ## Features
//!
//! - Request DTOs (RegisterRequest, LoginRequest, etc.)
//! - Response DTOs (UserResponse, LoginResponse, etc.)
//! - Error response format (ErrorResponse)
//! - Generic response wrapper (AppResponse)
//!
//! ## Example
//!
//! ```rust
//! use auth_manager_api::{LoginRequest, LoginResponse};
//!
//! let request = LoginRequest {
//!     email: "user@example.com".to_string(),
//!     password: "password123".to_string(),
//! };
//! ```

pub mod error;
pub mod requests;
pub mod responses;
pub mod result;

// Re-exports for convenient access
pub use error::ErrorResponse;
pub use requests::*;
pub use responses::*;
pub use result::{AppResponse, StatusCode};
