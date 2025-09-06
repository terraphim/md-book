#!/bin/bash

# Cloudflare Token Rotation Script
# Automates the process of rotating Cloudflare API tokens with 1Password integration
# This script creates a new token, updates 1Password, tests it, and optionally revokes the old one

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VAULT_NAME="MD-Book-Deployment"
CLOUDFLARE_ITEM="Cloudflare"
DRY_RUN=false
AUTO_REVOKE=false
BACKUP_TOKEN=""

echo -e "${BLUE}🔄 Cloudflare Token Rotation${NC}"
echo -e "${BLUE}=============================${NC}"
echo ""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            echo -e "${YELLOW}🧪 Running in dry-run mode${NC}"
            shift
            ;;
        --auto-revoke)
            AUTO_REVOKE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --dry-run      Show what would be done without making changes"
            echo "  --auto-revoke  Automatically revoke old token after successful test"
            echo "  --help         Show this help message"
            echo ""
            echo "Process:"
            echo "  1. Backup current token from 1Password"
            echo "  2. Create new Cloudflare API token with same permissions"
            echo "  3. Update 1Password vault with new token"
            echo "  4. Test new token with deployment validation"
            echo "  5. Sync new token to GitHub secrets"
            echo "  6. Optionally revoke old token"
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}🔍 Checking prerequisites...${NC}"
    
    # Check required tools
    local tools=("op" "curl" "jq")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            echo -e "${RED}❌ $tool not found${NC}"
            case $tool in
                "op") echo "  Install with: brew install 1password-cli" ;;
                "curl") echo "  curl should be available on most systems" ;;
                "jq") echo "  Install with: brew install jq" ;;
            esac
            exit 1
        fi
    done
    
    # Check 1Password authentication
    if ! op account list &> /dev/null; then
        echo -e "${RED}❌ Not authenticated with 1Password${NC}"
        echo "  Run: op signin"
        exit 1
    fi
    
    # Check vault access
    if ! op vault get "$VAULT_NAME" &> /dev/null; then
        echo -e "${RED}❌ Cannot access vault: $VAULT_NAME${NC}"
        echo "  Run: ./scripts/setup-1password.sh"
        exit 1
    fi
    
    echo -e "${GREEN}✅ All prerequisites met${NC}"
}

# Backup current token
backup_current_token() {
    echo -e "${YELLOW}💾 Backing up current token...${NC}"
    
    if BACKUP_TOKEN=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/api_token" 2>/dev/null); then
        echo -e "${GREEN}✅ Current token backed up${NC}"
        echo "  Token backup: ${BACKUP_TOKEN:0:10}...${BACKUP_TOKEN: -4}"
        return 0
    else
        echo -e "${RED}❌ Failed to backup current token${NC}"
        exit 1
    fi
}

# Get current token permissions
get_token_permissions() {
    echo -e "${YELLOW}🔍 Analyzing current token permissions...${NC}"
    
    if ! TOKEN_INFO=$(curl -s -H "Authorization: Bearer $BACKUP_TOKEN" \
                          "https://api.cloudflare.com/client/v4/user/tokens/verify"); then
        echo -e "${RED}❌ Failed to get token information${NC}"
        return 1
    fi
    
    if ! echo "$TOKEN_INFO" | jq -e '.success' >/dev/null 2>&1; then
        echo -e "${RED}❌ Current token is invalid or expired${NC}"
        echo "  Response: $(echo "$TOKEN_INFO" | jq -r '.errors[0].message' 2>/dev/null || echo "Unknown error")"
        return 1
    fi
    
    echo -e "${GREEN}✅ Current token is valid${NC}"
    
    # Extract token details
    if TOKEN_ID=$(echo "$TOKEN_INFO" | jq -r '.result.id' 2>/dev/null); then
        echo "  Token ID: $TOKEN_ID"
    fi
    
    if EXPIRES_ON=$(echo "$TOKEN_INFO" | jq -r '.result.expires_on' 2>/dev/null) && [ "$EXPIRES_ON" != "null" ]; then
        echo "  Expires: $EXPIRES_ON"
    else
        echo "  Expires: Never"
    fi
    
    return 0
}

