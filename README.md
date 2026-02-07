# Auth Manager

Service d'authentification production-ready en Rust, conçu pour fonctionner en local et sur AWS Lambda.

**Architecture multi-crate** : Le projet inclut un crate API (`auth-manager-api`) WASM-compatible pour partager les types entre backend et frontend.

## Fonctionnalités

- ✅ Inscription et connexion utilisateur
- ✅ Authentification JWT (HS256)
- ✅ Tokens de rafraîchissement sécurisés (HttpOnly cookies)
- ✅ Hachage de mots de passe avec bcrypt
- ✅ Gestion des sessions et déconnexion
- ✅ Changement de mot de passe
- ✅ Validation des entrées
- ✅ Gestion des tentatives de connexion
- ✅ Support Docker et Docker Compose
- ✅ Déploiement AWS Lambda
- ✅ Base de données PostgreSQL avec Diesel ORM
- ✅ API types WASM-compatible pour intégration frontend

## Prérequis

- **Docker** et **Docker Compose** (recommandé)
- **Rust** 1.92+ (si exécution locale sans Docker)
- **PostgreSQL** 15+ (si exécution locale sans Docker)
- **Make** (pour les commandes simplifiées)

## Installation

### 1. Cloner le projet

```bash
git clone <repository-url>
cd auth-manager
```

### 2. Configuration

Copiez le fichier d'exemple et ajustez les variables d'environnement :

```bash
cp .env.example .env
```

Variables importantes :

```env
# Configuration du serveur
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=debug

# Base de données
DATABASE_URL=postgres://postgres:postgres@auth_db:5432/auth_db

# JWT
JWT_SECRET=votre_secret_jwt_ici

# CORS
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
```

### 3. Démarrer l'application

#### Avec Docker (recommandé)

```bash
# Démarrer les services (app + PostgreSQL)
make local

# Ou en arrière-plan
make local-detached
```

L'application sera accessible sur `http://localhost:3000`

#### Sans Docker

```bash
# Installer Diesel CLI
cargo install diesel_cli --no-default-features --features postgres

# Lancer les migrations
diesel migration run

# Compiler et lancer
cargo build --release
cargo run
```

## API Endpoints

### Santé

```http
GET /health
```

### Authentification

#### Inscription
```http
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "username": "username",
  "password": "SecurePass123!"
}
```

#### Connexion
```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

Réponse :
```json
{
  "access_token": "eyJhbGc...",
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "username",
    "created_at": "2024-01-01T00:00:00Z"
  },
  "expires_in": 3600
}
```

Le refresh token est automatiquement stocké dans un cookie HttpOnly.

#### Rafraîchir le token
```http
POST /auth/refresh
Cookie: refresh_token=<hash>
```

#### Déconnexion
```http
POST /auth/logout
Authorization: Bearer <access_token>
```

### Utilisateurs

#### Obtenir l'utilisateur courant
```http
GET /users/me
Authorization: Bearer <access_token>
```

#### Obtenir un utilisateur par ID
```http
GET /users/{id}
Authorization: Bearer <access_token>
```

#### Changer le mot de passe
```http
POST /users/{id}/change-password
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "old_password": "OldPass123!",
  "new_password": "NewPass456!"
}
```

#### Supprimer un utilisateur
```http
DELETE /users/{id}
Authorization: Bearer <access_token>
```

## Développement

### Commandes Make

```bash
# Développement
make local              # Démarrer l'environnement de développement
make stop               # Arrêter tous les conteneurs
make restart            # Redémarrer l'environnement

# Base de données
make migrate            # Appliquer les migrations
make revert             # Annuler la dernière migration
make db-shell           # Ouvrir un shell PostgreSQL
make db-reset           # Réinitialiser la base de données

# Tests
make test               # Lancer tous les tests
make test t=test_login  # Lancer un test spécifique
make test-watch         # Tests en mode watch

# Code Quality
make fmt                # Formater le code
make clippy             # Linter
make ci                 # Vérifications CI (format + lint + tests)

