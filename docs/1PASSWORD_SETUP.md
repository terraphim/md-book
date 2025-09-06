# 1Password Integration Setup Guide

This guide walks you through setting up 1Password CLI integration for secure secret management in your MD-Book deployment pipeline.

## Overview

1Password integration provides:
- **Secure secret storage** in 1Password vaults instead of plain text files
- **Automated secret loading** in deployment scripts and GitHub Actions
- **Team collaboration** through shared vaults
- **Audit trail** of all secret access
- **Easy rotation** with automatic propagation

## Prerequisites

### 1. 1Password Account
- Active 1Password account (Personal, Family, or Business)
- Access to create vaults and items
- For teams: Admin access to create service accounts

### 2. Required Tools
```bash
# Install 1Password CLI
brew install 1password-cli

# Install GitHub CLI (for secret sync)
brew install gh

# Verify installations
op --version
gh --version
```

## Quick Setup

### 1. Run Automated Setup
```bash
./scripts/setup-1password.sh
```

This script will:
- Verify 1Password CLI installation
- Check authentication
- Create vault and items if needed
- Test secret retrieval
- Generate .env file from 1Password

### 2. Local Development Usage
```bash
# Option 1: Use op run (recommended)
op run --env-file=.env.1password -- ./scripts/deploy.sh production

# Option 2: Generate .env file
op inject -i .env.1password -o .env
./scripts/deploy.sh production

# Option 3: Manual environment loading
source <(op inject -i .env.1password)
./scripts/deploy.sh production
```

### 3. GitHub Actions Setup
```bash
# Sync secrets to GitHub repository
./scripts/sync-secrets-to-github.sh

# Or with 1Password service account (recommended)
# Add OP_SERVICE_ACCOUNT_TOKEN to GitHub secrets
gh secret set OP_SERVICE_ACCOUNT_TOKEN --body "ops_..."
```

## Detailed Setup

### Step 1: 1Password Authentication

#### First-time Setup
```bash
# Add your account (if not already done)
op account add

# Sign in
op signin
```

#### Verify Authentication
```bash
# Check account status
op account list

# Test basic access
op vault list
```

### Step 2: Vault and Item Creation

#### Manual Vault Setup
```bash
# Create deployment vault
op vault create "MD-Book-Deployment" --description "MD-Book deployment secrets"

# Create Cloudflare item
op item create \
  --category="API Credential" \
  --title="Cloudflare" \
  --vault="MD-Book-Deployment" \
  --url="https://dash.cloudflare.com/" \
  "api_token[password]=your-cloudflare-api-token" \
  "account_id[text]=your-cloudflare-account-id"
```

#### Automated Setup (Recommended)
```bash
# Run the setup script
./scripts/setup-1password.sh

# Follow the prompts to enter your Cloudflare credentials
```

### Step 3: Vault Structure

Your 1Password vault should contain:

```
📁 MD-Book-Deployment/
  🔐 Cloudflare
    🔑 api_token (password field)
    📝 account_id (text field)
    🌐 website: https://dash.cloudflare.com/
  
  🔐 Domains (optional)
    📝 production: docs.yourdomain.com
    📝 staging: staging-docs.yourdomain.com
  
  🔐 GitHub (optional)
    🔑 personal_access_token
  
  🔐 Analytics (optional)
    🔑 token
    🔗 dsn
```

### Step 4: Secret References

The `.env.1password` file contains references to your vault:

```bash
# Format: op://vault-name/item-name/field-name
CLOUDFLARE_API_TOKEN="op://MD-Book-Deployment/Cloudflare/api_token"
CLOUDFLARE_ACCOUNT_ID="op://MD-Book-Deployment/Cloudflare/account_id"

# Optional custom domains
PRODUCTION_DOMAIN="op://MD-Book-Deployment/Domains/production"
STAGING_DOMAIN="op://MD-Book-Deployment/Domains/staging"
```

### Step 5: Local Development