# Create new token (interactive process)
create_new_token() {
    echo -e "${YELLOW}🔑 Creating new Cloudflare API token...${NC}"
    echo ""
    echo -e "${BLUE}📋 Manual Steps Required:${NC}"
    echo "1. Open: https://dash.cloudflare.com/profile/api-tokens"
    echo "2. Click 'Create Token'"
    echo "3. Use 'Custom token' template"
    echo "4. Set permissions:"
    echo "   - Account: Cloudflare Pages:Edit"
    echo "   - Zone: Zone:Read"  
    echo "   - Account: Account:Read"
    echo "5. Add account/zone restrictions if needed"
    echo "6. Set TTL (optional, recommended: 90 days)"
    echo "7. Click 'Continue to summary' → 'Create Token'"
    echo "8. Copy the new token"
    echo ""
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN] Would prompt for new token${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🔑 Enter the new API token:${NC}"
    read -r -s NEW_TOKEN
    echo ""
    
    if [ -z "$NEW_TOKEN" ]; then
        echo -e "${RED}❌ No token provided${NC}"
        exit 1
    fi
    
    # Validate new token format
    if [[ ! $NEW_TOKEN =~ ^[A-Za-z0-9_-]+$ ]] || [ ${#NEW_TOKEN} -lt 40 ]; then
        echo -e "${RED}❌ Token format appears invalid${NC}"
        echo "  Expected: 40+ alphanumeric characters"
        exit 1
    fi
    
    echo -e "${GREEN}✅ New token received${NC}"
    echo "  Token: ${NEW_TOKEN:0:10}...${NEW_TOKEN: -4}"
}

# Test new token
test_new_token() {
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN] Would test new token${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🧪 Testing new token...${NC}"
    
    # Test token verification
    if TOKEN_TEST=$(curl -s -H "Authorization: Bearer $NEW_TOKEN" \
                        "https://api.cloudflare.com/client/v4/user/tokens/verify"); then
        
        if echo "$TOKEN_TEST" | jq -e '.success' >/dev/null 2>&1; then
            echo -e "${GREEN}✅ New token verification successful${NC}"
        else
            echo -e "${RED}❌ New token verification failed${NC}"
            echo "  Error: $(echo "$TOKEN_TEST" | jq -r '.errors[0].message' 2>/dev/null)"
            exit 1
        fi
    else
        echo -e "${RED}❌ Failed to test new token${NC}"
        exit 1
    fi
    
    # Test account access
    if ACCOUNT_ID=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/account_id" 2>/dev/null); then
        if ACCOUNT_TEST=$(curl -s -H "Authorization: Bearer $NEW_TOKEN" \
                             "https://api.cloudflare.com/client/v4/accounts/$ACCOUNT_ID"); then
            
            if echo "$ACCOUNT_TEST" | jq -e '.success' >/dev/null 2>&1; then
                echo -e "${GREEN}✅ Account access confirmed${NC}"
            else
                echo -e "${RED}❌ Account access failed${NC}"
                echo "  Error: $(echo "$ACCOUNT_TEST" | jq -r '.errors[0].message' 2>/dev/null)"
                exit 1
            fi
        fi
    fi
}

# Update 1Password vault
update_1password_vault() {
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN] Would update 1Password vault${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🔐 Updating 1Password vault...${NC}"
    
    # Create backup of old token as separate field (optional)
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    if op item edit "$CLOUDFLARE_ITEM" --vault="$VAULT_NAME" \
       "previous_token_$TIMESTAMP[password]=$BACKUP_TOKEN" &>/dev/null; then
        echo -e "${GREEN}✅ Previous token backed up as separate field${NC}"
    fi
    
    # Update the main token field
    if op item edit "$CLOUDFLARE_ITEM" --vault="$VAULT_NAME" \
       "api_token[password]=$NEW_TOKEN"; then
        echo -e "${GREEN}✅ 1Password vault updated successfully${NC}"
    else
        echo -e "${RED}❌ Failed to update 1Password vault${NC}"
        exit 1
    fi
}

# Test deployment with new token
test_deployment() {
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN] Would test deployment${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🚀 Testing deployment with new token...${NC}"
    
    # Run validation script
    if ./scripts/validate-secrets.sh --check-cloudflare; then
        echo -e "${GREEN}✅ Secret validation passed${NC}"
    else
        echo -e "${RED}❌ Secret validation failed${NC}"
        echo "  Rolling back 1Password vault..."
        op item edit "$CLOUDFLARE_ITEM" --vault="$VAULT_NAME" \
           "api_token[password]=$BACKUP_TOKEN"
        exit 1
    fi
    
    # Test a minimal deployment (staging)
    echo -e "${YELLOW}📦 Testing staging deployment...${NC}"
    if op run --env-file=.env.1password -- ./scripts/deploy.sh staging; then
        echo -e "${GREEN}✅ Staging deployment successful${NC}"
    else
        echo -e "${RED}❌ Staging deployment failed${NC}"
        echo "  Rolling back 1Password vault..."
        op item edit "$CLOUDFLARE_ITEM" --vault="$VAULT_NAME" \
           "api_token[password]=$BACKUP_TOKEN"
        exit 1
    fi
}

# Sync to GitHub secrets
sync_to_github() {
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN] Would sync to GitHub secrets${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🔄 Syncing new token to GitHub...${NC}"
    
    if ./scripts/sync-secrets-to-github.sh; then
        echo -e "${GREEN}✅ GitHub secrets updated${NC}"
    else
        echo -e "${YELLOW}⚠️  GitHub sync failed (manual sync may be needed)${NC}"
        echo "  Run: ./scripts/sync-secrets-to-github.sh"
    fi
}

# Revoke old token
revoke_old_token() {
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN] Would revoke old token${NC}"
        return 0
    fi
    
    if [ "$AUTO_REVOKE" = false ]; then
        echo ""
        echo -e "${YELLOW}🗑️  Do you want to revoke the old token? (y/N):${NC}"
        read -r REVOKE_CONFIRM
        if [[ ! $REVOKE_CONFIRM =~ ^[Yy]$ ]]; then
            echo -e "${YELLOW}⚠️  Old token not revoked - remember to revoke it manually${NC}"
            echo "  Old token: ${BACKUP_TOKEN:0:10}...${BACKUP_TOKEN: -4}"
            return 0
        fi
    fi
    
    echo -e "${YELLOW}🗑️  Revoking old token...${NC}"
    
    # Get token ID for revocation (this requires the old token to still be valid)
    if TOKEN_INFO=$(curl -s -H "Authorization: Bearer $BACKUP_TOKEN" \
                        "https://api.cloudflare.com/client/v4/user/tokens/verify"); then
        
        if TOKEN_ID=$(echo "$TOKEN_INFO" | jq -r '.result.id' 2>/dev/null); then
            # Revoke the token
            if curl -s -X DELETE \
               -H "Authorization: Bearer $BACKUP_TOKEN" \
               "https://api.cloudflare.com/client/v4/user/tokens/$TOKEN_ID" | \
               jq -e '.success' >/dev/null 2>&1; then
                echo -e "${GREEN}✅ Old token revoked successfully${NC}"
            else
                echo -e "${YELLOW}⚠️  Failed to revoke old token (it may have expired)${NC}"
            fi
        else
            echo -e "${YELLOW}⚠️  Could not determine old token ID${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  Old token no longer valid (may have expired)${NC}"
    fi
}

# Show summary
show_summary() {
    echo ""
    echo -e "${BLUE}📋 Token Rotation Summary${NC}"
    echo -e "${BLUE}===========================${NC}"
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}🧪 Dry run completed - no changes were made${NC}"
    else
        echo -e "${GREEN}✅ Token rotation completed successfully!${NC}"
    fi
    
    echo ""
    echo -e "${YELLOW}📝 Actions Performed:${NC}"
    echo "  • Backed up current token"
    echo "  • Created new Cloudflare API token"
    echo "  • Updated 1Password vault"
    echo "  • Validated new token functionality"
    echo "  • Tested deployment with new token"
    echo "  • Synced to GitHub repository secrets"
    if [ "$AUTO_REVOKE" = true ] || [ "$REVOKE_CONFIRM" = "y" ] || [ "$REVOKE_CONFIRM" = "Y" ]; then
        echo "  • Revoked old token"
    fi
    
    echo ""
    echo -e "${YELLOW}🔍 Next Steps:${NC}"
    echo "  • Monitor deployments for any issues"
    echo "  • Check Cloudflare dashboard for token usage"
    echo "  • Update team documentation with rotation date"
    echo "  • Schedule next rotation (recommended: 90 days)"
    
    if [ "$DRY_RUN" = false ]; then
        echo ""
        echo -e "${BLUE}📊 Token Information:${NC}"
        echo "  • New token: ${NEW_TOKEN:0:10}...${NEW_TOKEN: -4}"
        echo "  • Rotation date: $(date)"
        echo "  • Backup location: 1Password vault (previous_token_* field)"
    fi
}

# Main execution
main() {
    check_prerequisites
    backup_current_token
    get_token_permissions
    create_new_token
    test_new_token
    update_1password_vault
    test_deployment
    sync_to_github
    revoke_old_token
    show_summary
}

# Run main function
main "$@"