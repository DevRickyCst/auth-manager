# Infrastructure AWS - Auth Manager

Infrastructure-as-code pour déployer Auth Manager sur AWS Lambda avec SAM (Serverless Application Model).

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Internet                             │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
         ┌─────────────────────────┐
         │   API Gateway HTTP API   │
         │   Endpoints publics      │
         └────────────┬─────────────┘
                      │
                      ▼
         ┌─────────────────────────┐
         │   Lambda Function        │
         │   (Container Image)      │
         │   - Rust/Axum            │
         │   - JWT Auth             │
         └────────────┬─────────────┘
                      │
                      ▼
         ┌─────────────────────────┐
         │   PostgreSQL Database    │
         │   (RDS ou externe)       │
         └──────────────────────────┘
```

## Composants

### 1. ECR Repository (`auth-manager-prod`)
- Stocke les images Docker de l'application
- Scan automatique des vulnérabilités
- Politique de rétention: garde les 10 dernières images

### 2. Lambda Function (`auth-manager-prod`)
- Runtime: Image Docker (Rust)
- Mémoire: 1024 MB (configurable)
- Timeout: 30 secondes (configurable)
- Variables d'environnement sécurisées
- X-Ray tracing activé

### 3. API Gateway HTTP API
- Point d'entrée HTTP pour toutes les requêtes
- CORS configuré
- Intégration Lambda Proxy (mode `$default`)
- Logs CloudWatch activés

### 4. CloudWatch
- Log Groups pour Lambda et API Gateway
- Rétention: 30 jours
- Alarmes pour erreurs et throttling

### 5. IAM
- Rôle d'exécution Lambda avec permissions minimales
- Accès CloudWatch Logs
- Accès X-Ray

## Fichiers

```
infra/
├── template.yaml      # Template SAM (infrastructure)
├── samconfig.toml     # Configuration SAM (paramètres)
└── README.md          # Ce fichier
```

## Prérequis

### 1. Outils nécessaires

```bash
# AWS CLI
brew install awscli  # macOS
# ou
pip install awscli

# SAM CLI
brew install aws-sam-cli  # macOS
# ou
pip install aws-sam-cli

# Docker
# Télécharger depuis https://www.docker.com/
```

### 2. Configuration AWS

```bash
# Configurer AWS CLI
aws configure

# Entrer:
# - AWS Access Key ID
# - AWS Secret Access Key
# - Default region (ex: us-east-1)
# - Default output format (json)

# Vérifier la configuration
aws sts get-caller-identity
```

### 3. Base de données PostgreSQL

Vous aurez besoin d'une base de données PostgreSQL accessible:
- **RDS PostgreSQL** (recommandé pour production)
- **Aurora Serverless PostgreSQL** (recommandé pour coût optimisé)
- Base externe (développement uniquement)

## Configuration

### Étape 1: Éditer les paramètres

Éditez `infra/samconfig.toml` :

```toml
[default.deploy.parameters]
parameter_overrides = [
    "DatabaseUrl=postgres://user:password@host:5432/database",
    "JwtSecret=VOTRE_SECRET_JWT_ICI",
    "CorsAllowedOrigins=https://votre-domaine.com",
    "LambdaMemorySize=1024",
    "LambdaTimeout=30"
]
```

**Paramètres importants:**

| Paramètre | Description | Exemple |
|-----------|-------------|---------|
| `DatabaseUrl` | Connection string PostgreSQL | `postgres://user:pass@host:5432/db` |
| `JwtSecret` | Secret pour signer les JWT (min 32 caractères) | `votre-secret-securise-ici` |
| `CorsAllowedOrigins` | Origines CORS autorisées | `https://app.com,https://www.app.com` |
| `LambdaMemorySize` | Mémoire Lambda en MB | `1024` |
| `LambdaTimeout` | Timeout Lambda en secondes | `30` |

## Déploiement

### Premier déploiement complet

```bash
cd auth-manager

# 1. Créer l'infrastructure SAM (ECR, Lambda, API Gateway)
./scripts/deploy-lambda.sh --create-stack

# 2. Builder et déployer l'application
./scripts/deploy-lambda.sh
```

C'est tout ! Le script va:
1. ✅ Créer le repository ECR
2. ✅ Créer la fonction Lambda
3. ✅ Créer l'API Gateway
4. ✅ Builder l'image Docker
5. ✅ Pousser l'image vers ECR
6. ✅ Mettre à jour Lambda avec l'image

### Déploiements suivants (mises à jour)

```bash
# Mettre à jour le code uniquement
./scripts/deploy-lambda.sh
```

