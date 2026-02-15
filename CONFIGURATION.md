# Configuration Guide - Auto-Environment Detection

## 🎯 Vue d'ensemble

auth-manager détecte **automatiquement** l'environnement et charge la bonne configuration:

```
🖥️  Local (Docker)     → .env (DATABASE_URL locale)
☁️  AWS Lambda         → Variables d'environnement Lambda (DATABASE_URL Neon)
```

**Aucune configuration manuelle requise!** 🎉

---

## 🔍 Comment ça marche?

### Détection automatique

Le code (`src/config.rs`) détecte l'environnement dans cet ordre:

1. **Lambda AWS** → Vérifie si `AWS_LAMBDA_FUNCTION_NAME` existe
   ```rust
   if env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
       Environment::Production
   }
   ```

2. **Variable APP_ENV** → Vérifie `APP_ENV=production`
   ```rust
   match env::var("APP_ENV").as_deref() {
       Ok("production") | Ok("prod") => Environment::Production,
       _ => Environment::Development,
   }
   ```

3. **Défaut** → Développement local
   ```rust
   Environment::Development
   ```

### Chargement de la DATABASE_URL

**En développement (local):**
```bash
# Lit DATABASE_URL depuis .env
DATABASE_URL=postgres://postgres:postgres@auth_db:5432/auth_db
```

**En production (Lambda):**
```bash
# Lambda injecte automatiquement la variable
DATABASE_URL=postgresql://neondb_owner:xxx@ep-xxx.neon.tech/dofus-graal?sslmode=require
```

---

## 📂 Fichiers de configuration

### 1. `.env` (Développement local uniquement)

```bash
# Chargé automatiquement en mode développement
APP_ENV=development
DATABASE_URL=postgres://postgres:postgres@auth_db:5432/auth_db
JWT_SECRET=dev_secret_key_NOT_FOR_PRODUCTION
FRONTEND_URL=http://localhost:8080
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=debug
```

**Utilisation:**
```bash
# Docker Compose charge automatiquement .env
docker-compose up

# Ou export manuel
export $(cat .env | grep -v '^#' | xargs)
cargo run
```

### 2. `infra/params/prod.json` (Credentials production - NON COMMITÉ)

**⚠️ NOUVEAU WORKFLOW:** Les credentials production sont maintenant gérés via SAM parameters, pas via `.env.production`.

```json
{
  "DatabaseUrl": "postgresql://neondb_owner:xxx@ep-xxx.neon.tech/dofus-graal?sslmode=require",
  "JwtSecret": "your-strong-production-secret-32-chars-min",
  "FrontendUrl": "https://dofus-graal.eu"
}
```

**Utilisation:**
```bash
# Créer depuis le template
cp infra/params/prod.json.example infra/params/prod.json

# Éditer avec vos credentials
vim infra/params/prod.json

# Déployer (lit automatiquement params/prod.json)
make deploy

# Migrations production
export DATABASE_URL=$(jq -r '.DatabaseUrl' infra/params/prod.json)
diesel migration run
```

⚠️ **IMPORTANT:**
- Ce fichier est dans `.gitignore` (JAMAIS commité!)
- Remplace l'ancien `.env.production` (déprécié)
- Utilisé par `scripts/deploy-lambda.sh` pour injecter les variables dans Lambda

### 3. Lambda Environment Variables (Production AWS)

Configurées automatiquement via SAM + `infra/params/prod.json`:

```yaml
# infra/template.yaml
Parameters:
  DatabaseUrl:    # Injecté depuis params/prod.json
    Type: String
    NoEcho: true
  JwtSecret:      # Injecté depuis params/prod.json
    Type: String
    NoEcho: true
  FrontendUrl:    # Injecté depuis params/prod.json
    Type: String

Resources:
  AuthManagerFunction:
    Environment:
      Variables:
        APP_ENV: production
        DATABASE_URL: !Ref DatabaseUrl
        JWT_SECRET: !Ref JwtSecret
        FRONTEND_URL: !Ref FrontendUrl
        RUST_LOG: info
```

**Déploiement:** `make deploy` lit automatiquement `infra/params/prod.json` et injecte les valeurs.

---

## 🚀 Workflows

### Développement local (Docker)

```bash
cd /Users/aymericcreusot/Documents/Aymeric/github/dofus-graal/auth-manager

# Démarrer PostgreSQL + app
docker-compose up

# Les variables de .env sont chargées automatiquement
# → DATABASE_URL pointe vers PostgreSQL local
# → L'app détecte Environment::Development
```

**Logs attendus:**
```
🚀 Starting auth-manager...
🌍 Environment detected: DEVELOPMENT
📦 Development mode: loading .env file
✅ Configuration loaded successfully
   Database: postgres://***:***@auth_db:5432/auth_db
   Frontend: http://localhost:8080
   Server: 0.0.0.0:3000
✅ Database connection pool initialized
💻 Running in local HTTP server mode
🌐 Server listening on http://0.0.0.0:3000
```

### Production (AWS Lambda)

```bash
# Déployer via SAM
make deploy

# Lambda injecte automatiquement:
# - AWS_LAMBDA_FUNCTION_NAME (détecté → Production)
# - DATABASE_URL (Neon)
# - JWT_SECRET
# - etc.

# L'app détecte automatiquement Environment::Production
```

