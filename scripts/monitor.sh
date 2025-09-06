#!/bin/bash

# Monitoring script for MD-Book deployment
# Checks deployment health and provides status information

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PRODUCTION_URL="https://md-book.pages.dev"
STAGING_URL="https://md-book-staging.pages.dev"
WORKER_HEALTH_ENDPOINT="/api/health"

# Check URL health
check_url() {
    local url=$1
    local name=$2
    
    echo -n "Checking $name... "
    
    if curl -s -f -o /dev/null --max-time 10 "$url"; then
        echo -e "${GREEN}✅ OK${NC}"
        return 0
    else
        echo -e "${RED}❌ FAILED${NC}"
        return 1
    fi
}

# Check API endpoint
check_api() {
    local base_url=$1
    local name=$2
    
    echo -n "Checking $name API... "
    
    local response
    if response=$(curl -s -f --max-time 10 "$base_url$WORKER_HEALTH_ENDPOINT" 2>/dev/null); then
        if echo "$response" | grep -q '"status":"ok"'; then
            echo -e "${GREEN}✅ OK${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠️  Response invalid${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ FAILED${NC}"
        return 1
    fi
}

# Get deployment info
get_deployment_info() {
    local environment=$1
    local project_name="md-book"
    
    if [ "$environment" = "staging" ]; then
        project_name="md-book-staging"
    fi
    
    echo -e "${YELLOW}📊 $environment Deployment Info:${NC}"
    
    if command -v wrangler &> /dev/null; then
        # Get Pages deployment info
        echo "Recent deployments:"
        wrangler pages deployment list --project-name="$project_name" --limit=3 2>/dev/null || echo "  Unable to fetch deployment info"
        
        # Get Worker deployment info if applicable
        if [ "$DEPLOY_WORKER" != "false" ]; then
            echo ""
            echo "Worker deployment:"
            wrangler deployments list --name="md-book-worker" --limit=2 2>/dev/null || echo "  Unable to fetch worker info"
        fi
    else
        echo "  Wrangler CLI not available - install with: npm install -g wrangler"
    fi
    
    echo ""
}

# Performance check
performance_check() {
    local url=$1
    local name=$2
    
    echo -e "${YELLOW}⚡ $name Performance:${NC}"
    
    # Use curl to measure response times
    local time_total
    local time_connect
    local time_starttransfer
    
    if command -v curl &> /dev/null; then
        if time_total=$(curl -s -w "%{time_total}" -o /dev/null --max-time 30 "$url" 2>/dev/null); then
            time_connect=$(curl -s -w "%{time_connect}" -o /dev/null --max-time 30 "$url" 2>/dev/null || echo "0")
            time_starttransfer=$(curl -s -w "%{time_starttransfer}" -o /dev/null --max-time 30 "$url" 2>/dev/null || echo "0")
            
            echo "  Total time: ${time_total}s"
            echo "  Connect time: ${time_connect}s"
            echo "  First byte: ${time_starttransfer}s"
            
            # Evaluate performance
            if (( $(echo "$time_total < 2.0" | bc -l) )); then
                echo -e "  Performance: ${GREEN}Excellent${NC}"
            elif (( $(echo "$time_total < 5.0" | bc -l) )); then
                echo -e "  Performance: ${YELLOW}Good${NC}"
            else
                echo -e "  Performance: ${RED}Needs Improvement${NC}"
            fi
        else
            echo "  Unable to measure performance"
        fi
    else
        echo "  curl not available for performance testing"
    fi
    
    echo ""
}

# Check SSL certificate
check_ssl() {
    local url=$1
    local name=$2
    
    echo -e "${YELLOW}🔒 $name SSL Certificate:${NC}"
    
    local domain
    domain=$(echo "$url" | sed 's|https\?://||' | sed 's|/.*||')
    
    if command -v openssl &> /dev/null; then
        local cert_info
        if cert_info=$(echo | openssl s_client -servername "$domain" -connect "$domain:443" 2>/dev/null | openssl x509 -noout -dates 2>/dev/null); then
            echo "$cert_info" | while IFS= read -r line; do
                echo "  $line"
            done
            echo -e "  Status: ${GREEN}Valid${NC}"
        else
            echo -e "  Status: ${RED}Unable to verify${NC}"
        fi
    else
        echo "  openssl not available for SSL checking"
    fi
    
    echo ""
}

# Main monitoring function
monitor() {
    local environment=${1:-all}
    
    echo -e "${BLUE}🔍 MD-Book Deployment Monitor${NC}"
    echo -e "${BLUE}==============================${NC}"
    echo "Timestamp: $(date)"
    echo ""
    
    local overall_status=0
    
    case $environment in
        "production"|"prod"|"all")
            echo -e "${GREEN}📊 Production Environment${NC}"
            echo "URL: $PRODUCTION_URL"
            echo ""
            
            check_url "$PRODUCTION_URL" "Production site" || overall_status=1
            check_api "$PRODUCTION_URL" "Production" || overall_status=1
            echo ""
            
            if [ "$1" != "quick" ]; then
                performance_check "$PRODUCTION_URL" "Production"
                check_ssl "$PRODUCTION_URL" "Production"
                get_deployment_info "production"
            fi
            
            if [ "$environment" != "all" ]; then
                exit $overall_status
            fi
            ;;
    esac
    
    case $environment in
        "staging"|"stage"|"all")
            echo -e "${YELLOW}📊 Staging Environment${NC}"
            echo "URL: $STAGING_URL"
            echo ""
            
            check_url "$STAGING_URL" "Staging site" || overall_status=1
            check_api "$STAGING_URL" "Staging" || overall_status=1
            echo ""
            
            if [ "$1" != "quick" ]; then
                performance_check "$STAGING_URL" "Staging"
                check_ssl "$STAGING_URL" "Staging"
                get_deployment_info "staging"
            fi
            ;;
    esac
    
    # Summary
    echo -e "${BLUE}📈 Summary${NC}"
    if [ $overall_status -eq 0 ]; then
        echo -e "${GREEN}✅ All systems operational${NC}"
    else
        echo -e "${RED}❌ Some issues detected${NC}"
    fi
    
    echo ""
    echo -e "${BLUE}🔧 Troubleshooting Commands:${NC}"
    echo "  ./scripts/monitor.sh production  # Check production only"
    echo "  ./scripts/monitor.sh staging     # Check staging only"
    echo "  ./scripts/monitor.sh quick       # Quick health check"
    echo "  ./scripts/deploy.sh production   # Redeploy if issues found"
    
    exit $overall_status
}

# Handle script arguments
case ${1:-""} in
    "-h"|"--help"|"help")
        echo "MD-Book Deployment Monitor"
        echo ""
        echo "Usage: $0 [environment] [mode]"
        echo ""
        echo "Environments:"
        echo "  production, prod  Check production environment only"
        echo "  staging, stage   Check staging environment only"
        echo "  all              Check all environments (default)"
        echo ""
        echo "Modes:"
        echo "  quick            Quick health check only"
        echo "  (default)        Full monitoring with performance and SSL checks"
        echo ""
        echo "Examples:"
        echo "  $0                    # Full check of all environments"
        echo "  $0 production         # Full check of production"
        echo "  $0 production quick   # Quick production health check"
        echo "  $0 staging           # Full check of staging"
        exit 0
        ;;
    *)
        monitor "$@"
        ;;
esac