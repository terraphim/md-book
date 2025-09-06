#!/bin/bash

# Deploy Cloudflare Worker for MD-Book
# This script deploys the worker with appropriate environment settings

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default environment
ENVIRONMENT=${1:-production}
WORKER_DIR="worker"

echo -e "${GREEN}🚀 Deploying MD-Book Worker to ${ENVIRONMENT}${NC}"

# Check if wrangler is installed
if ! command -v wrangler &> /dev/null; then
    echo -e "${YELLOW}⚠️  Installing Wrangler CLI...${NC}"
    npm install -g wrangler
fi

# Check if worker directory exists
if [ ! -d "$WORKER_DIR" ]; then
    echo -e "${RED}❌ Worker directory not found: $WORKER_DIR${NC}"
    exit 1
fi

# Check if required environment variables are set
if [ -z "$CLOUDFLARE_API_TOKEN" ] || [ -z "$CLOUDFLARE_ACCOUNT_ID" ]; then
    echo -e "${RED}❌ Missing required environment variables:${NC}"
    echo "   CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID must be set"
    echo ""
    echo "   Set them in your environment:"
    echo "   export CLOUDFLARE_API_TOKEN=\"your-token-here\""
    echo "   export CLOUDFLARE_ACCOUNT_ID=\"your-account-id-here\""
    exit 1
fi

# Change to worker directory
cd "$WORKER_DIR"

echo -e "${YELLOW}📦 Preparing worker deployment...${NC}"

# Validate worker code
echo -e "${YELLOW}🔍 Validating worker code...${NC}"
if [ -f "src/index.js" ]; then
    # Basic syntax check
    node -c src/index.js
    echo -e "${GREEN}✅ Worker code validation passed${NC}"
else
    echo -e "${RED}❌ Worker source file not found: src/index.js${NC}"
    exit 1
fi

# Deploy based on environment
case $ENVIRONMENT in
    "production")
        echo -e "${GREEN}🚀 Deploying to production...${NC}"
        wrangler deploy --env production
        ;;
    "staging")
        echo -e "${YELLOW}🚀 Deploying to staging...${NC}"
        wrangler deploy --env staging
        ;;
    *)
        echo -e "${RED}❌ Unknown environment: $ENVIRONMENT${NC}"
        echo "   Supported environments: production, staging"
        exit 1
        ;;
esac

echo -e "${GREEN}✅ Worker deployment completed successfully!${NC}"

# Show deployment info
echo ""
echo -e "${GREEN}📋 Deployment Information:${NC}"
echo "   Environment: $ENVIRONMENT"
echo "   Timestamp: $(date)"
echo ""

# List recent deployments
echo -e "${YELLOW}📝 Recent deployments:${NC}"
wrangler deployments list --limit 5 2>/dev/null || echo "   Unable to fetch deployment history"

echo ""
echo -e "${GREEN}🎉 MD-Book Worker is now live!${NC}"

# Return to original directory
cd ..