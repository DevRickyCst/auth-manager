# auth-manager-api

Shared API types for the auth-manager authentication service. This crate is designed to be **WASM-compatible** and can be used in both backend (Rust/Axum) and frontend (Rust/WASM) applications.

## Features

- ✅ **WASM-compatible** - No platform-specific dependencies
- ✅ **Type-safe API communication** - Share types between client and server
- ✅ **Minimal dependencies** - Only serde, uuid, chrono
- ✅ **Serde support** - All types are serializable/deserializable

## Usage

### In Backend (auth-manager)

```rust
use auth_manager_api::{LoginRequest, UserResponse, AppResponse};

// Parse request
let request: LoginRequest = /* ... */;

// Create response
let user = UserResponse { /* ... */ };
AppResponse::ok(user)
```

### In Frontend (WASM)

Add to `Cargo.toml`:

```toml
[dependencies]
auth-manager-api = { path = "../auth-manager/auth-manager-api" }
serde-wasm-bindgen = "0.6"
wasm-bindgen-futures = "0.4"
```

Example usage:

```rust
use auth_manager_api::{LoginRequest, LoginResponse};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn login(email: String, password: String) -> Result<JsValue, JsValue> {
    let request = LoginRequest { email, password };

    // Send to backend...
    let response: LoginResponse = /* ... */;

    Ok(serde_wasm_bindgen::to_value(&response)?)
}
```

## Types

### Requests

- `RegisterRequest` - User registration
- `LoginRequest` - User login
- `RefreshTokenRequest` - JWT refresh
- `ChangePasswordRequest` - Password change

### Responses

- `UserResponse` - User information
- `LoginResponse` - Login result with tokens
- `PublicLoginResponse` - Login result without refresh token
- `RefreshTokenResponse` - New access token
- `ErrorResponse` - Error details

### Generic Response

- `AppResponse<T>` - Generic wrapper for API responses

## Dependencies

- `serde` - Serialization/deserialization
- `serde_json` - JSON support
- `uuid` - UUID identifiers
- `chrono` - DateTime handling

All dependencies are WASM-compatible.

## Building for WASM

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build for WASM
cargo build --target wasm32-unknown-unknown --release
```

## License

MIT
