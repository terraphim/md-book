#!/bin/bash

# Secret Validation Script
# Validates that secrets are accessible and properly configured for deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VAULT_NAME="MD-Book-Deployment"
CLOUDFLARE_ITEM="Cloudflare"
CHECK_1PASSWORD=true
CHECK_ENV=true
CHECK_GITHUB=false
CHECK_CLOUDFLARE=false
VERBOSE=false

echo -e "${BLUE}🔍 Secret Validation${NC}"
echo -e "${BLUE}===================${NC}"
echo ""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-1password)
            CHECK_1PASSWORD=false
            shift
            ;;
        --no-env)
            CHECK_ENV=false
            shift
            ;;
        --check-github)
            CHECK_GITHUB=true
            shift
            ;;
        --check-cloudflare)
            CHECK_CLOUDFLARE=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --no-1password      Skip 1Password validation"
            echo "  --no-env           Skip environment variable validation"
            echo "  --check-github     Validate GitHub secrets access"
            echo "  --check-cloudflare Validate Cloudflare API connection"
            echo "  --verbose, -v      Enable verbose output"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Validation results
VALIDATION_PASSED=true

# Log function for verbose output
log_verbose() {
    if [ "$VERBOSE" = true ]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

# Check 1Password setup
validate_1password() {
    if [ "$CHECK_1PASSWORD" = false ]; then
        echo -e "${YELLOW}⚠️  Skipping 1Password validation${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🔍 Validating 1Password setup...${NC}"
    
    # Check if 1Password CLI is installed
    if ! command -v op &> /dev/null; then
        echo -e "${RED}❌ 1Password CLI not found${NC}"
        echo "  Install with: brew install 1password-cli"
        VALIDATION_PASSED=false
        return 1
    fi
    
    log_verbose "1Password CLI found: $(op --version)"
    echo -e "${GREEN}✅ 1Password CLI installed${NC}"
    
    # Check authentication
    if ! op account list &> /dev/null; then
        echo -e "${RED}❌ Not authenticated with 1Password${NC}"
        echo "  Run: op signin"
        VALIDATION_PASSED=false
        return 1
    fi
    
    log_verbose "1Password authentication confirmed"
    echo -e "${GREEN}✅ 1Password authenticated${NC}"
    
    # Check vault access
    if ! op vault get "$VAULT_NAME" &> /dev/null; then
        echo -e "${RED}❌ Cannot access vault: $VAULT_NAME${NC}"
        echo "  Run: ./scripts/setup-1password.sh"
        VALIDATION_PASSED=false
        return 1
    fi
    
    log_verbose "Vault '$VAULT_NAME' accessible"
    echo -e "${GREEN}✅ Vault accessible: $VAULT_NAME${NC}"
    
    # Check Cloudflare item
    if ! op item get "$CLOUDFLARE_ITEM" --vault="$VAULT_NAME" &> /dev/null; then
        echo -e "${RED}❌ Cannot access Cloudflare item${NC}"
        echo "  Run: ./scripts/setup-1password.sh"
        VALIDATION_PASSED=false
        return 1
    fi
    
    log_verbose "Cloudflare item accessible"
    echo -e "${GREEN}✅ Cloudflare item accessible${NC}"
    
    # Test secret retrieval
    if API_TOKEN=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/api_token" 2>/dev/null); then
        log_verbose "API token retrieved: ${API_TOKEN:0:10}..."
        echo -e "${GREEN}✅ API token retrieval successful${NC}"
    else
        echo -e "${RED}❌ Failed to retrieve API token${NC}"
        VALIDATION_PASSED=false
    fi
    
    if ACCOUNT_ID=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/account_id" 2>/dev/null); then
        log_verbose "Account ID retrieved: ${ACCOUNT_ID:0:8}...${ACCOUNT_ID: -4}"
        echo -e "${GREEN}✅ Account ID retrieval successful${NC}"
    else
        echo -e "${RED}❌ Failed to retrieve Account ID${NC}"
        VALIDATION_PASSED=false
    fi
    
    return 0
}

# Check environment variables
validate_environment() {
    if [ "$CHECK_ENV" = false ]; then
        echo -e "${YELLOW}⚠️  Skipping environment variable validation${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🔍 Validating environment variables...${NC}"
    
    # Check if .env file exists
    if [ -f ".env" ]; then
        echo -e "${GREEN}✅ .env file found${NC}"
        log_verbose ".env file exists and is readable"
        
        # Source environment variables
        set -a
        source .env
        set +a
    elif [ -f ".env.1password" ]; then
        echo -e "${YELLOW}⚠️  .env file not found, but .env.1password exists${NC}"
        echo "  Generate .env with: op inject -i .env.1password -o .env"
    else
        echo -e "${YELLOW}⚠️  No .env files found${NC}"
    fi
    
    # Check required environment variables
    if [ -n "$CLOUDFLARE_API_TOKEN" ]; then
        log_verbose "CLOUDFLARE_API_TOKEN set: ${CLOUDFLARE_API_TOKEN:0:10}..."
        echo -e "${GREEN}✅ CLOUDFLARE_API_TOKEN is set${NC}"
    else
        echo -e "${RED}❌ CLOUDFLARE_API_TOKEN not set${NC}"
        VALIDATION_PASSED=false
    fi
    
    if [ -n "$CLOUDFLARE_ACCOUNT_ID" ]; then
        log_verbose "CLOUDFLARE_ACCOUNT_ID set: ${CLOUDFLARE_ACCOUNT_ID:0:8}...${CLOUDFLARE_ACCOUNT_ID: -4}"
        echo -e "${GREEN}✅ CLOUDFLARE_ACCOUNT_ID is set${NC}"
    else
        echo -e "${RED}❌ CLOUDFLARE_ACCOUNT_ID not set${NC}"
        VALIDATION_PASSED=false
    fi
    
    # Check other configuration variables
    local config_vars=("INPUT_DIR" "OUTPUT_DIR" "SKIP_TESTS" "DEPLOY_WORKER")
    for var in "${config_vars[@]}"; do
        if [ -n "${!var}" ]; then
            log_verbose "$var = ${!var}"
            echo -e "${GREEN}✅ $var configured${NC}"
        else
            log_verbose "$var not set (using defaults)"
        fi
    done
    
    return 0
}

# Check GitHub secrets (requires GitHub CLI)
validate_github_secrets() {
    if [ "$CHECK_GITHUB" = false ]; then
        return 0
    fi
    
    echo -e "${YELLOW}🔍 Validating GitHub secrets...${NC}"
    
    # Check if GitHub CLI is available
    if ! command -v gh &> /dev/null; then
        echo -e "${RED}❌ GitHub CLI not found${NC}"
        echo "  Install with: brew install gh"
        VALIDATION_PASSED=false
        return 1
    fi
    
    # Check authentication
    if ! gh auth status &> /dev/null; then
        echo -e "${RED}❌ Not authenticated with GitHub${NC}"
        echo "  Run: gh auth login"
        VALIDATION_PASSED=false
        return 1
    fi
    
    echo -e "${GREEN}✅ GitHub CLI authenticated${NC}"
    
    # List secrets (this requires appropriate permissions)
    if SECRET_LIST=$(gh secret list 2>/dev/null); then
        log_verbose "GitHub secrets list retrieved"
        
        # Check for required secrets
        if echo "$SECRET_LIST" | grep -q "CLOUDFLARE_API_TOKEN"; then
            echo -e "${GREEN}✅ CLOUDFLARE_API_TOKEN exists in GitHub secrets${NC}"
        else
            echo -e "${RED}❌ CLOUDFLARE_API_TOKEN not found in GitHub secrets${NC}"
            VALIDATION_PASSED=false
        fi
        
        if echo "$SECRET_LIST" | grep -q "CLOUDFLARE_ACCOUNT_ID"; then
            echo -e "${GREEN}✅ CLOUDFLARE_ACCOUNT_ID exists in GitHub secrets${NC}"
        else
            echo -e "${RED}❌ CLOUDFLARE_ACCOUNT_ID not found in GitHub secrets${NC}"
            VALIDATION_PASSED=false
        fi
        
        if echo "$SECRET_LIST" | grep -q "OP_SERVICE_ACCOUNT_TOKEN"; then
            echo -e "${GREEN}✅ OP_SERVICE_ACCOUNT_TOKEN exists in GitHub secrets${NC}"
        else
            echo -e "${YELLOW}⚠️  OP_SERVICE_ACCOUNT_TOKEN not found (optional for 1Password integration)${NC}"
        fi
        
        if [ "$VERBOSE" = true ]; then
            echo -e "${BLUE}[DEBUG] All GitHub secrets:${NC}"
            echo "$SECRET_LIST"
        fi
    else
        echo -e "${YELLOW}⚠️  Cannot list GitHub secrets (may require admin permissions)${NC}"
    fi
    
    return 0
}

# Validate Cloudflare API connection
validate_cloudflare_api() {
    if [ "$CHECK_CLOUDFLARE" = false ]; then
        return 0
    fi
    
    echo -e "${YELLOW}🔍 Validating Cloudflare API connection...${NC}"
    
    # Check if curl is available
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}❌ curl not found${NC}"
        VALIDATION_PASSED=false
        return 1
    fi
    
    # Ensure we have the token (try to load from 1Password if needed)
    if [ -z "$CLOUDFLARE_API_TOKEN" ] && [ "$CHECK_1PASSWORD" = true ]; then
        if op account list &> /dev/null && op vault get "$VAULT_NAME" &> /dev/null; then
            CLOUDFLARE_API_TOKEN=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/api_token" 2>/dev/null)
        fi
    fi
    
    if [ -z "$CLOUDFLARE_API_TOKEN" ]; then
        echo -e "${RED}❌ CLOUDFLARE_API_TOKEN not available for testing${NC}"
        VALIDATION_PASSED=false
        return 1
    fi
    
    # Test token verification
    log_verbose "Testing Cloudflare API token verification"
    if API_RESPONSE=$(curl -s -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
                           "https://api.cloudflare.com/client/v4/user/tokens/verify"); then
        
        if echo "$API_RESPONSE" | grep -q '"success":true'; then
            echo -e "${GREEN}✅ Cloudflare API token is valid${NC}"
            
            if [ "$VERBOSE" = true ]; then
                TOKEN_INFO=$(echo "$API_RESPONSE" | grep -o '"expires_on":"[^"]*"' | cut -d'"' -f4)
                if [ -n "$TOKEN_INFO" ] && [ "$TOKEN_INFO" != "null" ]; then
                    echo -e "${BLUE}[DEBUG] Token expires: $TOKEN_INFO${NC}"
                fi
            fi
        else
            echo -e "${RED}❌ Cloudflare API token is invalid${NC}"
            log_verbose "API response: $API_RESPONSE"
            VALIDATION_PASSED=false
        fi
    else
        echo -e "${RED}❌ Failed to connect to Cloudflare API${NC}"
        VALIDATION_PASSED=false
    fi
    
    # Test account access if we have Account ID
    if [ -z "$CLOUDFLARE_ACCOUNT_ID" ] && [ "$CHECK_1PASSWORD" = true ]; then
        if op account list &> /dev/null && op vault get "$VAULT_NAME" &> /dev/null; then
            CLOUDFLARE_ACCOUNT_ID=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/account_id" 2>/dev/null)
        fi
    fi
    
    if [ -n "$CLOUDFLARE_ACCOUNT_ID" ]; then
        log_verbose "Testing account access for: ${CLOUDFLARE_ACCOUNT_ID:0:8}...${CLOUDFLARE_ACCOUNT_ID: -4}"
        if ACCOUNT_RESPONSE=$(curl -s -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
                                   "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID"); then
            
            if echo "$ACCOUNT_RESPONSE" | grep -q '"success":true'; then
                echo -e "${GREEN}✅ Cloudflare account access confirmed${NC}"
            else
                echo -e "${RED}❌ Cannot access Cloudflare account${NC}"
                log_verbose "Account response: $ACCOUNT_RESPONSE"
                VALIDATION_PASSED=false
            fi
        else
            echo -e "${RED}❌ Failed to test account access${NC}"
            VALIDATION_PASSED=false
        fi
    fi
    
    return 0
}

