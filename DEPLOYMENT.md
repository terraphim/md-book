# MD-Book Cloudflare Deployment

This document provides comprehensive instructions for deploying MD-Book to Cloudflare Pages and Workers.

## Quick Start

### 1. Initial Setup

```bash
# Run the setup script to configure everything
./scripts/setup-cloudflare.sh
```

### 2. Configure Environment

#### Option A: 1Password Integration (Recommended)
```bash
# Set up 1Password integration (secure, team-friendly)
./scripts/setup-1password.sh

# Deploy with 1Password
op run --env-file=.env.1password -- ./scripts/deploy.sh production
```

#### Option B: Environment Variables (Traditional)
```bash
# Copy environment template
cp .env.example .env

# Set your Cloudflare credentials (get these from Cloudflare Dashboard)
export CLOUDFLARE_API_TOKEN="your-api-token-here"      # From API Tokens page
export CLOUDFLARE_ACCOUNT_ID="your-account-id-here"    # From Dashboard sidebar
```

**⚠️ Important**: Never commit `.env` file to git - it's already in `.gitignore`

**🔐 1Password Benefits**: Secure secret storage, team collaboration, audit trails, and automated rotation.

### 3. Deploy

#### With 1Password (Recommended)
```bash
# Deploy to production with 1Password
op run --env-file=.env.1password -- ./scripts/deploy.sh production

# Deploy to staging with 1Password  
op run --env-file=.env.1password -- ./scripts/deploy.sh staging

# Auto-detect 1Password (fallback to env vars if not available)
./scripts/deploy.sh production

# Force disable 1Password
USE_1PASSWORD=false ./scripts/deploy.sh production
```

#### With Environment Variables
```bash
# Deploy to production
./scripts/deploy.sh production

# Deploy to staging
./scripts/deploy.sh staging
```

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   GitHub Repo   │───▶│  GitHub Actions  │───▶│ Cloudflare Edge │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                    ┌──────────────────┐    ┌─────────────────────┐
                    │   Build Process  │    │  Cloudflare Pages   │
                    │                  │    │  (Static Hosting)   │
                    │ • Rust Compile   │    └─────────────────────┘
                    │ • Static Site    │                │
                    │ • Asset Optimize │                ▼
                    │ • Search Index   │    ┌─────────────────────┐
                    └──────────────────┘    │ Cloudflare Workers  │
                                            │  (API & Functions)  │
                                            └─────────────────────┘
```

## Components

### 1. Cloudflare Pages
- **Purpose**: Static site hosting with global CDN
- **Features**: Automatic HTTPS, instant deployments, preview deployments
- **Configuration**: `wrangler.toml`

### 2. Cloudflare Worker
- **Purpose**: API endpoints, redirects, and edge functions
- **Features**: Sub-millisecond response times, serverless scaling
- **Configuration**: `worker/wrangler.toml`

### 3. GitHub Actions
- **Purpose**: Automated CI/CD pipeline
- **Features**: Testing, building, deploying, monitoring
- **Configuration**: `.github/workflows/`

## Deployment Modes

### Production Deployment
- **Trigger**: Push to `main` branch
- **URL**: `https://md-book.pages.dev`
- **Features**: Full feature set, production optimizations
- **Workflow**: `.github/workflows/deploy.yml`

### Staging Deployment  
- **Trigger**: Manual deployment
- **URL**: `https://md-book-staging.pages.dev`
- **Features**: Same as production, used for testing
- **Command**: `./scripts/deploy.sh staging`

### Preview Deployments
- **Trigger**: Pull requests
- **URL**: `https://preview-{pr-number}.md-book.pages.dev`
- **Features**: Automatic preview for PR changes
- **Duration**: Temporary, removed when PR is closed

## Scripts Reference

### `./scripts/setup-cloudflare.sh`
Initial setup for Cloudflare deployment
- Installs required tools
- Configures environment
- Tests local build
- Provides GitHub Actions setup instructions

### `./scripts/setup-1password.sh`
1Password integration setup for secure secret management
- Installs and configures 1Password CLI
- Creates vault and items for Cloudflare secrets
- Tests secret retrieval
- Generates .env file from 1Password vault

### `./scripts/validate-secrets.sh`
Comprehensive secret validation
- Validates 1Password setup and access
- Checks environment variables
- Tests GitHub secrets (optional)
- Validates Cloudflare API connection (optional)

### `./scripts/sync-secrets-to-github.sh`
GitHub secrets synchronization from 1Password
- Retrieves secrets from 1Password vault
- Syncs to GitHub repository secrets
- Supports dry-run mode for testing

### `./scripts/deploy.sh [environment] [input_dir] [output_dir]`
Comprehensive deployment script
- Runs tests and builds
- Deploys to Cloudflare Pages
- Optionally deploys Worker
- Provides deployment summary

