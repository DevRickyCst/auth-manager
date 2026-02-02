# Auth Manager

Service d'authentification production-ready en Rust, conçu pour fonctionner en local et sur AWS Lambda.

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
SERVER_PORT=8080
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

L'application sera accessible sur `http://localhost:8080`

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
├── src/
│   ├── api/                    # Types publics de l'API
│   │   ├── requests.rs         # DTOs de requête
│   │   ├── responses.rs        # DTOs de réponse
│   │   ├── error.rs            # Format d'erreur
│   │   └── result.rs           # Wrapper de réponse
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
├── Cargo.toml                  # Dépendances Rust
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

### 1. Construire le package Lambda

```bash
make build-lambda
```

Cela crée un binaire statique optimisé dans `bin/lambda.zip`.

### 2. Déployer sur AWS

```bash
# Upload vers S3
make push-s3

# Mettre à jour la fonction Lambda
make update-lambda

# Ou tout en une commande
make deploy-lambda
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

- **[Rust](https://www.rust-lang.org/)** - Langage de programmation
- **[Axum](https://github.com/tokio-rs/axum)** - Framework web
- **[Tokio](https://tokio.rs/)** - Runtime asynchrone
- **[Diesel](https://diesel.rs/)** - ORM et query builder
- **[PostgreSQL](https://www.postgresql.org/)** - Base de données
- **[jsonwebtoken](https://github.com/Keats/jsonwebtoken)** - JWT
- **[bcrypt](https://github.com/Keats/rust-bcrypt)** - Hachage de mots de passe
- **[lambda_http](https://github.com/awslabs/aws-lambda-rust-runtime)** - Runtime Lambda
- **[Docker](https://www.docker.com/)** - Conteneurisation

## Architecture

Le projet suit une architecture en couches stricte :

1. **Couche HTTP** (`handlers/`) - Gestionnaires de routes minimalistes
2. **Couche Service** (`auth/services.rs`) - Logique métier
3. **Couche Persistance** (`db/repositories/`) - Accès base de données

Les responsabilités sont clairement séparées :
- Les handlers ne contiennent aucune logique métier
- Les services orchestrent la logique métier
- Les repositories gèrent exclusivement l'accès aux données

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