#### Method 1: op run (Recommended)
```bash
# Deploy with secrets injected from 1Password
op run --env-file=.env.1password -- ./scripts/deploy.sh production

# Build only
op run --env-file=.env.1password -- cargo run -- -i docs -o dist

# Development server
op run --env-file=.env.1password -- cargo run -- -i docs -o dist --serve --watch
```

#### Method 2: Environment File Generation
```bash
# Generate .env from 1Password
op inject -i .env.1password -o .env

# Use normally (secrets now in .env)
./scripts/deploy.sh production

# Clean up (important!)
rm .env
```

#### Method 3: Shell Integration
```bash
# Load secrets into current shell
eval $(op inject -i .env.1password)

# Now run commands normally
./scripts/deploy.sh production
```

## GitHub Actions Integration

### Option 1: Service Account (Recommended for Teams)

#### 1. Create Service Account
In 1Password Business:
1. Go to Integrations → Service Accounts
2. Create new service account
3. Grant access to "MD-Book-Deployment" vault
4. Copy the service account token (`ops_...`)

#### 2. Add to GitHub Secrets
```bash
# Add service account token
gh secret set OP_SERVICE_ACCOUNT_TOKEN --body "ops_v1_..."

# The workflow will automatically load secrets from 1Password
```

#### 3. Workflow Configuration
The `.github/workflows/deploy.yml` automatically detects and uses 1Password:
```yaml
- name: Load secrets from 1Password
  uses: 1password/load-secrets-action@v2
  if: secrets.OP_SERVICE_ACCOUNT_TOKEN != ''
  with:
    export-env: true
  env:
    OP_SERVICE_ACCOUNT_TOKEN: ${{ secrets.OP_SERVICE_ACCOUNT_TOKEN }}
    CLOUDFLARE_API_TOKEN: "op://MD-Book-Deployment/Cloudflare/api_token"
    CLOUDFLARE_ACCOUNT_ID: "op://MD-Book-Deployment/Cloudflare/account_id"
```

### Option 2: Secret Sync (Alternative)

#### Manual Sync
```bash
# Sync secrets to GitHub repository secrets
./scripts/sync-secrets-to-github.sh

# Dry run to see what would be synced
./scripts/sync-secrets-to-github.sh --dry-run
```

#### Automated Sync
The repository includes a workflow that automatically syncs secrets weekly:
- Runs every Sunday at 2 AM UTC
- Can be triggered manually
- Creates issues on failure

## Validation and Testing

### Validate Setup
```bash
# Comprehensive validation
./scripts/validate-secrets.sh

# Verbose output
./scripts/validate-secrets.sh --verbose

# Check specific components
./scripts/validate-secrets.sh --check-cloudflare --check-github
```

### Test Deployment
```bash
# Test with 1Password integration
op run --env-file=.env.1password -- ./scripts/deploy.sh staging

# Test without 1Password (fallback to env vars)
USE_1PASSWORD=false ./scripts/deploy.sh staging
```

## Secret Rotation

### Cloudflare Token Rotation
```bash
# Automated rotation (coming soon)
./scripts/rotate-cloudflare-token.sh

# Manual rotation
# 1. Create new token in Cloudflare dashboard
# 2. Update 1Password item
op item edit "Cloudflare" --vault="MD-Book-Deployment" "api_token[password]=new-token"

# 3. Sync to GitHub (if using secret sync)
./scripts/sync-secrets-to-github.sh

# 4. Test deployment
./scripts/validate-secrets.sh --check-cloudflare
```

### Service Account Rotation
1. Generate new service account token in 1Password
2. Update GitHub secret:
   ```bash
   gh secret set OP_SERVICE_ACCOUNT_TOKEN --body "ops_new_token"
   ```
3. Revoke old service account token

## Troubleshooting

### Common Issues

#### "op: command not found"
```bash
# Install 1Password CLI
brew install 1password-cli

# Or download from: https://1password.com/downloads/command-line/
```

#### "User not signed in"
```bash
# Sign in to your account
op signin

# Or add account first
op account add
op signin
```

