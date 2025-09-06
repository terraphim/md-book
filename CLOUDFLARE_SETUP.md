# Cloudflare Secrets Setup Guide

This guide explains exactly what Cloudflare secrets you need and how to get them for deploying your MD-Book project.

## Required Secrets

### 1. CLOUDFLARE_API_TOKEN

**What it does:**
- Authenticates your deployment scripts with Cloudflare's API
- Allows GitHub Actions to deploy to Cloudflare Pages
- Enables worker deployments and configuration updates

**Required Permissions:**
- `Cloudflare Pages:Edit` - Deploy and manage Pages projects
- `Zone:Read` - Read DNS and domain information  
- `Account:Read` - Access account-level resources

**How to Get It:**

1. **Go to Cloudflare API Tokens page:**
   Visit: https://dash.cloudflare.com/profile/api-tokens

2. **Create Custom Token:**
   - Click "Create Token"
   - Select "Custom token" (not pre-made templates)

3. **Set Permissions:**
   ```
   Account - Cloudflare Pages:Edit - [Your Account]
   Zone - Zone:Read - All zones (or specific zones)  
   Account - Account:Read - [Your Account]
   ```

4. **Add Restrictions (Optional but Recommended):**
   - **Account resources:** Select your specific account
   - **Zone resources:** Select specific zones if you don't want all zones
   - **Client IP:** Add your deployment server IPs for extra security
   - **TTL:** Set expiration date (90 days recommended)

5. **Create and Copy Token:**
   - Click "Continue to summary" → "Create Token"
   - **⚠️ IMPORTANT**: Copy the token immediately - you won't see it again!
   - Token format: `1234567890abcdef1234567890abcdef12345678`

### 2. CLOUDFLARE_ACCOUNT_ID

**What it does:**
- Identifies which Cloudflare account to deploy resources to
- Required for creating Pages projects and Workers
- Links deployments to the correct billing account

**How to Get It:**

1. **Go to Cloudflare Dashboard:**
   Visit: https://dash.cloudflare.com/

2. **Find Account ID:**
   - On any page, look at the **right sidebar**
   - Under "Account" section, you'll see "Account ID"
   - Format: `a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0`

3. **Copy the Value:**
   - Click the copy icon next to the Account ID
   - It's a 32-character hex string

## Setting Up Secrets

### For GitHub Actions (Automated Deployment)

1. **Go to your GitHub repository**
2. **Navigate to Settings → Secrets and variables → Actions**
3. **Add Repository Secrets:**
   - Click "New repository secret"
   - Name: `CLOUDFLARE_API_TOKEN`
   - Value: [Paste your API token]
   - Click "Add secret"
   - Repeat for `CLOUDFLARE_ACCOUNT_ID`

### For Local Development

#### Option 1: Environment Variables
```bash
export CLOUDFLARE_API_TOKEN="your-api-token-here"
export CLOUDFLARE_ACCOUNT_ID="your-account-id-here"
```

#### Option 2: .env File
```bash
# Copy the template
cp .env.example .env

# Edit .env file
echo "CLOUDFLARE_API_TOKEN=your-api-token-here" >> .env
echo "CLOUDFLARE_ACCOUNT_ID=your-account-id-here" >> .env
```

**⚠️ Important**: Never commit `.env` file to git - it's already in `.gitignore`

## Verification

Test that your secrets work:

```bash
# Test API token
curl -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
  "https://api.cloudflare.com/client/v4/user/tokens/verify"

# Test account access
curl -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
  "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID"
```

Both should return successful responses with account information.

## Security Best Practices

### Token Security
- **Rotate regularly**: Create new tokens every 90 days
- **Minimal permissions**: Only grant required permissions
- **Monitor usage**: Check token activity in Cloudflare dashboard
- **Revoke unused**: Delete old tokens when no longer needed

### Access Control
- **Repository secrets**: Only repository admins can view/edit secrets
- **Environment restrictions**: Use different tokens for staging vs production
- **IP restrictions**: Limit tokens to specific IPs when possible
- **Audit logs**: Monitor API token usage in Cloudflare dashboard

## Troubleshooting

### Common Issues

**"Authentication failed"**
- Check that `CLOUDFLARE_API_TOKEN` is correct
- Verify token hasn't expired
- Ensure token has required permissions

**"Account not found"**
- Check that `CLOUDFLARE_ACCOUNT_ID` is correct (32-character hex string)
- Verify you have access to the account
- Make sure account ID matches the token's account

**"Permission denied"**
- Token needs `Cloudflare Pages:Edit` permission
- Add `Account:Read` and `Zone:Read` permissions
- Check if token is restricted to specific zones

### Debug Commands

```bash
# Verify token
wrangler whoami

# List accounts
curl -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
  "https://api.cloudflare.com/client/v4/accounts"

# Test Pages access
curl -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
  "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/pages/projects"
```

## What Happens During Deployment

When you deploy, these secrets are used to:

1. **Authenticate with Cloudflare API**
2. **Create/update Pages project** in your account
3. **Upload static files** (HTML, CSS, JS, images)
4. **Configure caching rules** and security headers
5. **Deploy Worker functions** (if enabled)
6. **Set up custom domains** (if configured)
7. **Generate deployment URLs** for preview/production

The entire process is automated and secure - secrets never appear in logs or output.