# Display validation summary
show_summary() {
    echo ""
    echo -e "${BLUE}📋 Validation Summary${NC}"
    echo -e "${BLUE}=====================${NC}"
    
    if [ "$VALIDATION_PASSED" = true ]; then
        echo -e "${GREEN}✅ All validations passed!${NC}"
        echo ""
        echo -e "${YELLOW}🚀 Ready for deployment${NC}"
        echo "  Your secrets are properly configured and accessible."
        echo ""
        echo -e "${BLUE}Next Steps:${NC}"
        echo "  • Run deployment: ./scripts/deploy.sh production"
        echo "  • Test with 1Password: op run --env-file=.env.1password -- ./scripts/deploy.sh"
        echo "  • Sync to GitHub: ./scripts/sync-secrets-to-github.sh"
    else
        echo -e "${RED}❌ Some validations failed${NC}"
        echo ""
        echo -e "${YELLOW}🔧 Required Actions:${NC}"
        echo "  • Fix the issues mentioned above"
        echo "  • Run: ./scripts/setup-1password.sh (for 1Password setup)"
        echo "  • Check environment variables and .env file"
        echo "  • Verify API tokens and permissions"
        echo ""
        echo -e "${BLUE}For Help:${NC}"
        echo "  • Setup guide: ./docs/1PASSWORD_SETUP.md"
        echo "  • Deployment docs: ./DEPLOYMENT.md"
        echo "  • Run with --verbose for more details"
        
        exit 1
    fi
}

# Main execution
main() {
    validate_1password
    echo ""
    
    validate_environment
    echo ""
    
    validate_github_secrets
    echo ""
    
    validate_cloudflare_api
    echo ""
    
    show_summary
}

# Run main function
main "$@"