#### "Vault not found"
```bash
# List available vaults
op vault list

# Create the vault
op vault create "MD-Book-Deployment"

# Or run setup script
./scripts/setup-1password.sh
```

#### "Item not found"
```bash
# List items in vault
op item list --vault="MD-Book-Deployment"

# Create Cloudflare item
./scripts/setup-1password.sh
```

#### "Permission denied"
For team accounts:
- Ensure you have access to the vault
- Check if vault permissions allow item creation
- Contact your 1Password admin

#### GitHub Actions Failing
```bash
# Check if service account token is set
gh secret list | grep OP_SERVICE_ACCOUNT_TOKEN

# Verify token has vault access
# Check GitHub Actions logs for specific error
```

### Debug Commands

```bash
# Test 1Password connection
op account get
op vault get "MD-Book-Deployment"
op item get "Cloudflare" --vault="MD-Book-Deployment"

# Test secret reading
op read "op://MD-Book-Deployment/Cloudflare/api_token"

# Test GitHub CLI
gh auth status
gh secret list

# Test Cloudflare API
curl -H "Authorization: Bearer $(op read 'op://MD-Book-Deployment/Cloudflare/api_token')" \
  https://api.cloudflare.com/client/v4/user/tokens/verify

# Validate entire setup
./scripts/validate-secrets.sh --verbose --check-github --check-cloudflare
```

## Security Best Practices

### 1Password Security
- **Use strong master password** and 2FA on 1Password account
- **Enable Travel Mode** when traveling
- **Regularly audit** vault access logs
- **Use service accounts** for automation instead of personal accounts
- **Limit vault access** to only necessary team members

### Secret Management
- **Never commit** secrets to Git repositories
- **Rotate secrets regularly** (every 90 days recommended)
- **Use minimal permissions** for API tokens
- **Monitor secret usage** in audit logs
- **Clean up unused secrets**

### Development Workflow
- **Always use** `op run` or `op inject` instead of storing secrets in files
- **Delete .env files** after use or add to .gitignore
- **Use different tokens** for development and production
- **Validate secrets** before deployment

### Team Collaboration
- **Use shared vaults** for team secrets
- **Document secret locations** in team wiki
- **Train team members** on 1Password best practices
- **Implement approval processes** for production secret changes

## Advanced Configuration

### Custom Vault Names
If you use different vault names, update these files:
- `.env.1password` - Update secret references
- `scripts/setup-1password.sh` - Update VAULT_NAME variable
- `scripts/sync-secrets-to-github.sh` - Update VAULT_NAME variable
- `.github/workflows/deploy.yml` - Update secret references

### Multiple Environments
Create separate items or vaults for different environments:

```bash
# Staging secrets
op item create \
  --title="Cloudflare-Staging" \
  --vault="MD-Book-Deployment" \
  "api_token[password]=staging-token" \
  "account_id[text]=staging-account"

# Update .env.1password for staging
CLOUDFLARE_API_TOKEN="op://MD-Book-Deployment/Cloudflare-Staging/api_token"
```

### Integration with Other Tools
- **Docker**: Use `op run` to inject secrets into containers
- **Terraform**: Use 1Password Terraform provider
- **Kubernetes**: Use 1Password Secrets Injector
- **CI/CD**: Use 1Password Actions for other platforms

## Support and Resources

### Documentation
- [1Password CLI Documentation](https://developer.1password.com/docs/cli/)
- [1Password GitHub Actions](https://github.com/1Password/load-secrets-action)
- [1Password Service Accounts](https://support.1password.com/service-accounts/)

### Community
- [1Password Community](https://1password.community/)
- [1Password CLI GitHub](https://github.com/1Password/cli)
- [Project Issues](https://github.com/terraphim/md-book/issues)

### Professional Support
- [1Password Business Support](https://support.1password.com/contact/)
- [Deployment Documentation](../DEPLOYMENT.md)
- [Cloudflare Setup Guide](../CLOUDFLARE_SETUP.md)

---

*This setup enables secure, scalable secret management for your MD-Book deployment pipeline with industry-standard security practices.*