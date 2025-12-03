# MD-Book Deployment Guide

This document provides comprehensive instructions for deploying MD-Book to various hosting platforms including Cloudflare Pages, Workers, and Netlify.

## Deployment Options

MD-Book supports multiple hosting platforms:

- **[Cloudflare Pages](#cloudflare-deployment)** - Unlimited bandwidth, Workers integration, advanced caching
- **[Netlify](#netlify-deployment)** - Simple deployment, forms, branch previews, generous free tier
- **[Vercel](#vercel-deployment)** - Zero-config deployments, edge functions, serverless integration
- **[GitHub Pages](#github-pages-deployment)** - Free hosting for public repos, Jekyll-free static sites
- **[AWS Amplify](#aws-amplify-deployment)** - AWS integration, serverless backend, custom domains
- **[Render](#render-deployment)** - Free SSL, auto-deploy from Git, DDoS protection
- **[Railway](#railway-deployment)** - Simple deployments, automatic HTTPS, preview environments
- **[Fly.io](#flyio-deployment)** - Edge deployment, global distribution, Dockerfile support
- **[DigitalOcean App Platform](#digitalocean-app-platform-deployment)** - Simple pricing, managed infrastructure

## Quick Start (Cloudflare)

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

## Cloudflare Deployment

### Architecture Overview

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

## Netlify Deployment

MD-Book can also be deployed to Netlify, which offers a simple drag-and-drop deployment option and Git-based continuous deployment.

### Quick Netlify Deployment

#### Option 1: Drag and Drop
1. **Build your site locally:**
   ```bash
   # Build with default input/output
   cargo run -- -i docs -o dist
   
   # Or use custom directories
   cargo run -- -i your-content -o build
   ```

2. **Deploy to Netlify:**
   - Go to [Netlify Drop](https://app.netlify.com/drop)
   - Drag your `dist` (or `build`) folder to the deploy area
   - Your site will be live instantly with a random URL
   - Optionally set a custom subdomain

#### Option 2: Git-based Deployment
1. **Connect your repository:**
   - Go to [Netlify](https://app.netlify.com/)
   - Click "Add new site" → "Import an existing project"
   - Connect your Git provider and select your repository

2. **Configure build settings:**
   ```yaml
   # Build command
   cargo run -- -i docs -o dist
   
   # Publish directory  
   dist
   
   # Environment variables (optional)
   RUST_VERSION=1.70.0
   ```

#### Option 3: Netlify CLI
```bash
# Install Netlify CLI
npm install -g netlify-cli

# Login to Netlify
netlify login

# Build your site
cargo run -- -i docs -o dist

# Deploy to draft URL
netlify deploy --dir=dist

# Deploy to production
netlify deploy --prod --dir=dist
```

### Netlify Configuration File

Create `netlify.toml` in your project root:

```toml
[build]
  command = "cargo run -- -i docs -o dist"
  publish = "dist"

[build.environment]
  RUST_VERSION = "1.70.0"

# Redirect rules for SPA-like behavior
[[redirects]]
  from = "/*"
  to = "/404.html"
  status = 404

# Headers for better caching
[[headers]]
  for = "/css/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000"

[[headers]]
  for = "/js/*"  
  [headers.values]
    Cache-Control = "public, max-age=31536000"

[[headers]]
  for = "*.html"
  [headers.values]
    Cache-Control = "public, max-age=0, must-revalidate"

# Enable Netlify Functions (optional)
[functions]
  directory = "netlify/functions"
```

### GitHub Actions for Netlify

Create `.github/workflows/netlify-deploy.yml`:

```yaml
name: Deploy to Netlify

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: 1.70.0
          
      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Build site
        run: |
          cargo run -- -i docs -o dist
          
      - name: Deploy to Netlify
        uses: nwtgck/actions-netlify@v3.0
        with:
          publish-dir: './dist'
          production-branch: main
          github-token: ${{ secrets.GITHUB_TOKEN }}
          deploy-message: "Deploy from GitHub Actions"
        env:
          NETLIFY_AUTH_TOKEN: ${{ secrets.NETLIFY_AUTH_TOKEN }}
          NETLIFY_SITE_ID: ${{ secrets.NETLIFY_SITE_ID }}
```

### Netlify Features for MD-Book

#### Form Handling
Add contact forms to your documentation:

```html
<!-- In your markdown or template -->
<form name="feedback" method="POST" data-netlify="true">
  <p>
    <label>Name: <input type="text" name="name" required /></label>
  </p>
  <p>
    <label>Email: <input type="email" name="email" required /></label>
  </p>
  <p>
    <label>Message: <textarea name="message" required></textarea></label>
  </p>
  <p>
    <button type="submit">Send Feedback</button>
  </p>
</form>
```

#### Branch Previews
- Every pull request gets a preview URL
- Perfect for reviewing documentation changes
- Automatic cleanup when PR is merged/closed

#### Custom Domains
```toml
# In netlify.toml
[build]
  command = "cargo run -- -i docs -o dist"
  publish = "dist"

# Custom domain configuration
[[headers]]
  for = "/*"
  [headers.values]
    X-Frame-Options = "DENY"
    X-XSS-Protection = "1; mode=block"
    X-Content-Type-Options = "nosniff"
    Referrer-Policy = "strict-origin-when-cross-origin"
```

### Netlify vs Cloudflare Comparison

| Feature | Netlify | Cloudflare Pages |
|---------|---------|-----------------|
| **Deployment** | Git-based, CLI, Drag & Drop | Git-based, CLI, API |
| **Build Time** | 15min free tier | Unlimited |
| **Bandwidth** | 100GB/month free | Unlimited |
| **Forms** | Built-in form handling | Requires Workers |
| **Functions** | Netlify Functions | Cloudflare Workers |
| **CDN** | Global CDN | Global CDN (faster) |
| **Custom Headers** | Via netlify.toml | Via _headers file |
| **Redirects** | Built-in | Built-in |
| **Branch Previews** | ✅ | ✅ |
| **Custom Domains** | Free SSL | Free SSL |
| **Analytics** | Paid add-on | Built-in |

### Migration from Cloudflare to Netlify

If you want to migrate from Cloudflare to Netlify:

1. **Export your build:**
   ```bash
   # Build locally
   cargo run -- -i docs -o dist
   ```

2. **Create netlify.toml:**
   ```toml
   [build]
     command = "cargo run -- -i docs -o dist"
     publish = "dist"
   ```

3. **Set up redirects:**
   ```toml
   # Convert Cloudflare _redirects to netlify.toml
   [[redirects]]
     from = "/old-path/*"
     to = "/new-path/:splat"
     status = 301
   ```

4. **Deploy:**
   ```bash
   netlify init
   netlify deploy --prod
   ```

## Vercel Deployment

Vercel offers zero-configuration deployments with automatic HTTPS and global CDN.

### Quick Vercel Deployment

#### Option 1: Vercel CLI
```bash
# Install Vercel CLI
npm install -g vercel

# Build your site
cargo run -- -i docs -o dist

# Deploy to Vercel
vercel --prod

# Or deploy with custom configuration
vercel deploy --prod --name md-book
```

#### Option 2: Git Integration
1. **Connect repository:**
   - Go to [Vercel Dashboard](https://vercel.com/dashboard)
   - Click "Add New" → "Project"
   - Import your Git repository

2. **Configure build:**
   - **Framework Preset**: Other
   - **Build Command**: `cargo run -- -i docs -o dist`
   - **Output Directory**: `dist`
   - **Install Command**: Leave empty (Rust pre-installed)

3. **Deploy:**
   - Click "Deploy"
   - Your site will be live in minutes

#### Option 3: Manual Upload
```bash
# Build locally
cargo run -- -i docs -o dist

# Deploy with Vercel CLI
cd dist && vercel --prod
```

### Vercel Configuration

Create `vercel.json` in your project root:

```json
{
  "version": 2,
  "name": "md-book",
  "builds": [
    {
      "src": "package.json",
      "use": "@vercel/static-build",
      "config": {
        "distDir": "dist"
      }
    }
  ],
  "buildCommand": "cargo run -- -i docs -o dist",
  "devCommand": "cargo run -- -i docs -o dist --serve",
  "installCommand": "echo 'Rust is pre-installed'",
  "outputDirectory": "dist",
  "routes": [
    {
      "src": "/(.*)",
      "dest": "/$1",
      "status": 200
    }
  ],
  "headers": [
    {
      "source": "/css/(.*)",
      "headers": [
        {
          "key": "Cache-Control",
          "value": "public, max-age=31536000, immutable"
        }
      ]
    },
    {
      "source": "/js/(.*)",
      "headers": [
        {
          "key": "Cache-Control",
          "value": "public, max-age=31536000, immutable"
        }
      ]
    },
    {
      "source": "/(.*).html",
      "headers": [
        {
          "key": "Cache-Control",
          "value": "public, max-age=0, must-revalidate"
        }
      ]
    },
    {
      "source": "/(.*)",
      "headers": [
        {
          "key": "X-Content-Type-Options",
          "value": "nosniff"
        },
        {
          "key": "X-Frame-Options",
          "value": "DENY"
        },
        {
          "key": "X-XSS-Protection",
          "value": "1; mode=block"
        }
      ]
    }
  ],
  "rewrites": [
    {
      "source": "/:path*",
      "destination": "/:path*"
    }
  ]
}
```

### GitHub Actions for Vercel

Create `.github/workflows/vercel-deploy.yml`:

```yaml
name: Deploy to Vercel

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]

jobs:
  deploy:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: 1.70.0

      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Build site
        run: cargo run -- -i docs -o dist

      - name: Deploy to Vercel
        uses: amondnet/vercel-action@v25
        with:
          vercel-token: ${{ secrets.VERCEL_TOKEN }}
          vercel-org-id: ${{ secrets.VERCEL_ORG_ID }}
          vercel-project-id: ${{ secrets.VERCEL_PROJECT_ID }}
          vercel-args: '--prod'
          working-directory: ./dist
```

### Required Vercel Secrets

Add to GitHub repository secrets:
- `VERCEL_TOKEN` - Get from Vercel Dashboard → Settings → Tokens
- `VERCEL_ORG_ID` - Found in Vercel project settings
- `VERCEL_PROJECT_ID` - Found in Vercel project settings

## GitHub Pages Deployment

GitHub Pages offers free hosting for public repositories with custom domain support.

### Quick GitHub Pages Deployment

#### Option 1: GitHub Actions (Recommended)

Create `.github/workflows/github-pages.yml`:

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main, master]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: "pages"
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: 1.70.0

      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Build site
        run: cargo run -- -i docs -o _site

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: '_site'

  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

#### Option 2: Manual Deployment

```bash
# Build your site
cargo run -- -i docs -o dist

# Create gh-pages branch
git checkout --orphan gh-pages
git rm -rf .
cp -r dist/* .
git add .
git commit -m "Deploy to GitHub Pages"
git push origin gh-pages --force

# Switch back to main
git checkout main
```

### GitHub Pages Configuration

1. **Enable GitHub Pages:**
   - Go to repository Settings → Pages
   - Source: "GitHub Actions" (for workflow) or "Deploy from branch" (for manual)
   - Branch: `gh-pages` if using manual method

2. **Custom Domain (Optional):**
   - Add `CNAME` file to output directory:
   ```bash
   echo "docs.yourdomain.com" > dist/CNAME
   ```

3. **Base URL Configuration:**
   - For project pages (username.github.io/repo-name), update links to use base path
   - Modify templates to include base path in asset URLs

### GitHub Pages Features

- **Free SSL/TLS certificates** for custom domains
- **Jekyll-free deployment** (disable Jekyll with `.nojekyll` file)
- **CDN distribution** via GitHub's infrastructure
- **Version control** built-in
- **Branch-based deployment** strategies

## AWS Amplify Deployment

AWS Amplify provides fully managed hosting with AWS service integration.

### Quick AWS Amplify Deployment

#### Option 1: Amplify Console

1. **Connect repository:**
   - Go to [AWS Amplify Console](https://console.aws.amazon.com/amplify/)
   - Click "New app" → "Host web app"
   - Connect your Git provider

2. **Configure build:**
   ```yaml
   version: 1
   frontend:
     phases:
       preBuild:
         commands:
           - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
           - source $HOME/.cargo/env
       build:
         commands:
           - cargo run -- -i docs -o dist
     artifacts:
       baseDirectory: dist
       files:
         - '**/*'
     cache:
       paths:
         - target/**/*
         - ~/.cargo/**/*
   ```

3. **Deploy:**
   - Review settings
   - Click "Save and deploy"

#### Option 2: Amplify CLI

```bash
# Install Amplify CLI
npm install -g @aws-amplify/cli

# Configure Amplify
amplify configure

# Initialize project
amplify init

# Add hosting
amplify add hosting

# Build and deploy
cargo run -- -i docs -o dist
amplify publish
```

### AWS Amplify Configuration

Create `amplify.yml`:

```yaml
version: 1
frontend:
  phases:
    preBuild:
      commands:
        # Install Rust
        - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        - source $HOME/.cargo/env
        - rustc --version
    build:
      commands:
        - cargo build --release
        - cargo run --release -- -i docs -o dist
  artifacts:
    baseDirectory: dist
    files:
      - '**/*'
  cache:
    paths:
      - target/**/*
      - ~/.cargo/**/*
```

### AWS Amplify Features

- **Custom domains** with free SSL certificates
- **Branch deployments** for staging/production
- **Environment variables** management
- **Serverless backend** integration (Lambda, DynamoDB)
- **Authentication** via Amazon Cognito
- **CI/CD** built-in
- **Performance monitoring** and analytics

## Render Deployment

Render offers simple deployments with free SSL and automatic scaling.

### Quick Render Deployment

#### Option 1: Render Dashboard

1. **Create Web Service:**
   - Go to [Render Dashboard](https://dashboard.render.com/)
   - Click "New +" → "Static Site"
   - Connect your repository

2. **Configure:**
   - **Build Command**: `cargo run -- -i docs -o dist`
   - **Publish Directory**: `dist`
   - **Environment**: Select "Rust"

3. **Deploy:**
   - Click "Create Static Site"
   - Automatic deployments on push

#### Option 2: render.yaml

Create `render.yaml` in repository root:

```yaml
services:
  - type: web
    name: md-book
    env: static
    buildCommand: cargo run -- -i docs -o dist
    staticPublishPath: dist
    headers:
      - path: /*
        name: X-Frame-Options
        value: DENY
      - path: /*
        name: X-Content-Type-Options
        value: nosniff
      - path: /css/*
        name: Cache-Control
        value: public, max-age=31536000, immutable
      - path: /js/*
        name: Cache-Control
        value: public, max-age=31536000, immutable
    routes:
      - type: rewrite
        source: /*
        destination: /index.html
```

### Render Features

- **Free SSL certificates** for custom domains
- **DDoS protection** built-in
- **Auto-deploy** from Git
- **Preview environments** for pull requests
- **Custom headers** and redirects
- **Zero-downtime deploys**

## Railway Deployment

Railway provides simple deployments with automatic HTTPS and preview environments.

### Quick Railway Deployment

#### Option 1: Railway CLI

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login
railway login

# Initialize project
railway init

# Link to project (or create new)
railway link

# Deploy
cargo run -- -i docs -o dist
railway up
```

#### Option 2: Railway Dashboard

1. **Create Project:**
   - Go to [Railway Dashboard](https://railway.app/dashboard)
   - Click "New Project" → "Deploy from GitHub repo"
   - Select your repository

2. **Configure:**
   - Railway auto-detects Rust projects
   - Set build command: `cargo build --release && cargo run --release -- -i docs -o dist`
   - Set start command: Static site serving

3. **Deploy:**
   - Automatic deployment begins
   - Get deployment URL

### Railway Configuration

Create `railway.toml`:

```toml
[build]
builder = "NIXPACKS"
buildCommand = "cargo build --release"

[deploy]
startCommand = "cargo run --release -- -i docs -o dist --serve --port $PORT"
healthcheckPath = "/"
healthcheckTimeout = 100
restartPolicyType = "ON_FAILURE"
restartPolicyMaxRetries = 10
```

Or use `Procfile`:

```
web: cargo run --release -- -i docs -o dist --serve --port $PORT
```

### Railway Features

- **Automatic HTTPS** for all deployments
- **Preview environments** for branches and PRs
- **Environment variables** management
- **Database integration** (PostgreSQL, MySQL, Redis)
- **Metrics and logging**
- **Custom domains** with SSL

## Fly.io Deployment

Fly.io provides edge deployment with global distribution using containers.

### Quick Fly.io Deployment

#### Setup and Deploy

```bash
# Install flyctl
curl -L https://fly.io/install.sh | sh

# Login
flyctl auth login

# Launch app (creates fly.toml)
flyctl launch

# Deploy
flyctl deploy
```

### Fly.io Configuration

Create `fly.toml`:

```toml
app = "md-book"
primary_region = "iad"

[build]
  [build.args]
    RUST_VERSION = "1.70.0"

[env]
  PORT = "8080"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 0

  [[http_service.checks]]
    grace_period = "10s"
    interval = "30s"
    method = "GET"
    timeout = "5s"
    path = "/"
```

Create `Dockerfile`:

```dockerfile
# Build stage
FROM rust:1.70 AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/md-book /usr/local/bin/md-book
COPY --from=builder /app/docs ./docs
COPY --from=builder /app/src/templates ./templates

# Build the site
RUN md-book -i docs -o dist

# Serve static files
EXPOSE 8080

CMD ["md-book", "-i", "docs", "-o", "dist", "--serve", "--port", "8080"]
```

### Fly.io Features

- **Global edge network** - Deploy to multiple regions
- **Auto-scaling** based on demand
- **Zero-downtime deploys**
- **Built-in SSL/TLS**
- **Private networking** between services
- **Volume storage** for persistent data
- **Custom domains** with automatic certificates

## DigitalOcean App Platform Deployment

DigitalOcean App Platform offers simple pricing and managed infrastructure.

### Quick DigitalOcean Deployment

#### Option 1: App Platform Console

1. **Create App:**
   - Go to [DigitalOcean App Platform](https://cloud.digitalocean.com/apps)
   - Click "Create App"
   - Connect your repository

2. **Configure:**
   - **Type**: Static Site
   - **Build Command**: `cargo run -- -i docs -o dist`
   - **Output Directory**: `dist`
   - **Environment**: Rust

3. **Deploy:**
   - Review plan ($0 for static sites)
   - Click "Launch Your App"

#### Option 2: App Spec (YAML)

Create `.do/app.yaml`:

```yaml
name: md-book
region: nyc
static_sites:
  - name: web
    github:
      repo: yourusername/md-book
      branch: main
      deploy_on_push: true
    build_command: cargo run -- -i docs -o dist
    output_dir: dist
    routes:
      - path: /
    environment_slug: rust
    envs:
      - key: RUST_VERSION
        value: "1.70.0"
    cors:
      allow_origins:
        - prefix: https://
      allow_methods:
        - GET
        - OPTIONS
        - POST
      allow_headers:
        - Content-Type
```

### DigitalOcean Features

- **Free tier** for static sites
- **Automatic SSL** certificates
- **CDN** included
- **Auto-deploy** from Git
- **Preview deployments** for PRs
- **Simple pricing** - No surprises
- **Managed databases** integration
- **Easy monitoring** and alerts

## Platform Comparison Table

| Feature | Cloudflare | Netlify | Vercel | GitHub Pages | AWS Amplify | Render | Railway | Fly.io | DigitalOcean |
|---------|-----------|---------|--------|--------------|-------------|--------|---------|--------|--------------|
| **Free Tier** | Unlimited | 100GB | 100GB | Unlimited | Free (pay-as-go) | Free | $5 credit | Free tier | Free static |
| **Build Minutes** | Unlimited | 300/mo | 6000/mo | Unlimited | 1000/mo | 500/mo | Limited | N/A | 100/mo |
| **Bandwidth** | Unlimited | 100GB | 100GB | 100GB/mo | 15GB | 100GB | 100GB | Limited | 100GB |
| **Custom Domain** | ✅ Free SSL | ✅ Free SSL | ✅ Free SSL | ✅ Free SSL | ✅ Free SSL | ✅ Free SSL | ✅ Free SSL | ✅ Free SSL | ✅ Free SSL |
| **Auto Deploy** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Preview Deploys** | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Edge Functions** | Workers | Functions | Functions | ❌ | Lambda | ❌ | Limited | ✅ | Functions |
| **Global CDN** | ✅ 200+ | ✅ | ✅ | ✅ | ✅ | ✅ | Limited | ✅ Multi-region | ✅ |
| **Analytics** | ✅ Built-in | Paid | Paid | Limited | ✅ | Basic | Basic | Basic | Basic |
| **DDoS Protection** | ✅ Enterprise | Basic | ✅ | ✅ | ✅ AWS Shield | ✅ | Limited | ✅ | ✅ |
| **Build Cache** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Deployment Speed** | Very Fast | Fast | Fast | Medium | Fast | Fast | Fast | Medium | Fast |
| **Ease of Setup** | Medium | Easy | Very Easy | Easy | Medium | Easy | Very Easy | Medium | Easy |

## Choosing the Right Platform

### Use **Cloudflare Pages** if you need:
- Unlimited bandwidth and builds
- Advanced edge computing with Workers
- Best-in-class DDoS protection
- Global CDN with 200+ locations

### Use **Netlify** if you need:
- Simple drag-and-drop deployment
- Built-in form handling
- Generous free tier
- Easy custom domains

### Use **Vercel** if you need:
- Zero-config deployment
- Excellent Next.js integration (future)
- Edge functions
- Great developer experience

### Use **GitHub Pages** if you need:
- Simple hosting for open source
- Free for public repositories
- Direct GitHub integration
- Jekyll support (if needed)

### Use **AWS Amplify** if you need:
- AWS ecosystem integration
- Serverless backend (Lambda, DynamoDB)
- Enterprise features
- Advanced authentication

### Use **Render** if you need:
- Simple pricing
- Free SSL and DDoS protection
- PostgreSQL/Redis included in free tier
- Preview environments

### Use **Railway** if you need:
- Simplest deployment experience
- Database integration
- Preview environments
- Hobby projects

### Use **Fly.io** if you need:
- Global edge deployment
- Container-based deployment
- Multi-region by default
- Low-latency worldwide

### Use **DigitalOcean** if you need:
- Simple pricing
- Managed infrastructure
- Free static site hosting
- Great documentation

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