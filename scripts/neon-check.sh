#!/bin/bash
# ============================================================================
# Neon Database Health Check Script
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$PROJECT_ROOT/.env.production"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Neon Database Health Check${NC}"
echo "========================================"
echo ""

# Check if .env.production exists
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}❌ .env.production not found!${NC}"
    echo "Please create it first: cp .env.example .env.production"
    exit 1
fi

# Load environment variables
export $(cat "$ENV_FILE" | grep -v '^#' | xargs)

if [ -z "$DATABASE_URL" ]; then
    echo -e "${RED}❌ DATABASE_URL not set in .env.production${NC}"
    exit 1
fi

echo -e "${GREEN}✅ .env.production loaded${NC}"
echo ""

# Extract database info
DB_HOST=$(echo $DATABASE_URL | sed -n 's/.*@\(.*\):.*/\1/p')
DB_NAME=$(echo $DATABASE_URL | sed -n 's/.*\/\([^?]*\).*/\1/p')

echo "📊 Database Information:"
echo "  Host: $DB_HOST"
echo "  Database: $DB_NAME"
echo ""

# Check 1: Network connectivity
echo -e "${YELLOW}1. Testing network connectivity...${NC}"
if ping -c 1 -W 2 $DB_HOST &> /dev/null; then
    echo -e "${GREEN}   ✅ Host is reachable${NC}"
else
    echo -e "${YELLOW}   ⚠️  Host ping failed (might be blocked, but DB could still work)${NC}"
fi
echo ""

# Check 2: Database connection
echo -e "${YELLOW}2. Testing database connection...${NC}"
if psql "$DATABASE_URL" -c "SELECT 1" &> /dev/null; then
    echo -e "${GREEN}   ✅ Database connection successful${NC}"
else
    echo -e "${RED}   ❌ Database connection failed!${NC}"
    echo "   Check your DATABASE_URL and password"
    exit 1
fi
echo ""

# Check 3: List tables
echo -e "${YELLOW}3. Checking database schema...${NC}"
TABLES=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';")
echo -e "   📋 Tables found: ${GREEN}$TABLES${NC}"

if [ "$TABLES" -gt 0 ]; then
    echo ""
    echo "   Tables:"
    psql "$DATABASE_URL" -c "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE' ORDER BY table_name;" | grep -v "(" | grep -v "rows)" | grep -v "^$" | sed 's/^/     /'
else
    echo -e "   ${YELLOW}⚠️  No tables found. Run migrations: make migrate-prod${NC}"
fi
echo ""

# Check 4: Migration status
echo -e "${YELLOW}4. Checking migration status...${NC}"
if command -v diesel &> /dev/null; then
    echo "   Migrations:"
    diesel migration list 2>&1 | sed 's/^/     /'
    echo ""
else
    echo -e "   ${YELLOW}⚠️  diesel CLI not installed. Install: cargo install diesel_cli --no-default-features --features postgres${NC}"
    echo ""
fi

# Check 5: Database stats
echo -e "${YELLOW}5. Database statistics...${NC}"
USER_COUNT=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM users;" 2>/dev/null || echo "0")
echo -e "   👤 Users: ${GREEN}$USER_COUNT${NC}"

DB_SIZE=$(psql "$DATABASE_URL" -t -c "SELECT pg_size_pretty(pg_database_size(current_database()));" 2>/dev/null || echo "unknown")
echo -e "   💾 Database size: ${GREEN}$DB_SIZE${NC}"
echo ""

# Check 6: SSL status
echo -e "${YELLOW}6. Checking SSL connection...${NC}"
SSL_STATUS=$(psql "$DATABASE_URL" -t -c "SHOW ssl;" 2>/dev/null || echo "unknown")
echo -e "   🔒 SSL: ${GREEN}$SSL_STATUS${NC}"
echo ""

# Summary
echo "========================================"
echo -e "${GREEN}✅ Health check completed!${NC}"
echo ""
echo "Next steps:"
echo "  • Run migrations: make migrate-prod"
echo "  • Test API: cd auth-manager && cargo run"
echo "  • Deploy: make deploy"
