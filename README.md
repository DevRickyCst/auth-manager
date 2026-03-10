# Auth Manager

Service d'authentification production-ready en Rust, conçu pour fonctionner nativement en local et déployé sur AWS Lambda.

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
- ✅ Déploiement AWS Lambda (image ECR)
- ✅ Base de données PostgreSQL avec Diesel ORM
- ✅ API types WASM-compatible pour intégration frontend

## Prérequis

- **Rust** 1.92+
- **Docker & Docker Compose** (PostgreSQL uniquement — géré à la racine du monorepo)
- **diesel_cli** — migrations base de données
- **cargo-watch** — hot-reload en développement
- **Make**

```bash
cargo install diesel_cli --no-default-features --features postgres
cargo install cargo-watch
```

## Installation

### 1. Configuration

```bash
cp .env.example .env
```

Variables importantes :

```env
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=debug

DATABASE_URL=postgres://postgres:postgres@localhost:5432/auth_db
TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5433/auth_test_db

JWT_SECRET=votre_secret_jwt_ici
FRONTEND_URL=http://localhost:8080
```

### 2. Démarrer les bases de données

Les bases PostgreSQL sont gérées par le `docker-compose.yml` à la **racine du monorepo** :

```bash
# Depuis la racine du monorepo
docker compose up -d

# Ou depuis auth-manager via make
make db-start
```

| Base | Port | Persistance |
|---|---|---|
| `postgres-dev` | `5432` | volume Docker |
| `postgres-test` | `5433` | tmpfs (RAM) |

### 3. Migrations

```bash
make migrate   # → diesel migration run
```

### 4. Lancer l'application

```bash
make local     # hot-reload → cargo watch -x run
# ou
cargo run
```

L'application est accessible sur `http://localhost:3000`.

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
# ── Bases de données ──────────────────────────────────────────
make db-start           # Démarrer postgres-dev + postgres-test (docker compose racine)
make db-stop            # Arrêter les bases
make db-logs            # Suivre les logs des bases
make db-shell           # Shell psql → localhost:5432
make db-shell-prod      # Shell psql → Neon (production)

# ── Application ───────────────────────────────────────────────
make local              # Hot-reload → cargo watch -x run
make run                # Lancer une fois → cargo run
make stop               # Arrêter les bases (alias db-stop)

# ── Migrations ────────────────────────────────────────────────
make migrate            # → diesel migration run
make revert             # → diesel migration revert
make db-reset           # Reset complet (SUPPRIME TOUTES LES DONNÉES)

# ── Tests ─────────────────────────────────────────────────────
make test               # → cargo test -- --test-threads=1
make test t=test_login  # Test spécifique
make test-watch         # Tests en mode watch

# ── Code Quality ──────────────────────────────────────────────
make check              # cargo check
make fmt                # cargo fmt
make clippy             # cargo clippy
make ci                 # format + lint + test

# ── Build ─────────────────────────────────────────────────────
make build              # cargo build --release

# ── API Crate ─────────────────────────────────────────────────
cargo test -p auth-manager-api
cargo build --manifest-path auth-manager-api/Cargo.toml \
  --target wasm32-unknown-unknown --release

# ── Déploiement Lambda ────────────────────────────────────────
make deploy-create-stack   # Créer l'infra (première fois)
make deploy                # Build image Docker → push ECR → update Lambda
make deploy-only           # Update Lambda sans rebuild
make deploy-logs           # Logs Lambda en temps réel
make deploy-status         # Statut du stack CloudFormation
```

### Structure du projet

```
auth-manager/
├── Cargo.toml                  # Package principal
├── auth-manager-api/           # 🎯 API types crate (WASM-compatible)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── requests.rs         # DTOs de requête
│       ├── responses.rs        # DTOs de réponse
│       ├── error.rs            # Format d'erreur
│       └── result.rs           # Wrapper de réponse
├── src/                        # Code backend
│   ├── response.rs             # Wrapper Axum pour API types
│   ├── auth/
│   │   ├── jwt.rs              # Gestion JWT
│   │   ├── password.rs         # Hachage bcrypt
│   │   ├── services.rs         # Logique métier
│   │   └── extractors.rs       # Extracteurs Axum
│   ├── db/
│   │   ├── models/             # Modèles Diesel
│   │   ├── repositories/       # Accès base de données
│   │   ├── schema.rs           # Schéma généré par Diesel
│   │   └── connection.rs       # Pool de connexions r2d2
│   ├── handlers/
│   │   ├── auth.rs
│   │   ├── user.rs
│   │   └── health.rs
│   ├── app.rs                  # Configuration du routeur
│   ├── error.rs                # Types d'erreur
│   └── main.rs                 # Point d'entrée (local + Lambda)
├── migrations/                 # Migrations Diesel
├── infra/                      # Infrastructure de déploiement
│   ├── template.yaml           # SAM template
│   ├── samconfig.toml
│   └── params/                 # Paramètres prod (non commités)
├── scripts/
│   └── neon-check.sh           # Vérification DB prod
├── postman/                    # Collection Postman
├── .env.example
├── diesel.toml
├── makefile
└── CLAUDE.md
```

### Ajouter une migration

```bash
diesel migration generate nom_migration
# Éditer migrations/.../up.sql et down.sql
make migrate
```

### Tests

Les tests utilisent la base **postgres-test** (`localhost:5433`, tmpfs) :

```bash
# S'assurer que les bases tournent
make db-start

