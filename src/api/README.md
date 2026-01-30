# Module API

Ce module contient tous les types publics exposés par l'API REST. Ces types sont conçus pour être facilement réutilisables dans une bibliothèque cliente ou un SDK.

## Organisation

```
api/
├── mod.rs          # Re-exports publics
├── requests.rs     # Types de requêtes (Request DTOs)
├── responses.rs    # Types de réponses (Response DTOs)
├── error.rs        # Type d'erreur publique (ErrorResponse)
└── result.rs       # Wrapper de réponse générique (AppResponse<T>)
```

## Types de requêtes (requests.rs)

Types utilisés pour les requêtes entrantes:
- `RegisterRequest` - Inscription d'un nouvel utilisateur
- `LoginRequest` - Connexion d'un utilisateur
- `RefreshTokenRequest` - Rafraîchissement des tokens
- `ChangePasswordRequest` - Changement de mot de passe

## Types de réponses (responses.rs)

Types utilisés pour les réponses de l'API:
- `UserResponse` - Informations d'un utilisateur
- `LoginResponse` - Réponse de connexion complète (interne)
- `PublicLoginResponse` - Réponse de connexion publique (sans refresh token)
- `RefreshTokenResponse` - Réponse de rafraîchissement de token

## Gestion des erreurs (error.rs)

- `ErrorResponse` - Format standard des erreurs retournées par l'API

Structure:
```json
{
  "error": "ERROR_CODE",
  "message": "Human readable message",
  "details": "Optional details"
}
```

## Wrapper de réponse (result.rs)

- `AppResponse<T>` - Wrapper générique pour toutes les réponses des handlers
- `AppResult<T>` - Type alias pour `Result<AppResponse<T>, AppError>`

### Utilisation de AppResponse

```rust
use crate::api::AppResponse;

// Réponse 200 OK
pub async fn get_user() -> Result<AppResponse<UserResponse>, AppError> {
    let user = service.get_user()?;
    Ok(AppResponse::ok(user))
}

// Réponse 201 Created
pub async fn create_user() -> Result<AppResponse<UserResponse>, AppError> {
    let user = service.create_user()?;
    Ok(AppResponse::created(user))
}

// Réponse 204 No Content
pub async fn delete_user() -> Result<AppResponse<()>, AppError> {
    service.delete_user()?;
    Ok(AppResponse::no_content())
}

// Réponse avec headers personnalisés
pub async fn login() -> Result<AppResponse<LoginResponse>, AppError> {
    let response = service.login()?;
    let mut headers = HeaderMap::new();
    headers.insert("Set-Cookie", cookie_value);
    Ok(AppResponse::ok(response).with_headers(headers))
}
```

## Utilisation dans une bibliothèque cliente

Ce module peut être facilement extrait pour créer un SDK client:

```rust
// Dans un projet client
use auth_manager_api::{
    LoginRequest,
    LoginResponse,
    UserResponse,
    ErrorResponse
};

async fn login(client: &Client, credentials: LoginRequest) -> Result<LoginResponse, ErrorResponse> {
    // Implémentation du client
}
```

## Avantages de cette organisation

1. **Séparation claire** - Tous les types publics sont dans un seul module
2. **Facilité de réutilisation** - Peut être extrait en crate séparé
3. **Cohérence** - Un seul endroit pour définir le contrat de l'API
4. **Maintenabilité** - Facile de trouver et modifier les types
5. **Documentation** - Tous les types exposés sont documentés ensemble