# API Crate
cargo test -p auth-manager-api                      # Tests du crate API
cargo build --manifest-path auth-manager-api/Cargo.toml \
  --target wasm32-unknown-unknown --release         # Build WASM

# Logs
make logs               # Suivre tous les logs
make logs-app           # Logs de l'application uniquement
make logs-db            # Logs de la base de données

# Shells
make shell              # Shell dans le conteneur app
make shell-test         # Shell dans le conteneur de tests
```

### Structure du projet

```
auth-manager/
├── Cargo.toml                  # Backend package
├── auth-manager-api/           # 🎯 API types crate (WASM-compatible)
│   ├── Cargo.toml              # Dépendances minimales
│   ├── README.md               # Documentation du crate
│   └── src/
│       ├── lib.rs              # Exports publics
│       ├── requests.rs         # DTOs de requête
│       ├── responses.rs        # DTOs de réponse
│       ├── error.rs            # Format d'erreur
│       └── result.rs           # Wrapper de réponse
├── src/                        # Backend code
│   ├── response.rs             # Wrapper Axum pour API types
│   ├── auth/                   # Module d'authentification
│   │   ├── jwt.rs              # Gestion JWT
│   │   ├── password.rs         # Hachage bcrypt
│   │   ├── services.rs         # Logique métier
│   │   └── extractors.rs       # Extracteurs Axum
│   ├── db/                     # Couche de persistance
│   │   ├── models/             # Modèles Diesel
│   │   ├── repositories/       # Accès base de données
│   │   ├── schema.rs           # Schéma généré
│   │   └── connection.rs       # Pool de connexions
│   ├── handlers/               # Gestionnaires HTTP
│   │   ├── auth.rs             # Routes d'auth
│   │   ├── user.rs             # Routes utilisateur
│   │   └── health.rs           # Santé
│   ├── app.rs                  # Configuration du routeur
│   ├── error.rs                # Types d'erreur
│   └── main.rs                 # Point d'entrée
├── migrations/                 # Migrations Diesel
├── docker/                     # Fichiers Docker
│   ├── Dockerfile              # Image de développement
│   ├── Dockerfile.lambda       # Image Lambda optimisée
│   ├── docker-compose.yml      # Stack de dev
│   └── docker-compose.test.yml # Stack de tests
├── postman/                    # Collection Postman
├── .env.example                # Variables d'environnement exemple
├── diesel.toml                 # Configuration Diesel
├── makefile                    # Commandes simplifiées
└── CLAUDE.md                   # Guide pour Claude Code
```

### Ajouter une migration

```bash
# Créer une nouvelle migration
diesel migration generate nom_migration

# Éditer les fichiers up.sql et down.sql dans migrations/

# Appliquer la migration
make migrate
```

### Tests

Les tests sont exécutés dans un environnement Docker isolé avec une base de données de test dédiée :

```bash
# Tous les tests
make test

# Test spécifique
make test t=test_register_success

# Avec sortie détaillée
make test t=test_name -- --nocapture
```

## Déploiement AWS Lambda

### 1. Premier déploiement (création de l'infrastructure)

```bash
# Crée le stack CloudFormation avec ECR, Lambda, API Gateway
make deploy-create-stack
```

### 2. Déploiements suivants

```bash
# Déploiement complet (build + push + update)
make deploy

# Mise à jour sans rebuild (si l'image existe déjà)
make deploy-only

# Voir les logs Lambda en temps réel
make deploy-logs