**Logs attendus:**
```
🚀 Starting auth-manager...
🌍 Environment detected: PRODUCTION
📦 Production mode: using injected environment variables
✅ Configuration loaded successfully
   Database: postgresql://***:***@ep-xxx.neon.tech/dofus-graal
   Frontend: https://dofus-graal.eu
✅ Database connection pool initialized
☁️  Running in AWS Lambda mode
```

### Test config prod localement (sans Lambda)

```bash
# Charger credentials depuis params/prod.json
export DATABASE_URL=$(jq -r '.DatabaseUrl' infra/params/prod.json)
export JWT_SECRET=$(jq -r '.JwtSecret' infra/params/prod.json)
export FRONTEND_URL=$(jq -r '.FrontendUrl' infra/params/prod.json)
export APP_ENV=production

# Lancer l'app
cargo run

# L'app détecte APP_ENV=production
# → Utilise DATABASE_URL Neon
# → Mode production activé
```

---

## 🔧 Variables d'environnement

### Obligatoires

| Variable | Développement | Production | Description |
|----------|---------------|------------|-------------|
| `DATABASE_URL` | ✅ Local PostgreSQL | ✅ Neon PostgreSQL | Connection string DB |
| `JWT_SECRET` | ⚠️ Dev default | ✅ Requis (32+ chars) | Secret pour signer les JWT |

### Optionnelles (avec fallbacks)

| Variable | Défaut Dev | Défaut Prod | Description |
|----------|------------|-------------|-------------|
| `APP_ENV` | `development` | `production` | Force l'environnement |
| `FRONTEND_URL` | `http://localhost:8080` | `https://dofus-graal.eu` | URL frontend pour CORS |
| `SERVER_HOST` | `0.0.0.0` | N/A (Lambda) | Host serveur local |
| `SERVER_PORT` | `3000` | N/A (Lambda) | Port serveur local |
| `JWT_EXPIRATION` | `7d` | `7d` | Durée de vie des tokens |
| `RUST_LOG` | `debug` | `info` | Niveau de logging |

---

## 🛡️ Sécurité

### ✅ Bonnes pratiques

1. **Ne jamais commiter `infra/params/prod.json`**
   ```bash
   # Vérifié dans .gitignore
   git status --ignored | grep params/prod.json
   # → doit apparaître comme ignored
   ```

2. **Utiliser des secrets forts en production**
   ```bash
   # Générer un JWT_SECRET de 32+ caractères
   openssl rand -base64 32
   ```

3. **SSL obligatoire en production**
   ```bash
   # Toujours ajouter ?sslmode=require pour Neon
   DATABASE_URL=postgresql://...?sslmode=require
   ```

4. **Pas de credentials en clair dans les logs**
   ```rust
   // Le code masque automatiquement les credentials
   tracing::debug!("Database: {}", Config::mask_credentials(&database_url));
   // → postgres://***:***@localhost:5432/db
   ```

### ❌ À ne JAMAIS faire

- ❌ Commiter `.env` ou `infra/params/prod.json` dans Git
- ❌ Utiliser le même JWT_SECRET en dev et prod
- ❌ Désactiver SSL en production (`sslmode=disable`)
- ❌ Partager les credentials publiquement (refaire si fait!)

---

## 🐛 Troubleshooting

### Erreur: "DATABASE_URL not set"

**Cause:** Variable manquante

**Solution:**
```bash
# Développement
echo "DATABASE_URL=postgres://postgres:postgres@auth_db:5432/auth_db" >> .env

# Production (Lambda)
# → Configurer dans AWS Console → Lambda → Environment variables
```

### Erreur: "Failed to initialize database connection pool"

**Cause:** Database inaccessible ou credentials invalides

**Solution:**
```bash
# Vérifier la connexion
psql $DATABASE_URL -c "SELECT 1;"

# Vérifier les logs
docker-compose logs auth_db  # Local
make deploy-logs             # Lambda
```

### L'app utilise la mauvaise base de données

**Cause:** Environment mal détecté

**Solution:**
```bash
# Vérifier quelle config est chargée
RUST_LOG=debug cargo run

# Logs attendus:
# 🌍 Environment detected: DEVELOPMENT ou PRODUCTION
# 📦 Development mode: loading .env file
#     ou
# 📦 Production mode: using injected environment variables
```

**Forcer l'environnement:**
```bash
# Forcer dev
export APP_ENV=development

# Forcer prod
export APP_ENV=production
```

---

## 📊 Récapitulatif

### En Local (Docker)

```
.env → DATABASE_URL (local) → PostgreSQL Docker → auth_db
```

### En Production (Lambda)

```
Lambda Env Vars → DATABASE_URL (Neon) → Neon PostgreSQL Cloud → dofus-graal
```

### Test Prod en Local

```
params/prod.json → DATABASE_URL (Neon) → Neon PostgreSQL Cloud → dofus-graal
```

---

## ✅ Checklist setup

- [ ] `.env` existe avec DATABASE_URL local
- [ ] `infra/params/prod.json` créé avec credentials Neon
- [ ] `infra/params/prod.json` dans `.gitignore` ✅
- [ ] Password Neon régénéré (si exposé)
- [ ] JWT_SECRET généré (32+ caractères)
- [ ] Test local: `docker-compose up` → connecte à PostgreSQL local
- [ ] Test prod local: `export DATABASE_URL=$(jq -r '.DatabaseUrl' infra/params/prod.json) && cargo run`
- [ ] Lambda déployée: `make deploy` → injecte params/prod.json automatiquement

---

**🎉 Configuration automatique prête!**

Plus besoin de changer le code ou les configs manuellement.
Le système détecte tout automatiquement! 🚀
