#!/bin/bash

# GitHub Secrets Sync Script
# Syncs secrets from 1Password vault to GitHub repository secrets
# This allows GitHub Actions to access secrets without storing them directly

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VAULT_NAME="TerraphimPlatform"
CLOUDFLARE_ITEM="md-book-cloudflare"
REPO_NAME=""
DRY_RUN=false

echo -e "${BLUE}🔄 GitHub Secrets Sync${NC}"
echo -e "${BLUE}=====================${NC}"
echo ""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            echo -e "${YELLOW}🧪 Running in dry-run mode - no changes will be made${NC}"
            shift
            ;;
        --repo)
            REPO_NAME="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --dry-run    Show what would be done without making changes"
            echo "  --repo REPO  Specify repository (owner/repo format)"
            echo "  --help       Show this help message"
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
    
    # Check 1Password CLI
    if ! command -v op &> /dev/null; then
        echo -e "${RED}❌ 1Password CLI not found${NC}"
        echo "  Install with: brew install 1password-cli"
        exit 1
    fi
    
    # Check GitHub CLI
    if ! command -v gh &> /dev/null; then
        echo -e "${RED}❌ GitHub CLI not found${NC}"
        echo "  Install with: brew install gh"
        exit 1
    fi
    
    # Check 1Password authentication
    if ! op account list &> /dev/null; then
        echo -e "${RED}❌ Not authenticated with 1Password${NC}"
        echo "  Run: op signin"
        exit 1
    fi
    
    # Check GitHub authentication
    if ! gh auth status &> /dev/null; then
        echo -e "${RED}❌ Not authenticated with GitHub${NC}"
        echo "  Run: gh auth login"
        exit 1
    fi
    
    echo -e "${GREEN}✅ All prerequisites met${NC}"
}

# Detect repository
detect_repository() {
    if [ -z "$REPO_NAME" ]; then
        # Try to detect from git remote
        if git remote -v &> /dev/null; then
            REPO_URL=$(git remote get-url origin 2>/dev/null || echo "")
            if [[ $REPO_URL =~ github\.com[:/]([^/]+/[^/]+) ]]; then
                REPO_NAME="${BASH_REMATCH[1]}"
                # Remove .git suffix if present
                REPO_NAME="${REPO_NAME%.git}"
            fi
        fi
    fi
    
    if [ -z "$REPO_NAME" ]; then
        echo -e "${RED}❌ Could not detect repository${NC}"
        echo "  Use --repo owner/repository to specify manually"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Repository: $REPO_NAME${NC}"
}

# Check vault access
check_vault_access() {
    echo -e "${YELLOW}🔍 Checking 1Password vault access...${NC}"
    
    if ! op vault get "$VAULT_NAME" &> /dev/null; then
        echo -e "${RED}❌ Cannot access vault: $VAULT_NAME${NC}"
        echo "  Run: ./scripts/setup-1password.sh"
        exit 1
    fi
    
    if ! op item get "$CLOUDFLARE_ITEM" --vault="$VAULT_NAME" &> /dev/null; then
        echo -e "${RED}❌ Cannot access Cloudflare item in vault${NC}"
        echo "  Run: ./scripts/setup-1password.sh"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Vault access confirmed${NC}"
}

# Retrieve secrets from 1Password
retrieve_secrets() {
    echo -e "${YELLOW}🔍 Retrieving secrets from 1Password...${NC}"
    
    # Retrieve Cloudflare API Token
    if ! API_TOKEN=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/api_token" 2>/dev/null); then
        echo -e "${RED}❌ Failed to retrieve Cloudflare API token${NC}"
        exit 1
    fi
    
    # Retrieve Cloudflare Account ID
    if ! ACCOUNT_ID=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/account_id" 2>/dev/null); then
        echo -e "${RED}❌ Failed to retrieve Cloudflare Account ID${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Secrets retrieved successfully${NC}"
    echo "  API Token: ${API_TOKEN:0:10}..."
    echo "  Account ID: ${ACCOUNT_ID:0:8}...${ACCOUNT_ID: -4}"
}

