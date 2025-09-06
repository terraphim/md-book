#!/bin/bash

# 1Password Setup Script for MD-Book Deployment
# This script sets up 1Password CLI integration for secure secret management

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VAULT_NAME="MD-Book-Deployment"
CLOUDFLARE_ITEM="Cloudflare"

echo -e "${BLUE}🔐 1Password Setup for MD-Book${NC}"
echo -e "${BLUE}===================================${NC}"
echo ""

# Check if 1Password CLI is installed
check_op_cli() {
    echo -e "${YELLOW}🔍 Checking 1Password CLI installation...${NC}"
    
    if ! command -v op &> /dev/null; then
        echo -e "${RED}❌ 1Password CLI not found${NC}"
        echo -e "${YELLOW}📋 Installation instructions:${NC}"
        echo "  macOS: brew install 1password-cli"
        echo "  Linux: See https://developer.1password.com/docs/cli/get-started/"
        echo "  Windows: Use package manager or download from 1Password"
        echo ""
        exit 1
    fi
    
    echo -e "${GREEN}✅ 1Password CLI found: $(op --version)${NC}"
}

# Check authentication status
check_authentication() {
    echo -e "${YELLOW}🔍 Checking 1Password authentication...${NC}"
    
    if ! op account list &> /dev/null; then
        echo -e "${RED}❌ Not authenticated with 1Password${NC}"
        echo -e "${YELLOW}🔑 Please sign in to 1Password:${NC}"
        echo "  op account add"
        echo "  op signin"
        echo ""
        exit 1
    fi
    
    echo -e "${GREEN}✅ Authenticated with 1Password${NC}"
}

# Check if vault exists
check_vault() {
    echo -e "${YELLOW}🔍 Checking for vault: ${VAULT_NAME}...${NC}"
    
    if ! op vault get "$VAULT_NAME" &> /dev/null; then
        echo -e "${YELLOW}⚠️  Vault '${VAULT_NAME}' not found${NC}"
        echo -e "${BLUE}📝 Creating vault...${NC}"
        
        if op vault create "$VAULT_NAME" --description "MD-Book deployment secrets"; then
            echo -e "${GREEN}✅ Vault '${VAULT_NAME}' created successfully${NC}"
        else
            echo -e "${RED}❌ Failed to create vault${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}✅ Vault '${VAULT_NAME}' exists${NC}"
    fi
}

# Create Cloudflare item if it doesn't exist
create_cloudflare_item() {
    echo -e "${YELLOW}🔍 Checking for Cloudflare item...${NC}"
    
    if ! op item get "$CLOUDFLARE_ITEM" --vault="$VAULT_NAME" &> /dev/null; then
        echo -e "${YELLOW}⚠️  Cloudflare item not found${NC}"
        echo -e "${BLUE}📝 Creating Cloudflare item...${NC}"
        
        echo -e "${YELLOW}🔑 Please provide your Cloudflare credentials:${NC}"
        echo ""
        
        # Get API token
        echo -e "${BLUE}Enter your Cloudflare API Token:${NC}"
        echo "  (Get it from: https://dash.cloudflare.com/profile/api-tokens)"
        read -r -s API_TOKEN
        echo ""
        
        # Get Account ID
        echo -e "${BLUE}Enter your Cloudflare Account ID:${NC}"
        echo "  (Find it in: https://dash.cloudflare.com/ - right sidebar)"
        read -r ACCOUNT_ID
        echo ""
        
        # Create the item
        if op item create \
            --category="API Credential" \
            --title="$CLOUDFLARE_ITEM" \
            --vault="$VAULT_NAME" \
            --url="https://dash.cloudflare.com/" \
            "api_token[password]=$API_TOKEN" \
            "account_id[text]=$ACCOUNT_ID"; then
            echo -e "${GREEN}✅ Cloudflare item created successfully${NC}"
        else
            echo -e "${RED}❌ Failed to create Cloudflare item${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}✅ Cloudflare item exists${NC}"
    fi
}

# Test secret retrieval
test_secret_retrieval() {
    echo -e "${YELLOW}🔍 Testing secret retrieval...${NC}"
    
    # Test API token
    if API_TOKEN=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/api_token" 2>/dev/null); then
        echo -e "${GREEN}✅ API token retrieved successfully${NC}"
        echo "  Token starts with: ${API_TOKEN:0:10}..."
    else
        echo -e "${RED}❌ Failed to retrieve API token${NC}"
        return 1
    fi
    
    # Test Account ID
    if ACCOUNT_ID=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/account_id" 2>/dev/null); then
        echo -e "${GREEN}✅ Account ID retrieved successfully${NC}"
        echo "  Account ID: ${ACCOUNT_ID:0:8}...${ACCOUNT_ID: -4}"
    else
        echo -e "${RED}❌ Failed to retrieve Account ID${NC}"
        return 1
    fi
    
    return 0
}

# Generate .env file from 1Password
generate_env_file() {
    echo -e "${YELLOW}🔍 Generating .env file from 1Password...${NC}"
    
    if [ ! -f ".env.1password" ]; then
        echo -e "${RED}❌ .env.1password template not found${NC}"
        echo "  Please ensure .env.1password exists in the project root"
        return 1
    fi
    
    if op inject -i ".env.1password" -o ".env"; then
        echo -e "${GREEN}✅ .env file generated successfully${NC}"
        echo -e "${YELLOW}📝 Note: .env file contains actual secrets - do not commit it${NC}"
    else
        echo -e "${RED}❌ Failed to generate .env file${NC}"
        return 1
    fi
}

# Validate Cloudflare connection
validate_cloudflare_connection() {
    echo -e "${YELLOW}🔍 Validating Cloudflare connection...${NC}"
    
    # Source the generated .env file
    if [ -f ".env" ]; then
        set -a
        source .env
        set +a
    else
        echo -e "${RED}❌ .env file not found. Run this script first to generate it.${NC}"
        return 1
    fi
    
    # Test Cloudflare API
    if command -v curl &> /dev/null && [ -n "$CLOUDFLARE_API_TOKEN" ]; then
        if curl -s -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
           "https://api.cloudflare.com/client/v4/user/tokens/verify" | grep -q '"success":true'; then
            echo -e "${GREEN}✅ Cloudflare API connection successful${NC}"
        else
            echo -e "${RED}❌ Cloudflare API connection failed${NC}"
            echo "  Check your API token permissions"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  Skipping API validation (curl not available or token empty)${NC}"
    fi
}

# Display usage instructions
show_usage_instructions() {
    echo ""
    echo -e "${BLUE}🎉 1Password setup complete!${NC}"
    echo ""
    echo -e "${YELLOW}📋 Usage Instructions:${NC}"
    echo ""
    echo -e "${BLUE}Local Development:${NC}"
    echo "  # Option 1: Use op run with env file"
    echo "  op run --env-file=.env.1password -- ./scripts/deploy.sh"
    echo ""
    echo "  # Option 2: Generate .env file"
    echo "  op inject -i .env.1password -o .env"
    echo "  ./scripts/deploy.sh"
    echo ""
    echo -e "${BLUE}GitHub Actions Setup:${NC}"
    echo "  1. Create a service account in 1Password"
    echo "  2. Add service account token to GitHub secrets:"
    echo "     gh secret set OP_SERVICE_ACCOUNT_TOKEN --body \"ops_...\""
    echo "  3. The deployment workflow will automatically use 1Password"
    echo ""
    echo -e "${BLUE}Manual Secret Sync to GitHub:${NC}"
    echo "  ./scripts/sync-secrets-to-github.sh"
    echo ""
    echo -e "${YELLOW}⚠️  Security Notes:${NC}"
    echo "  • Never commit .env files"
    echo "  • Use service accounts for automation"
    echo "  • Rotate secrets regularly"
    echo "  • Monitor 1Password access logs"
}

# Main execution
main() {
    check_op_cli
    check_authentication
    check_vault
    create_cloudflare_item
    
    if test_secret_retrieval; then
        echo ""
        echo -e "${YELLOW}🔧 Setting up local environment...${NC}"
        generate_env_file
        validate_cloudflare_connection
        show_usage_instructions
    else
        echo -e "${RED}❌ Secret retrieval test failed${NC}"
        echo "  Please check your 1Password setup and try again"
        exit 1
    fi
}

# Handle script arguments
case "${1:-setup}" in
    "test")
        echo -e "${BLUE}🧪 Testing 1Password integration...${NC}"
        check_op_cli
        check_authentication
        test_secret_retrieval
        ;;
    "env")
        echo -e "${BLUE}🔧 Generating .env file...${NC}"
        generate_env_file
        ;;
    "validate")
        echo -e "${BLUE}🔍 Validating setup...${NC}"
        validate_cloudflare_connection
        ;;
    "setup"|*)
        main
        ;;
esac

echo ""
echo -e "${GREEN}✅ Script completed successfully${NC}"