### Options du script

```bash
# Aide
./scripts/deploy-lambda.sh --help

# Utiliser un profil AWS spécifique
./scripts/deploy-lambda.sh -p production

# Utiliser une région différente
./scripts/deploy-lambda.sh -r eu-west-1

# Skip le build Docker (utiliser l'image existante)
./scripts/deploy-lambda.sh --skip-build

# Créer/mettre à jour l'infrastructure
./scripts/deploy-lambda.sh --create-stack
```

## Workflow de déploiement

### Développement local → Production Lambda

```bash
# 1. Développer localement
make local

# 2. Tester localement
make test

# 3. Commit et push
git add .
git commit -m "Nouvelle feature"
git push

# 4. Déployer sur Lambda
./scripts/deploy-lambda.sh
```

### Pipeline recommandé

```
┌─────────────┐
│   Develop   │
│   locally   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Run tests  │
│  make test  │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    Push     │
│   to Git    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Deploy    │
│  to Lambda  │
└─────────────┘
```

## Utilisation SAM CLI

### Commandes utiles

```bash
cd infra

# Valider le template
sam validate

# Voir les changements avant déploiement
sam deploy --no-execute-changeset

# Déployer
sam deploy

# Voir les logs en direct
sam logs -n auth-manager-prod --tail

# Invoquer la fonction localement (nécessite Docker)
sam local invoke AuthManagerFunction

# Tester l'API localement
sam local start-api
```

## Tests après déploiement

### 1. Récupérer l'URL de l'API

```bash
aws cloudformation describe-stacks \
  --stack-name auth-manager-prod \
  --query 'Stacks[0].Outputs[?OutputKey==`ApiUrl`].OutputValue' \
  --output text
```

### 2. Tester les endpoints

```bash
# Santé
curl https://YOUR-API-URL/health

# Inscription
curl -X POST https://YOUR-API-URL/auth/register \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "test@example.com",
    "username": "testuser",
    "password": "SecurePass123!"
  }'

# Connexion
curl -X POST https://YOUR-API-URL/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'
```

## Monitoring

### Logs CloudWatch

```bash
# Logs Lambda en temps réel
sam logs -n auth-manager-prod --tail

# Ou avec AWS CLI
aws logs tail /aws/lambda/auth-manager-prod --follow

# Logs API Gateway
aws logs tail /aws/apigateway/auth-manager-prod --follow
```

### Métriques CloudWatch

Accédez à la console AWS CloudWatch pour voir:
- Nombre d'invocations Lambda
- Durée d'exécution
- Erreurs et throttles
- Requêtes API Gateway

### Alarmes configurées

Deux alarmes sont créées automatiquement:

1. **auth-manager-prod-errors**
   - Alerte si plus de 10 erreurs en 5 minutes
   - 2 périodes d'évaluation

2. **auth-manager-prod-throttles**
   - Alerte dès qu'un throttle est détecté

## Mises à jour

### Mettre à jour le code applicatif

```bash
# Modifier le code Rust
vim src/handlers/auth.rs

# Tester localement
make test

# Déployer sur Lambda
./scripts/deploy-lambda.sh
```

### Mettre à jour l'infrastructure

```bash
# Modifier le template SAM
vim infra/template.yaml

# Déployer les changements
./scripts/deploy-lambda.sh --create-stack
```

### Mettre à jour les paramètres

```bash
# Modifier la configuration
vim infra/samconfig.toml

# Redéployer l'infrastructure
./scripts/deploy-lambda.sh --create-stack
```

## Gestion des secrets

### Option 1: Paramètres SAM (actuel)

Les secrets sont passés via `samconfig.toml`. **À ne pas committer avec de vraies valeurs !**

```toml
# .gitignore doit contenir:
infra/samconfig.toml
```

### Option 2: AWS Secrets Manager (recommandé pour prod)

1. Créer un secret:
```bash
aws secretsmanager create-secret \
  --name auth-manager/jwt-secret \
  --secret-string "votre-secret-jwt"
```

2. Modifier `template.yaml` pour récupérer le secret

3. Ajouter les permissions IAM nécessaires

### Option 3: Variables d'environnement

```bash
export DATABASE_URL="postgres://..."
export JWT_SECRET="..."

# Passer via CLI
sam deploy --parameter-overrides \
  DatabaseUrl=$DATABASE_URL \
  JwtSecret=$JWT_SECRET
```

## Coûts estimés (Production)

### Trafic moyen (1M requêtes/mois)