# Sync secret to GitHub
sync_secret() {
    local secret_name="$1"
    local secret_value="$2"
    local description="$3"
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${BLUE}[DRY RUN]${NC} Would sync: $secret_name"
        return 0
    fi
    
    echo -e "${YELLOW}🔄 Syncing $secret_name...${NC}"
    
    # Use GitHub CLI to set the secret
    if echo "$secret_value" | gh secret set "$secret_name" --repo="$REPO_NAME"; then
        echo -e "${GREEN}✅ $secret_name synced successfully${NC}"
        return 0
    else
        echo -e "${RED}❌ Failed to sync $secret_name${NC}"
        return 1
    fi
}

# List current GitHub secrets
list_github_secrets() {
    echo -e "${YELLOW}🔍 Current GitHub secrets:${NC}"
    if gh secret list --repo="$REPO_NAME" 2>/dev/null; then
        echo ""
    else
        echo -e "${YELLOW}  Unable to list secrets (may require admin permissions)${NC}"
        echo ""
    fi
}

# Validate synced secrets
validate_synced_secrets() {
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN] Skipping validation${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🔍 Validating synced secrets...${NC}"
    
    # Check if secrets exist (we can't read their values for security)
    if gh secret list --repo="$REPO_NAME" | grep -q "CLOUDFLARE_API_TOKEN"; then
        echo -e "${GREEN}✅ CLOUDFLARE_API_TOKEN is set${NC}"
    else
        echo -e "${RED}❌ CLOUDFLARE_API_TOKEN not found${NC}"
        return 1
    fi
    
    if gh secret list --repo="$REPO_NAME" | grep -q "CLOUDFLARE_ACCOUNT_ID"; then
        echo -e "${GREEN}✅ CLOUDFLARE_ACCOUNT_ID is set${NC}"
    else
        echo -e "${RED}❌ CLOUDFLARE_ACCOUNT_ID not found${NC}"
        return 1
    fi
    
    return 0
}

# Show next steps
show_next_steps() {
    echo ""
    echo -e "${BLUE}🎉 Secret sync complete!${NC}"
    echo ""
    echo -e "${YELLOW}📋 Next Steps:${NC}"
    echo "  1. Your GitHub Actions workflows can now access:"
    echo "     - CLOUDFLARE_API_TOKEN"
    echo "     - CLOUDFLARE_ACCOUNT_ID"
    echo ""
    echo "  2. Test the deployment workflow:"
    echo "     git push origin main"
    echo ""
    echo "  3. Or trigger manually:"
    echo "     gh workflow run deploy.yml"
    echo ""
    echo -e "${YELLOW}📝 Notes:${NC}"
    echo "  - Secrets are encrypted and only visible to workflows"
    echo "  - Re-run this script when you rotate secrets in 1Password"
    echo "  - Use './scripts/rotate-cloudflare-token.sh' for automated rotation"
    echo ""
    echo -e "${YELLOW}🔍 Monitor:${NC}"
    echo "  - Workflow runs: gh run list"
    echo "  - Secret usage: GitHub repository settings > Secrets"
}

# Main execution
main() {
    check_prerequisites
    detect_repository
    check_vault_access
    list_github_secrets
    retrieve_secrets
    
    echo ""
    echo -e "${BLUE}🔄 Syncing secrets to GitHub...${NC}"
    
    # Sync each secret
    SYNC_SUCCESS=true
    
    if ! sync_secret "CLOUDFLARE_API_TOKEN" "$API_TOKEN" "Cloudflare API Token for deployment"; then
        SYNC_SUCCESS=false
    fi
    
    if ! sync_secret "CLOUDFLARE_ACCOUNT_ID" "$ACCOUNT_ID" "Cloudflare Account ID for deployment"; then
        SYNC_SUCCESS=false
    fi
    
    if [ "$SYNC_SUCCESS" = true ]; then
        validate_synced_secrets
        show_next_steps
    else
        echo -e "${RED}❌ Some secrets failed to sync${NC}"
        exit 1
    fi
}

# Handle direct execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi