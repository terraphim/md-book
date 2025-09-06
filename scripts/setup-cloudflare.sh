#!/bin/bash

# Setup script for Cloudflare Pages and Workers deployment
# This script helps configure the project for first-time deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 MD-Book Cloudflare Setup${NC}"
echo -e "${BLUE}===========================${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "wrangler.toml" ]; then
    echo -e "${RED}❌ This script must be run from the md-book project root directory${NC}"
    exit 1
fi

# Check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}🔍 Checking prerequisites...${NC}"
    
    local missing_tools=()
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        missing_tools+=("Node.js")
    fi
    
    # Check npm
    if ! command -v npm &> /dev/null; then
        missing_tools+=("npm")
    fi
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("Rust/Cargo")
    fi
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        echo -e "${RED}❌ Missing required tools: ${missing_tools[*]}${NC}"
        echo ""
        echo "Please install the missing tools:"
        for tool in "${missing_tools[@]}"; do
            case $tool in
                "Node.js"|"npm")
                    echo "  - Node.js and npm: https://nodejs.org/"
                    ;;
                "Rust/Cargo")
                    echo "  - Rust: https://rustup.rs/"
                    ;;
            esac
        done
        exit 1
    fi
    
    echo -e "${GREEN}✅ All prerequisites installed${NC}"
}

# Install Wrangler CLI
setup_wrangler() {
    echo -e "${YELLOW}📦 Setting up Wrangler CLI...${NC}"
    
    if command -v wrangler &> /dev/null; then
        echo "Wrangler already installed: $(wrangler --version)"
    else
        echo "Installing Wrangler globally..."
        npm install -g wrangler
    fi
    
    echo -e "${GREEN}✅ Wrangler CLI ready${NC}"
}

# Configure environment variables
setup_environment() {
    echo -e "${YELLOW}🔧 Setting up environment configuration...${NC}"
    
    # Create .env file if it doesn't exist
    if [ ! -f ".env" ]; then
        cp .env.example .env
        echo "Created .env file from template"
    else
        echo ".env file already exists"
    fi
    
    # Check if environment variables are set
    if [ -z "$CLOUDFLARE_API_TOKEN" ] || [ -z "$CLOUDFLARE_ACCOUNT_ID" ]; then
        echo ""
        echo -e "${YELLOW}⚠️  Environment variables not configured${NC}"
        echo ""
        echo "Please set up your Cloudflare credentials:"
        echo ""
        echo "1. Get your API Token from: https://dash.cloudflare.com/profile/api-tokens"
        echo "   - Click 'Create Token'"
        echo "   - Use 'Pages and Workers' template, or create custom with:"
        echo "     * Account: Cloudflare Pages:Edit"
        echo "     * Zone: Zone:Read, Zone:Page Rules:Edit"
        echo ""
        echo "2. Get your Account ID from: https://dash.cloudflare.com/"
        echo "   - It's shown in the right sidebar"
        echo ""
        echo "3. Set these in your environment:"
        echo "   export CLOUDFLARE_API_TOKEN=\"your-token-here\""
        echo "   export CLOUDFLARE_ACCOUNT_ID=\"your-account-id-here\""
        echo ""
        echo "4. Or add them to GitHub repository secrets for CI/CD"
        echo ""
        
        read -p "Press Enter to continue once you've configured the environment variables..."
    else
        echo -e "${GREEN}✅ Environment variables already configured${NC}"
    fi
}

# Create Cloudflare Pages project
create_pages_project() {
    echo -e "${YELLOW}🌐 Creating Cloudflare Pages project...${NC}"
    
    # Check if project exists
    if wrangler pages project list 2>/dev/null | grep -q "md-book"; then
        echo "Pages project 'md-book' already exists"
    else
        echo "Creating new Pages project..."
        # Note: This will be created automatically on first deployment
        echo "Project will be created automatically during first deployment"
    fi
    
    echo -e "${GREEN}✅ Pages project configuration ready${NC}"
}

