.PHONY: build test clean deploy-testnet deploy-mainnet format lint all

# Default target
all: format lint build test

# Build the contract
build:
	@echo "Building secure medical records sharing contract..."
	cargo build --target wasm32-unknown-unknown --release

# Run all tests
test:
	@echo "Running tests..."
	cargo test

# Run integration tests specifically
test-integration:
	@echo "Running integration tests..."
	cargo test integration_tests --test integration_tests

# Format code
format:
	@echo "Formatting code..."
	cargo fmt

# Run clippy for linting
lint:
	@echo "Running clippy..."
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Install dependencies
install-deps:
	@echo "Installing dependencies..."
	rustup target add wasm32-unknown-unknown
	cargo install --locked soroban-cli

# Deploy to testnet (requires environment variables)
deploy-testnet:
	@echo "Deploying to testnet..."
	@if [ -z "$$SOROBAN_NETWORK_PASSPHRASE" ]; then \
		echo "Error: SOROBAN_NETWORK_PASSPHRASE environment variable is not set"; \
		exit 1; \
	fi
	@if [ -z "$$SOROBAN_RPC_URL" ]; then \
		echo "Error: SOROBAN_RPC_URL environment variable is not set"; \
		exit 1; \
	fi
	soroban contract deploy \
		--wasm target/wasm32-unknown-unknown/release/secure_medical_records_sharing.wasm \
		--network testnet \
		--source-account default

# Deploy to mainnet (requires environment variables and confirmation)
deploy-mainnet:
	@echo "WARNING: Deploying to mainnet!"
	@echo "Make sure you have thoroughly tested on testnet first."
	@read -p "Are you sure you want to deploy to mainnet? (y/N): " confirm; \
	if [ "$$confirm" != "y" ] && [ "$$confirm" != "Y" ]; then \
		echo "Deployment cancelled."; \
		exit 1; \
	fi
	soroban contract deploy \
		--wasm target/wasm32-unknown-unknown/release/secure_medical_records_sharing.wasm \
		--network mainnet \
		--source-account default

# Generate contract bindings
bindings:
	@echo "Generating contract bindings..."
	soroban contract bindings typescript \
		--wasm target/wasm32-unknown-unknown/release/secure_medical_records_sharing.wasm \
		--output-dir ./bindings

# Run security audit
audit:
	@echo "Running security audit..."
	cargo audit

# Check contract size
size:
	@echo "Checking contract size..."
	@ls -lh target/wasm32-unknown-unknown/release/secure_medical_records_sharing.wasm

# Run benchmarks (if any)
bench:
	@echo "Running benchmarks..."
	cargo bench

# Full CI pipeline
ci: format lint build test audit size
	@echo "All CI checks passed!"

# Development setup
dev-setup: install-deps
	@echo "Setting up development environment..."
	@echo "Development environment ready!"

# Run compliance tests
test-compliance:
	@echo "Running HIPAA/GDPR compliance tests..."
	cargo test compliance --features compliance-tests

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --document-private-items

# Help target
help:
	@echo "Available targets:"
	@echo "  all            - Run format, lint, build, and test"
	@echo "  build          - Build the contract"
	@echo "  test           - Run all tests"
	@echo "  test-integration - Run integration tests"
	@echo "  test-compliance  - Run compliance tests"
	@echo "  format         - Format code"
	@echo "  lint           - Run clippy linting"
	@echo "  clean          - Clean build artifacts"
	@echo "  install-deps   - Install required dependencies"
	@echo "  deploy-testnet - Deploy to testnet"
	@echo "  deploy-mainnet - Deploy to mainnet (with confirmation)"
	@echo "  bindings       - Generate TypeScript bindings"
	@echo "  audit          - Run security audit"
	@echo "  size           - Check contract size"
	@echo "  bench          - Run benchmarks"
	@echo "  ci             - Run full CI pipeline"
	@echo "  dev-setup      - Set up development environment"
	@echo "  docs           - Generate documentation"
	@echo "  help           - Show this help message"