#!/bin/bash

# ============================================================================
# Auth Manager - Lambda Deployment Script (SAM)
# Builds Docker image and deploys to AWS Lambda using SAM
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
STACK_NAME="${STACK_NAME:-auth-manager-prod}"
AWS_REGION="${AWS_REGION:-eu-central-1}"
AWS_PROFILE="${AWS_PROFILE:-default}"

# ============================================================================
# Helper Functions
# ============================================================================

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_header() {
    echo ""
    echo -e "${BLUE}============================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}============================================================${NC}"
}

check_dependencies() {
    print_header "Checking Dependencies"

    local missing_deps=()

    if ! command -v docker &> /dev/null; then
        missing_deps+=("docker")
    fi

    if ! command -v aws &> /dev/null; then
        missing_deps+=("aws-cli")
    fi

    if ! command -v sam &> /dev/null; then
        missing_deps+=("sam-cli")
        print_warning "SAM CLI not installed. Install with: pip install aws-sam-cli"
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing required dependencies: ${missing_deps[*]}"
        echo ""
        echo "Install SAM CLI:"
        echo "  brew install aws-sam-cli  # macOS"
        echo "  pip install aws-sam-cli    # Python"
        exit 1
    fi

    print_success "All dependencies are installed"
}

get_aws_account_id() {
    aws sts get-caller-identity --profile "$AWS_PROFILE" --query Account --output text
}

get_ecr_repository_uri() {
    aws cloudformation describe-stacks \
        --stack-name "$STACK_NAME" \
        --profile "$AWS_PROFILE" \
        --region "$AWS_REGION" \
        --query 'Stacks[0].Outputs[?OutputKey==`ECRRepositoryUri`].OutputValue' \
        --output text 2>/dev/null || echo ""
}

# ============================================================================
# Main Functions
# ============================================================================

login_to_ecr() {
    print_header "Logging in to Amazon ECR"

    local account_id=$(get_aws_account_id)
    print_info "AWS Account ID: $account_id"

    aws ecr get-login-password \
        --region "$AWS_REGION" \
        --profile "$AWS_PROFILE" | \
        docker login \
            --username AWS \
            --password-stdin "$account_id.dkr.ecr.$AWS_REGION.amazonaws.com"

    print_success "Successfully logged in to ECR"
}

build_and_push_image() {
    print_header "Building and Pushing Docker Image"

    cd "$PROJECT_ROOT"

    local ecr_uri=$(get_ecr_repository_uri)
    if [ -z "$ecr_uri" ]; then
        print_error "ECR repository not found. Deploy SAM stack first with --create-stack"
        exit 1
    fi

    print_info "ECR Repository: $ecr_uri"
    print_info "Building Docker image..."

    # Build the image
    docker build \
        --platform linux/amd64 \
        --target runtime \
        -f docker/Dockerfile \
        -t "$ecr_uri:latest" \
        .

    print_success "Docker image built successfully"

    # Push to ECR
    print_info "Pushing image to ECR..."
    docker push "$ecr_uri:latest"

    print_success "Image pushed to ECR successfully"
}

update_lambda_image() {
    print_header "Updating Lambda Function Image"

    local function_name="$STACK_NAME"

    local ecr_uri=$(get_ecr_repository_uri)
    if [ -z "$ecr_uri" ]; then
        print_error "ECR repository not found"
        exit 1
    fi

    print_info "Function: $function_name"
    print_info "Image: $ecr_uri:latest"

    aws lambda update-function-code \
        --function-name "$function_name" \
        --image-uri "$ecr_uri:latest" \
        --profile "$AWS_PROFILE" \
        --region "$AWS_REGION" \
        --output json > /dev/null

    print_success "Lambda function updated"

    print_info "Waiting for function to be ready..."
    aws lambda wait function-updated \
        --function-name "$function_name" \
        --profile "$AWS_PROFILE" \
        --region "$AWS_REGION"

    print_success "Lambda function is ready"
}