| Service | Coût mensuel |
|---------|--------------|
| Lambda (1M invocations, 1GB RAM, 1s avg) | ~$20 |
| API Gateway HTTP API (1M requêtes) | ~$1 |
| ECR (stockage images) | ~$1 |
| CloudWatch Logs (10 GB) | ~$5 |
| **Total** | **~$27/mois** |

### Trafic élevé (10M requêtes/mois)

| Service | Coût mensuel |
|---------|--------------|
| Lambda | ~$180 |
| API Gateway | ~$10 |
| ECR | ~$1 |
| CloudWatch | ~$20 |
| **Total** | **~$211/mois** |

> **Note**: Les coûts de la base de données (RDS/Aurora) ne sont pas inclus.
> RDS t3.micro: ~$15/mois • Aurora Serverless v2: à partir de ~$40/mois

## Sécurité

### Bonnes pratiques

✅ **Secrets**
- Utilisez AWS Secrets Manager pour les secrets sensibles
- Ne commitez jamais `samconfig.toml` avec de vraies valeurs
- Utilisez des secrets forts (min 32 caractères pour JWT)

✅ **Réseau**
- Déployez RDS dans un VPC privé
- Configurez Lambda dans le même VPC si nécessaire
- Utilisez des Security Groups restrictifs

✅ **IAM**
- Principe du moindre privilège
- Pas de clés API hardcodées
- Rotation régulière des credentials

✅ **CORS**
- Configurez des origines spécifiques en production
- Évitez `*` en production

✅ **Monitoring**
- Activez CloudTrail pour l'audit
- Configurez SNS pour les alarmes
- Revoyez régulièrement les logs

## Troubleshooting

### Erreur: "SAM CLI not installed"

```bash
# macOS
brew install aws-sam-cli

# Linux/Windows
pip install aws-sam-cli

# Vérifier l'installation
sam --version
```

### Erreur: "Stack already exists"

```bash
# Mettre à jour au lieu de créer
./scripts/deploy-lambda.sh  # sans --create-stack
```

### Erreur: "ECR repository not found"

```bash
# Créer d'abord l'infrastructure
./scripts/deploy-lambda.sh --create-stack
```

### Lambda timeout

```bash
# Augmenter le timeout dans samconfig.toml
parameter_overrides = [
    "LambdaTimeout=60"  # au lieu de 30
]

# Redéployer
./scripts/deploy-lambda.sh --create-stack
```

### Erreur de connexion base de données

**Vérifications:**
1. ✅ Le `DATABASE_URL` est-il correct ?
2. ✅ Lambda peut-il accéder à la BDD ? (VPC, Security Groups)
3. ✅ Les credentials sont-ils valides ?
4. ✅ La base de données existe-t-elle ?

**Tester la connexion:**
```bash
# Depuis votre machine
psql "postgres://user:pass@host:5432/db"

# Voir les logs Lambda
sam logs -n auth-manager-prod --tail
```

### Cold start lent

Les cold starts peuvent prendre 2-5 secondes pour une image Docker Rust.

**Solutions:**
1. Augmenter la mémoire Lambda (plus rapide)
2. Utiliser Provisioned Concurrency (coûteux)
3. Utiliser un Lambda warmer (invoke périodique)

## Nettoyage / Suppression

### Supprimer toute l'infrastructure

```bash
# ATTENTION: Supprime toutes les ressources !

# Via SAM
sam delete --stack-name auth-manager-prod

# Ou via AWS CLI
aws cloudformation delete-stack --stack-name auth-manager-prod

# Supprimer manuellement les images ECR si nécessaire
aws ecr delete-repository --repository-name auth-manager-prod --force
```

## Support et ressources

### Documentation

- [AWS SAM Documentation](https://docs.aws.amazon.com/serverless-application-model/)
- [SAM CLI Reference](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-command-reference.html)
- [Lambda Container Images](https://docs.aws.amazon.com/lambda/latest/dg/images-create.html)
- [API Gateway HTTP API](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api.html)

### Commandes de référence

```bash
# Déploiement initial
./scripts/deploy-lambda.sh --create-stack

# Déploiement standard
./scripts/deploy-lambda.sh

# Avec profil AWS
./scripts/deploy-lambda.sh -p production

# Logs en direct
sam logs -n auth-manager-prod --tail

# Supprimer le stack
sam delete --stack-name auth-manager-prod
```

### Aide

Pour toute question:
1. Vérifier les logs CloudWatch
2. Vérifier les événements CloudFormation dans la console AWS
3. Consulter la documentation SAM
4. Ouvrir une issue sur GitHub