# Test local build
test_build() {
    echo -e "${YELLOW}🔨 Testing local build...${NC}"
    
    # Build the project
    echo "Building Rust binary..."
    cargo build --release
    
    # Create test input if needed
    if [ ! -d "test_input" ]; then
        echo "Creating test input directory..."
        mkdir -p test_input
        echo "# Test Documentation" > test_input/index.md
        echo "This is a test page for MD-Book deployment." >> test_input/index.md
    fi
    
    # Generate static site
    echo "Generating static site..."
    ./target/release/md-book -i test_input -o dist
    
    # Verify output
    if [ -f "dist/index.html" ]; then
        echo -e "${GREEN}✅ Local build successful${NC}"
        echo "Generated $(find dist -type f | wc -l) files"
    else
        echo -e "${RED}❌ Build failed - no index.html generated${NC}"
        exit 1
    fi
}

# Setup GitHub Actions secrets
setup_github_secrets() {
    echo -e "${YELLOW}🔐 GitHub Actions setup...${NC}"
    
    if command -v gh &> /dev/null; then
        echo "GitHub CLI detected. You can set secrets with:"
        echo ""
        echo "  gh secret set CLOUDFLARE_API_TOKEN"
        echo "  gh secret set CLOUDFLARE_ACCOUNT_ID"
        echo ""
    else
        echo "To enable automatic deployments, add these secrets to your GitHub repository:"
        echo ""
        echo "1. Go to: https://github.com/your-username/your-repo/settings/secrets/actions"
        echo "2. Add the following repository secrets:"
        echo "   - CLOUDFLARE_API_TOKEN: Your Cloudflare API token"
        echo "   - CLOUDFLARE_ACCOUNT_ID: Your Cloudflare account ID"
        echo ""
    fi
    
    echo -e "${GREEN}✅ GitHub Actions information provided${NC}"
}

# Show next steps
show_next_steps() {
    echo ""
    echo -e "${BLUE}🎉 Setup Complete!${NC}"
    echo -e "${BLUE}=================${NC}"
    echo ""
    echo "Your MD-Book project is now configured for Cloudflare deployment!"
    echo ""
    echo -e "${GREEN}Next Steps:${NC}"
    echo ""
    echo "1. 📝 Add your content:"
    echo "   - Put your Markdown files in the input directory"
    echo "   - Update book.toml with your configuration"
    echo ""
    echo "2. 🚀 Deploy manually:"
    echo "   ./scripts/deploy.sh production"
    echo ""
    echo "3. 🔄 Enable automatic deployments:"
    echo "   - Configure GitHub repository secrets"
    echo "   - Push to main branch to trigger deployment"
    echo ""
    echo "4. 🌐 Access your site:"
    echo "   - Production: https://md-book.pages.dev"
    echo "   - Staging: https://md-book-staging.pages.dev"
    echo ""
    echo -e "${GREEN}Useful Commands:${NC}"
    echo "  ./scripts/deploy.sh --help    # Show deployment options"
    echo "  ./scripts/deploy-worker.sh   # Deploy worker only"
    echo "  cargo run -- --help          # Show MD-Book options"
    echo ""
    echo -e "${YELLOW}📚 Documentation:${NC}"
    echo "  - Project docs: ./CLAUDE.md (Deployment section)"
    echo "  - Cloudflare Pages: https://pages.cloudflare.com/"
    echo "  - Cloudflare Workers: https://workers.cloudflare.com/"
}

# Main execution
main() {
    check_prerequisites
    setup_wrangler
    setup_environment
    create_pages_project
    test_build
    setup_github_secrets
    show_next_steps
}

# Handle help
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "MD-Book Cloudflare Setup Script"
    echo ""
    echo "This script helps you set up MD-Book for deployment to Cloudflare Pages."
    echo ""
    echo "Usage: $0"
    echo ""
    echo "What this script does:"
    echo "  1. Checks for required tools (Node.js, Rust, etc.)"
    echo "  2. Installs Wrangler CLI"
    echo "  3. Sets up environment configuration"
    echo "  4. Creates Cloudflare Pages project configuration"
    echo "  5. Tests local build"
    echo "  6. Provides GitHub Actions setup instructions"
    echo ""
    exit 0
fi

# Run main function
main "$@"