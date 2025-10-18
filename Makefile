.PHONY: help check fmt clippy test clean install-pre-commit

# Default target
help: ## Show this help message
	@echo "md-book Development Commands"
	@echo "============================"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development commands
check: ## Run cargo check
	cargo check --all-targets --all-features

fmt: ## Check code formatting
	cargo fmt --all -- --check

fmt-fix: ## Fix code formatting
	cargo fmt --all

clippy: ## Run clippy lints
	cargo clippy --all-targets --all-features -- -D warnings

clippy-pedantic: ## Run clippy with pedantic lints
	cargo clippy --all-targets --all-features -- -W clippy::all -W clippy::pedantic -W clippy::nursery -W clippy::cargo

test: ## Run tests
	cargo test --lib --bins

test-integration: ## Run integration tests
	cargo test --test integration --features "tokio,search,syntax-highlighting"

test-all: ## Run all tests including integration and e2e
	cargo test --lib --bins --test integration --test e2e --features "tokio,search,syntax-highlighting"

# Quality assurance
qa: fmt clippy test ## Run all quality checks (format, lint, test)

# Build commands
build: ## Build the project
	cargo build

build-release: ## Build release version
	cargo build --release

# Cleanup
clean: ## Clean build artifacts
	cargo clean

# Pre-commit setup
install-pre-commit: ## Install pre-commit hooks
	./scripts/setup-pre-commit.sh

# Development workflow
dev-check: check fmt clippy test ## Complete development check
	@echo "✅ All checks passed!"

# CI simulation
ci-local: ## Run CI checks locally
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --lib --bins
	cargo test --test integration --features "tokio,search,syntax-highlighting"
	@echo "✅ CI checks passed locally!"