create_ecr_repository() {
    print_header "Creating ECR Repository"

    local repo_name="auth-manager-prod"

    # Check if repository already exists
    local repo_exists=$(aws ecr describe-repositories \
        --repository-names "$repo_name" \
        --region "$AWS_REGION" \
        --profile "$AWS_PROFILE" \
        --query 'repositories[0].repositoryName' \
        --output text 2>/dev/null || echo "")

    if [ -n "$repo_exists" ]; then
        print_info "ECR repository already exists: $repo_name"
        return 0
    fi

    print_info "Creating ECR repository: $repo_name"

    aws ecr create-repository \
        --repository-name "$repo_name" \
        --region "$AWS_REGION" \
        --profile "$AWS_PROFILE" \
        --image-scanning-configuration scanOnPush=true \
        --tags Key=Environment,Value=production Key=Application,Value=auth-manager \
        --output json > /dev/null

    print_success "ECR repository created"
}

deploy_sam_stack() {
    print_header "Deploying SAM Stack"

    cd "$PROJECT_ROOT/infra"

    print_info "Stack: $STACK_NAME"
    print_info "Region: $AWS_REGION"
    print_info "Profile: $AWS_PROFILE"

    # Check if params/prod.json exists
    if [ ! -f "params/prod.json" ]; then
        print_error "params/prod.json not found!"
        echo ""
        echo "Create it from template:"
        echo "  cp params/prod.json.example params/prod.json"
        echo "  # Edit params/prod.json with your production credentials"
        exit 1
    fi

    # Check if jq is available
    if ! command -v jq &> /dev/null; then
        print_error "jq is not installed (required to read params/prod.json)"
        echo ""
        echo "Install jq:"
        echo "  brew install jq  # macOS"
        echo "  apt-get install jq  # Ubuntu/Debian"
        exit 1
    fi

    # Read parameters from JSON and convert to SAM format
    print_info "Reading parameters from params/prod.json..."
    local param_overrides=$(jq -r 'to_entries | map("\(.key)=\(.value)") | join(" ")' params/prod.json)

    # Check if samconfig.toml needs updating
    local account_id=$(get_aws_account_id)
    local ecr_repo_uri="${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com/auth-manager-prod"

    # Update samconfig.toml with correct ECR URI
    if command -v sed &> /dev/null; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            sed -i '' "s|image_repositories = \[\"AuthManagerFunction=.*\"\]|image_repositories = [\"AuthManagerFunction=${ecr_repo_uri}\"]|g" samconfig.toml
        else
            # Linux
            sed -i "s|image_repositories = \[\"AuthManagerFunction=.*\"\]|image_repositories = [\"AuthManagerFunction=${ecr_repo_uri}\"]|g" samconfig.toml
        fi
    fi

    print_info "Deploying with SAM..."

    sam deploy \
        --stack-name "$STACK_NAME" \
        --region "$AWS_REGION" \
        --profile "$AWS_PROFILE" \
        --parameter-overrides $param_overrides \
        --no-confirm-changeset \
        --no-fail-on-empty-changeset

    print_success "SAM stack deployed successfully"
}

show_outputs() {
    print_header "Stack Outputs"

    aws cloudformation describe-stacks \
        --stack-name "$STACK_NAME" \
        --profile "$AWS_PROFILE" \
        --region "$AWS_REGION" \
        --query 'Stacks[0].Outputs[].[OutputKey,OutputValue]' \
        --output table
}

show_api_url() {
    local api_url=$(aws cloudformation describe-stacks \
        --stack-name "$STACK_NAME" \
        --profile "$AWS_PROFILE" \
        --region "$AWS_REGION" \
        --query 'Stacks[0].Outputs[?OutputKey==`ApiUrl`].OutputValue' \
        --output text 2>/dev/null)

    if [ -n "$api_url" ]; then
        echo ""
        print_success "API URL: ${api_url}"
        echo ""
        echo "Test endpoints:"
        echo "  curl ${api_url}/health"
        echo "  curl -X POST ${api_url}/auth/register -H 'Content-Type: application/json' -d '{\"email\":\"test@example.com\",\"username\":\"test\",\"password\":\"Test123!\"}'"
    fi
}