# Tous les tests
make test

# Test spécifique
make test t=test_register_success

# Avec sortie détaillée
make test t=test_name -- --nocapture

# Mode watch
make test-watch
```

## Déploiement AWS Lambda

Le backend est déployé sur AWS Lambda via `cargo-lambda` + SAM (binaire natif zip sur `provided.al2023`).

### 1. Premier déploiement (création de l'infrastructure)

```bash
make deploy-create-stack
```

### 2. Déploiements suivants

```bash
make deploy          # build image → push ECR → update Lambda
make deploy-only     # update Lambda sans rebuild
make deploy-logs     # logs en temps réel
make deploy-status   # statut du stack
```

### Variables d'environnement Lambda

```
DATABASE_URL=postgres://...   # Neon PostgreSQL
JWT_SECRET=...
FRONTEND_URL=https://dofus-graal.eu
BCRYPT_COST=12
RUST_LOG=info
```

## Technologies

### Backend (`auth-manager`)

- **[Rust](https://www.rust-lang.org/)** - Langage
- **[Axum](https://github.com/tokio-rs/axum)** - Framework web
- **[Tokio](https://tokio.rs/)** - Runtime asynchrone
- **[Diesel](https://diesel.rs/)** - ORM et query builder
- **[PostgreSQL](https://www.postgresql.org/)** - Base de données
- **[jsonwebtoken](https://github.com/Keats/jsonwebtoken)** - JWT HS256
- **[bcrypt](https://github.com/Keats/rust-bcrypt)** - Hachage de mots de passe
- **[lambda_http](https://github.com/awslabs/aws-lambda-rust-runtime)** - Adapter Lambda

### API Types (`auth-manager-api`)

- **[Serde](https://serde.rs/)** - Sérialisation/désérialisation
- **[UUID](https://github.com/uuid-rs/uuid)** - Identifiants uniques
- **[Chrono](https://github.com/chronotope/chrono)** - Gestion des dates
- **WASM-compatible** — compilable pour `wasm32-unknown-unknown`

## Architecture

### Structure Multi-Crate

1. **`auth-manager-api`** — types partagés WASM-compatible (serde, uuid, chrono uniquement)
2. **`auth-manager`** — serveur HTTP/Lambda avec toute la logique métier

### Couches Backend

1. **API Types** (`auth-manager-api`) — DTOs Request/Response, format d'erreur
2. **HTTP** (`src/handlers/`, `src/response.rs`) — handlers minimalistes, pas de logique métier
3. **Service** (`src/auth/services.rs`) — toute la logique métier et authentification
4. **Persistance** (`src/db/repositories/`) — queries Diesel isolées, pool r2d2

## Intégration Frontend

```toml
# frontend Cargo.toml
[dependencies]
auth-manager-api = { path = "../auth-manager/auth-manager-api" }
```

Le frontend reçoit **uniquement** les DTOs et types légers — pas d'Axum, Diesel ou Tokio.

```bash
# Vérifier la compatibilité WASM
cargo build --manifest-path auth-manager-api/Cargo.toml \
  --target wasm32-unknown-unknown --release
```

## Sécurité

- Mots de passe hachés avec bcrypt (coût configurable)
- Tokens JWT signés avec expiration
- Refresh tokens stockés sous forme de hash
- Cookies HttpOnly pour les refresh tokens
- CORS configurable
- Validation des entrées
- Aucun log de données sensibles
