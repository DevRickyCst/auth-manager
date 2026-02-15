# Production Setup Guide

## 🎯 Overview

This guide covers production deployment for **auth-manager** using:
- **Database**: Neon PostgreSQL (Serverless)
- **Backend**: AWS Lambda + SAM (Serverless)
- **Environment**: `infra/params/prod.json` (NOT committed to git)

---

## 📋 Quick Start Checklist

Before deploying to production:

- [ ] Regenerate Neon database password (if exposed)
- [ ] Create `infra/params/prod.json` with production credentials
- [ ] Run database migrations
- [ ] Deploy to Lambda with `make deploy`
- [ ] Test production endpoints

---

## 1️⃣ Secure Production Credentials

### A. Regenerate Neon Password (if needed)

```bash
# 1. Go to Neon Console
https://console.neon.tech/app/projects

# 2. Select "dofus-graal" project

# 3. Settings → Connection string → Reset password

# 4. Copy the new connection string (pooler)
# Example:
# postgresql://neondb_owner:NEW_PASSWORD@ep-xxx.eu-central-1.aws.neon.tech/dofus-graal?sslmode=require
```

### B. Generate JWT Secret

```bash
# Generate a strong 32+ character secret
openssl rand -base64 32

# Copy the output for next step
```

### C. Create Production Parameters File

```bash
cd /Users/aymericcreusot/Documents/Aymeric/github/dofus-graal/auth-manager

# Copy template
cp infra/params/prod.json.example infra/params/prod.json

# Edit with your credentials
# {
#   "DatabaseUrl": "postgresql://neondb_owner:NEW_PASSWORD@ep-xxx.neon.tech/dofus-graal?sslmode=require",
#   "JwtSecret": "OUTPUT_FROM_OPENSSL_RAND",
#   "FrontendUrl": "https://dofus-graal.eu"
# }
```

**⚠️ IMPORTANT:**
- `infra/params/prod.json` is in `.gitignore` (NEVER commit it!)
- Always use `?sslmode=require` for database security
- JWT secret must be 32+ characters

---

## 2️⃣ Run Database Migrations

### Option A: Locally (Recommended)

```bash
cd /Users/aymericcreusot/Documents/Aymeric/github/dofus-graal/auth-manager

# Method 1: Via Makefile (easiest)
make migrate-prod

# Method 2: Manually
export DATABASE_URL=$(jq -r '.DatabaseUrl' infra/params/prod.json)
diesel migration run
```

**Expected output:**
```
Running migration 00000000000000_diesel_initial_setup
Running migration 00000000000001_create_users
✅ Migrations completed successfully!
```

### Option B: Via Docker (if diesel_cli not installed)

```bash
# Load DATABASE_URL from params
export DATABASE_URL=$(jq -r '.DatabaseUrl' infra/params/prod.json)

# Run migrations via Docker
docker run --rm \
  -e DATABASE_URL="$DATABASE_URL" \
  -v $(pwd)/migrations:/migrations \
  willsquire/diesel-cli \
  diesel migration run --migration-dir /migrations
```

### Verify Migrations

```bash
# List tables
psql $(jq -r '.DatabaseUrl' infra/params/prod.json) -c "\dt"

# Expected tables:
#  public | __diesel_schema_migrations
#  public | login_attempts
#  public | refresh_tokens
#  public | user_identities
#  public | users
```

---

## 3️⃣ Deploy to AWS Lambda

### Prerequisites

Ensure you have:
- **AWS CLI** configured (`aws configure --profile perso`)
- **SAM CLI** installed (`pip install aws-sam-cli` or `brew install aws-sam-cli`)
- **Docker** running
- **jq** installed (`brew install jq`)

### First Time Deployment (Create Stack)

```bash
cd /Users/aymericcreusot/Documents/Aymeric/github/dofus-graal/auth-manager

# Create Lambda infrastructure
make deploy-create-stack

# This will:
# 1. Create ECR repository
# 2. Build Docker image
# 3. Push to ECR
# 4. Deploy SAM stack (Lambda + API Gateway)
# 5. Inject params from infra/params/prod.json
```

### Update Deployment (After Code Changes)

```bash
# Build, push, and update Lambda
make deploy

# Skip Docker build (faster, for SAM-only changes)
make deploy-only
```

### Deployment Workflow

The deploy script (`scripts/deploy-lambda.sh`) will:
1. Check `infra/params/prod.json` exists
2. Read parameters from JSON
3. Login to ECR
4. Build Docker image (Rust optimized)
5. Push to ECR
6. Deploy with SAM (`--parameter-overrides` from JSON)
7. Update Lambda function

---

## 4️⃣ Verify Production Deployment

### Check Stack Status

```bash
make deploy-status

# Shows:
# - Stack status (CREATE_COMPLETE, UPDATE_COMPLETE, etc.)
# - API Gateway URL
# - Lambda ARN
# - ECR repository
```

### Test Endpoints