# Afficher le statut du stack
make deploy-status
```

### Configuration Lambda

Variables d'environnement requises dans la fonction Lambda :

```
DATABASE_URL=postgres://...
JWT_SECRET=...
CORS_ALLOWED_ORIGINS=https://yourdomain.com
BCRYPT_COST=12
RUST_LOG=info
```

## Technologies

### Backend (`auth-manager`)

- **[Rust](https://www.rust-lang.org/)** - Langage de programmation
- **[Axum](https://github.com/tokio-rs/axum)** - Framework web
- **[Tokio](https://tokio.rs/)** - Runtime asynchrone
- **[Diesel](https://diesel.rs/)** - ORM et query builder
- **[PostgreSQL](https://www.postgresql.org/)** - Base de données
- **[jsonwebtoken](https://github.com/Keats/jsonwebtoken)** - JWT
- **[bcrypt](https://github.com/Keats/rust-bcrypt)** - Hachage de mots de passe
- **[lambda_http](https://github.com/awslabs/aws-lambda-rust-runtime)** - Runtime Lambda
- **[Docker](https://www.docker.com/)** - Conteneurisation

### API Types (`auth-manager-api`)

- **[Serde](https://serde.rs/)** - Sérialisation/désérialisation
- **[UUID](https://github.com/uuid-rs/uuid)** - Identifiants uniques
- **[Chrono](https://github.com/chronotope/chrono)** - Gestion des dates
- **WASM-compatible** - Peut être compilé pour wasm32-unknown-unknown

## Architecture

Le projet suit une **architecture multi-crate en couches strictes** :

### Structure Multi-Crate

1. **`auth-manager-api`** (Crate WASM-compatible)
   - Types publics partagés entre backend et frontend
   - Dépendances minimales : serde, uuid, chrono uniquement
   - Peut être importé dans des applications WASM

2. **`auth-manager`** (Backend)
   - Serveur HTTP/Lambda avec Axum
   - Utilise `auth-manager-api` pour les types
   - Contient toute la logique métier et persistance

### Couches Backend

1. **Couche API Types** (`auth-manager-api` crate)
   - Request/Response DTOs
   - Format d'erreur
   - Wrapper de réponse générique

2. **Couche HTTP** (`src/handlers/`, `src/response.rs`)
   - Gestionnaires de routes minimalistes
   - Wrapper Axum pour les types API
   - Mapping d'erreurs vers HTTP

3. **Couche Service** (`src/auth/services.rs`)
   - Logique métier et validation
   - Orchestration JWT et mots de passe
   - Coordination entre repositories

4. **Couche Persistance** (`src/db/repositories/`)
   - Accès base de données exclusif
   - Queries Diesel isolées
   - Pool de connexions

### Séparation des Responsabilités

- Les **handlers** ne contiennent aucune logique métier
- Les **services** orchestrent toute la logique métier
- Les **repositories** gèrent exclusivement l'accès aux données
- Les **types API** sont indépendants du backend

## Intégration Frontend

Le crate `auth-manager-api` peut être utilisé dans des applications frontend Rust/WASM pour une communication type-safe avec l'API.

### Installation

Dans le `Cargo.toml` de votre frontend :

```toml
[dependencies]
auth-manager-api = { path = "../auth-manager/auth-manager-api" }
```

### Ce que le frontend reçoit

**Inclus** ✅ :
- Request DTOs : `RegisterRequest`, `LoginRequest`, `RefreshTokenRequest`, etc.
- Response DTOs : `UserResponse`, `LoginResponse`, `RefreshTokenResponse`, etc.
- Format d'erreur : `ErrorResponse`
- Dépendances légères : serde, uuid, chrono

**Exclu** ❌ :
- Axum (framework web)
- Diesel (ORM)
- Tokio (runtime async)
- Toute dépendance serveur

### Exemple d'utilisation

```rust
use auth_manager_api::{LoginRequest, LoginResponse};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn login(email: String, password: String) -> Result<JsValue, JsValue> {
    let request = LoginRequest { email, password };

    // Envoyer au backend via HTTP...
    let response: LoginResponse = fetch_json("/auth/login", request).await?;

    Ok(serde_wasm_bindgen::to_value(&response)?)
}
```

### Build WASM

```bash
# Installer la target WASM
rustup target add wasm32-unknown-unknown

# Vérifier la compatibilité
cd auth-manager-api
cargo build --target wasm32-unknown-unknown --release
```

## Sécurité

- Mots de passe hachés avec bcrypt (coût configurable)
- Tokens JWT signés et avec expiration
- Refresh tokens stockés sous forme de hash
- Cookies HttpOnly pour les refresh tokens
- CORS configurable
- Validation des entrées
- Pas de logs de données sensibles

## Licence

MIT

## Support

Pour toute question ou problème, ouvrez une issue sur le dépôt GitHub.