**Examples:**
```bash
# Traditional deployment
./scripts/deploy.sh production
./scripts/deploy.sh staging docs dist
SKIP_TESTS=true ./scripts/deploy.sh production
DEPLOY_WORKER=false ./scripts/deploy.sh production

# 1Password integration
op run --env-file=.env.1password -- ./scripts/deploy.sh production
USE_1PASSWORD=false ./scripts/deploy.sh production  # Force disable 1Password
```

### `./scripts/deploy-worker.sh [environment]`
Worker-specific deployment
- Validates Worker code
- Deploys to specified environment
- Shows deployment history

### `./scripts/monitor.sh [environment] [mode]`
Deployment monitoring and health checks
- Tests site availability
- Checks API endpoints
- Measures performance
- Validates SSL certificates

**Examples:**
```bash
./scripts/monitor.sh                    # Full check of all environments
./scripts/monitor.sh production         # Check production only
./scripts/monitor.sh production quick   # Quick health check
```

## Configuration

### Environment Variables

#### Required
```bash
CLOUDFLARE_API_TOKEN=your_cloudflare_api_token    # See "GitHub Secrets" section for details
CLOUDFLARE_ACCOUNT_ID=your_cloudflare_account_id  # See "GitHub Secrets" section for details
```

**How to Get These Values:**
- `CLOUDFLARE_API_TOKEN`: Create at [Cloudflare API Tokens](https://dash.cloudflare.com/profile/api-tokens)
- `CLOUDFLARE_ACCOUNT_ID`: Found in [Cloudflare Dashboard](https://dash.cloudflare.com/) sidebar

#### Optional
```bash
INPUT_DIR=test_input          # Source markdown directory
OUTPUT_DIR=dist               # Build output directory
SKIP_TESTS=false             # Skip tests during deployment
DEPLOY_WORKER=true           # Deploy worker with pages
```

### Cloudflare Configuration

#### `wrangler.toml` (Pages)
- Project configuration
- Security headers
- Caching rules
- Custom domains
- Redirects

#### `worker/wrangler.toml` (Worker)
- Worker configuration
- Route patterns
- Environment variables
- Resource limits

### GitHub Secrets

#### Required Secrets
Add these secrets to your GitHub repository (Settings → Secrets and variables → Actions):

**`CLOUDFLARE_API_TOKEN`**
- **Purpose**: Authenticates with Cloudflare API for deployments
- **Permissions Required**: `Cloudflare Pages:Edit`, `Zone:Read`, `Account:Read`
- **How to Get**: 
  1. Go to [Cloudflare Dashboard → My Profile → API Tokens](https://dash.cloudflare.com/profile/api-tokens)
  2. Click "Create Token"
  3. Use "Custom Token" template
  4. Set permissions: `Cloudflare Pages:Edit`, `Zone:Read`, `Account:Read`
  5. Optionally restrict to specific account/zones
  6. Copy the generated token (save it securely - you won't see it again)

**`CLOUDFLARE_ACCOUNT_ID`**
- **Purpose**: Identifies your Cloudflare account for resource management
- **How to Get**:
  1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com/)
  2. Select any domain or go to the main dashboard
  3. In the right sidebar, copy the "Account ID" value
  4. It looks like: `a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6`

#### Security Notes
- **Never commit these values** to your repository
- **Use minimal permissions** - only grant what's needed
- **Rotate tokens regularly** (every 90 days recommended)
- **Monitor token usage** in Cloudflare Dashboard → API Tokens

## 1Password Integration

MD-Book includes comprehensive 1Password CLI integration for secure secret management:

### Benefits
- **🔐 Secure Storage**: No secrets in plain text files or environment variables
- **👥 Team Collaboration**: Shared vaults for team deployments  
- **📊 Audit Trail**: Complete history of secret access
- **🔄 Easy Rotation**: Automatic propagation to all environments
- **🎯 Principle of Least Privilege**: Service accounts with minimal permissions

### Quick Start with 1Password
```bash
# 1. Set up 1Password integration
./scripts/setup-1password.sh

# 2. Deploy with 1Password
op run --env-file=.env.1password -- ./scripts/deploy.sh production

# 3. Sync secrets to GitHub for Actions
./scripts/sync-secrets-to-github.sh
```

### Workflows Supported
- **Local Development**: Use `op run` or `op inject` for seamless secret loading
- **GitHub Actions**: Automatic integration with 1Password service accounts
- **Team Deployment**: Shared vaults with role-based access
- **Secret Rotation**: Automated token rotation and propagation

### Files Created
- `.env.1password` - 1Password secret references template
- `scripts/setup-1password.sh` - Initial setup and vault creation
- `scripts/validate-secrets.sh` - Comprehensive secret validation
- `scripts/sync-secrets-to-github.sh` - GitHub secrets sync
- `docs/1PASSWORD_SETUP.md` - Complete setup documentation

See [1Password Setup Guide](./docs/1PASSWORD_SETUP.md) for detailed instructions.

## Features

### Performance Optimizations
- **Static Asset Caching**: 1-year cache for CSS/JS/images
- **HTML Revalidation**: Immediate content updates
- **Search Index Caching**: 1-hour cache for Pagefind
- **Edge Computing**: Worker functions run at 200+ locations

### Security Features
- **HTTPS Everywhere**: Automatic SSL/TLS certificates
- **Security Headers**: CSP, HSTS, XSS protection
- **Content Validation**: Input sanitization and validation
- **DDoS Protection**: Built-in Cloudflare protection

### Developer Experience
- **Live Previews**: Automatic PR deployments
- **Fast Deployments**: Typically complete in under 2 minutes
- **Rollback Support**: Easy deployment rollbacks
- **Comprehensive Monitoring**: Health checks and performance metrics

## API Endpoints (Worker)

### Health Check
```http
GET /api/health
```
Returns deployment status and version information.

### Search Suggestions
```http
GET /api/search/suggestions?q=query
```
Returns search suggestions based on the indexed content.

### Analytics Event
```http
POST /api/analytics/event
Content-Type: application/json

{
  "event": "page_view",
  "page": "/getting-started",
  "timestamp": "2024-09-06T12:00:00Z"
}
```

### Feedback Submission
```http
POST /api/feedback
Content-Type: application/json

{
  "message": "Great documentation!",
  "page": "/introduction",
  "rating": 5
}
```

## Troubleshooting

### Common Issues

#### Build Failures
```bash
# Check Rust installation
cargo --version
rustc --version

# Clean and rebuild
cargo clean
cargo build --release
```

#### Deployment Failures
```bash
# Verify environment variables
echo $CLOUDFLARE_API_TOKEN
echo $CLOUDFLARE_ACCOUNT_ID

# Check Wrangler authentication
wrangler whoami

# Manual deployment for debugging
wrangler pages deploy dist --project-name=md-book
```

#### Site Not Loading
```bash
# Check deployment status
wrangler pages deployment list --project-name=md-book

# Test API endpoints
curl -s https://md-book.pages.dev/api/health

# Run monitoring script
./scripts/monitor.sh production
```

### Debug Commands

```bash
# Test local build
cargo run -- -i test_input -o dist

# Validate worker locally
cd worker && wrangler dev

# Check DNS resolution
nslookup md-book.pages.dev

# Test SSL certificate
openssl s_client -servername md-book.pages.dev -connect md-book.pages.dev:443
```

## Performance Benchmarks

Expected performance metrics:
- **Page Load Time**: < 1.5s (globally)
- **First Contentful Paint**: < 0.8s
- **API Response Time**: < 100ms (from edge)
- **Build Time**: < 2 minutes (full deployment)

## Monitoring and Alerts

### Built-in Monitoring
- GitHub Actions deployment status
- Cloudflare analytics dashboard
- Worker execution metrics
- Pages performance insights

### Custom Monitoring
- Health check endpoints
- Performance monitoring script
- Error rate tracking
- User feedback collection

## Advanced Configuration

### Custom Domains
Add to `wrangler.toml`:
```toml
[pages.custom_domains]
production = "docs.yourdomain.com"
staging = "staging-docs.yourdomain.com"
```

### Additional Workers
Create new workers for specialized functionality:
```bash
mkdir additional-worker
cd additional-worker
wrangler init
```

### Environment-Specific Configuration
Use environment variables in `wrangler.toml`:
```toml
[env.production.vars]
API_URL = "https://api.yourdomain.com"

[env.staging.vars]
API_URL = "https://staging-api.yourdomain.com"
```

## Support and Resources

### Documentation
- [Cloudflare Pages Docs](https://developers.cloudflare.com/pages/)
- [Cloudflare Workers Docs](https://developers.cloudflare.com/workers/)
- [Wrangler CLI Reference](https://developers.cloudflare.com/workers/wrangler/)

### Community
- [Cloudflare Community](https://community.cloudflare.com/)
- [Cloudflare Discord](https://discord.gg/cloudflaredev)
- [GitHub Issues](https://github.com/terraphim/md-book/issues)

### Professional Support
- [Cloudflare Support](https://support.cloudflare.com/)
- [Enterprise Solutions](https://www.cloudflare.com/enterprise/)

---

*This deployment system is designed for high-performance, scalable documentation hosting with minimal maintenance overhead.*