```bash
# Get API URL from stack outputs
API_URL=$(aws cloudformation describe-stacks \
  --stack-name auth-manager-prod \
  --region eu-central-1 \
  --profile perso \
  --query 'Stacks[0].Outputs[?OutputKey==`ApiUrl`].OutputValue' \
  --output text)

# Test health endpoint
curl $API_URL/health

# Expected: {"status":"ok"}

# Test registration
curl -X POST $API_URL/auth/register \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "test@example.com",
    "username": "testuser",
    "password": "SecurePass123!"
  }'
```

### View Logs

```bash
# Real-time Lambda logs
make deploy-logs

# Or directly with SAM
sam logs -n auth-manager-prod --tail --region eu-central-1 --profile perso

# Or via AWS Console
https://eu-central-1.console.aws.amazon.com/cloudwatch/home?region=eu-central-1#logsV2:log-groups/log-group/$252Faws$252Flambda$252Fauth-manager-prod
```

---

## 🔧 Update Production Credentials

If you need to change DATABASE_URL or JWT_SECRET:

```bash
# 1. Edit params file
vim infra/params/prod.json

# 2. Redeploy (SAM will update Lambda env vars)
make deploy-only

# 3. Verify new env vars
aws lambda get-function-configuration \
  --function-name auth-manager-prod \
  --profile perso \
  --region eu-central-1 \
  --query 'Environment.Variables' \
  --output json
```

---

## 📊 Monitoring & Maintenance

### CloudWatch Metrics

```bash
# Error rate
aws cloudwatch get-metric-statistics \
  --namespace AWS/Lambda \
  --metric-name Errors \
  --dimensions Name=FunctionName,Value=auth-manager-prod \
  --start-time $(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M:%S) \
  --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
  --period 300 \
  --statistics Sum \
  --region eu-central-1 \
  --profile perso
```

### Database Monitoring (Neon)

```
https://console.neon.tech/app/projects
→ Select "dofus-graal"
→ Monitoring tab
```

**Free Tier Limits:**
- Storage: 500 MB
- Compute: 191.9h/month
- Connections: 100 simultaneous

---

## 🐛 Troubleshooting

### `params/prod.json not found`

```bash
# Create from template
cp infra/params/prod.json.example infra/params/prod.json

# Edit with your credentials
vim infra/params/prod.json
```

### `jq: command not found`

```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq

# Verify
jq --version
```

### Database Connection Errors

```bash
# Test connection
psql "$(jq -r '.DatabaseUrl' infra/params/prod.json)" -c "SELECT 1;"

# Check SSL mode
echo "$(jq -r '.DatabaseUrl' infra/params/prod.json)" | grep sslmode

# Should end with: ?sslmode=require
```

### Lambda Deployment Fails

```bash
# Check AWS credentials
aws sts get-caller-identity --profile perso

# Verify ECR repository exists
aws ecr describe-repositories \
  --repository-names auth-manager-prod \
  --region eu-central-1 \
  --profile perso

# Check SAM stack
aws cloudformation describe-stacks \
  --stack-name auth-manager-prod \
  --region eu-central-1 \
  --profile perso
```

---

## 🔒 Security Best Practices

### ✅ DO

- ✅ Store credentials in `infra/params/prod.json` (not committed)
- ✅ Use strong JWT secrets (32+ characters)
- ✅ Enable SSL for database (`sslmode=require`)
- ✅ Regenerate passwords after exposure
- ✅ Use AWS IAM roles for Lambda
- ✅ Monitor CloudWatch logs regularly

### ❌ DON'T

- ❌ Commit `infra/params/prod.json` to git
- ❌ Share credentials publicly
- ❌ Use the same password for local/prod
- ❌ Disable SSL in production
- ❌ Hardcode credentials in code

---

## 🚀 Useful Commands

```bash
# Deploy
make deploy                   # Build + push + deploy
make deploy-only              # Deploy without rebuilding
make deploy-create-stack      # First-time stack creation

# Monitoring
make deploy-status            # Show stack outputs
make deploy-logs              # Tail Lambda logs

# Database
make migrate-prod             # Run migrations on production
make db-shell-prod            # Connect to Neon PostgreSQL
make db-check-prod            # Check database health

# Cleanup
make deploy-delete            # Delete entire stack (⚠️ DESTRUCTIVE!)
```

---

## 📞 Support & Resources

- **Neon Docs**: https://neon.tech/docs
- **AWS Lambda**: https://docs.aws.amazon.com/lambda/
- **SAM CLI**: https://docs.aws.amazon.com/serverless-application-model/
- **Diesel**: https://diesel.rs/guides/getting-started

---

## ✅ Production Deployment Checklist

Before going live:

- [ ] Neon password regenerated (if exposed)
- [ ] `infra/params/prod.json` created and configured
- [ ] JWT secret generated (32+ characters)
- [ ] Database migrations run and verified
- [ ] Lambda deployed successfully
- [ ] Health endpoint returns 200 OK
- [ ] Registration/login tested
- [ ] CloudWatch logs monitored
- [ ] CORS configured for frontend URL
- [ ] API Gateway URL documented

---

**🎉 Your auth-manager is production-ready!**