# ============================================================================
# Usage
# ============================================================================

usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Deploy Auth Manager to AWS Lambda using SAM

OPTIONS:
    -s, --stack-name NAME     CloudFormation stack name [default: auth-manager-prod]
    -r, --region REGION       AWS region [default: eu-central-1]
    -p, --profile PROFILE     AWS CLI profile [default: default]
    --create-stack            Create new SAM stack (deploys infrastructure)
    --skip-build              Skip Docker build and push
    -h, --help                Show this help message

EXAMPLES:
    # First deployment (creates stack)
    $0 --create-stack

    # Update code (build, push, update Lambda)
    $0

    # Update with custom profile
    $0 -p production

    # Just update Lambda (skip build)
    $0 --skip-build

ENVIRONMENT VARIABLES:
    STACK_NAME               CloudFormation stack name
    AWS_REGION               AWS region
    AWS_PROFILE              AWS CLI profile

PREREQUISITES:
    1. AWS CLI installed and configured
    2. SAM CLI installed (pip install aws-sam-cli)
    3. Docker installed and running
    4. jq installed (brew install jq)
    5. Create infra/params/prod.json with your credentials
       (copy from infra/params/prod.json.example)

EOF
}

# ============================================================================
# Main Script
# ============================================================================

main() {
    local create_stack=false
    local skip_build=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -s|--stack-name)
                STACK_NAME="$2"
                shift 2
                ;;
            -r|--region)
                AWS_REGION="$2"
                shift 2
                ;;
            -p|--profile)
                AWS_PROFILE="$2"
                shift 2
                ;;
            --create-stack)
                create_stack=true
                shift
                ;;
            --skip-build)
                skip_build=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    print_header "Auth Manager - Lambda Deployment (SAM)"
    print_info "Stack Name: $STACK_NAME"
    print_info "AWS Region: $AWS_REGION"
    print_info "AWS Profile: $AWS_PROFILE"

    # Check dependencies
    check_dependencies

    # Create or update SAM stack
    if [ "$create_stack" = true ]; then
        print_warning "Creating new SAM stack..."

        # Step 1: Create ECR repository first
        create_ecr_repository

        # Step 2: Build and push initial Docker image
        local account_id=$(get_aws_account_id)
        local ecr_uri="${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com/auth-manager-prod"

        print_info "ECR Repository: $ecr_uri"

        # Login to ECR
        login_to_ecr

        # Build and push the image
        cd "$PROJECT_ROOT"

        print_info "Building Docker image..."
        docker build \
            --platform linux/amd64 \
            --target runtime \
            -f docker/Dockerfile \
            -t "$ecr_uri:latest" \
            .

        print_success "Docker image built successfully"

        print_info "Pushing image to ECR..."
        docker push "$ecr_uri:latest"

        print_success "Image pushed to ECR"

        # Step 3: Now deploy the SAM stack with Lambda
        deploy_sam_stack

        print_success "Stack created successfully"
        show_outputs
        show_api_url

        print_header "Deployment Complete"
        print_success "Auth Manager infrastructure created and deployed!"
        exit 0
    fi

    # Login to ECR
    login_to_ecr

    # Build and push Docker image
    if [ "$skip_build" = false ]; then
        build_and_push_image
    else
        print_warning "Skipping Docker build (--skip-build)"
    fi

    # Update Lambda function
    update_lambda_image

    # Show outputs
    show_outputs
    show_api_url

    print_header "Deployment Complete"
    print_success "Auth Manager deployed successfully to AWS Lambda!"
}

# Run main function
main "$@"
