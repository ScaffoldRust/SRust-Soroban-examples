# Energy Project Funding Escrow - Soroban Smart Contract
# Makefile for build, test, and deployment automation

# Contract configuration
CONTRACT_NAME = energy-project-funding-escrow
WASM_TARGET = wasm32-unknown-unknown
NETWORK = testnet
SOROBAN_RPC_URL = https://soroban-testnet.stellar.org:443

# Build directories
BUILD_DIR = target
WASM_DIR = $(BUILD_DIR)/$(WASM_TARGET)/release

# Default target
.PHONY: all
all: build optimize

# Install dependencies and setup environment
.PHONY: install
install:
	@echo "Installing Rust and Soroban CLI..."
	rustup target add $(WASM_TARGET)
	cargo install --locked soroban-cli
	@echo "Installation complete!"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf .soroban
	@echo "Clean complete!"

# Build the contract
.PHONY: build
build:
	@echo "Building $(CONTRACT_NAME) contract..."
	cargo build --target $(WASM_TARGET) --release
	@echo "Build complete!"

# Optimize the WebAssembly binary
.PHONY: optimize
optimize: build
	@echo "Optimizing contract binary..."
	soroban contract optimize --wasm $(WASM_DIR)/$(CONTRACT_NAME).wasm
	@echo "Optimization complete!"

# Run tests
.PHONY: test
test:
	@echo "Running contract tests..."
	cargo test
	@echo "Tests complete!"

# Run tests with coverage
.PHONY: test-coverage
test-coverage:
	@echo "Running tests with coverage..."
	cargo test --features testutils
	@echo "Coverage tests complete!"

# Lint the code
.PHONY: lint
lint:
	@echo "Running clippy linter..."
	cargo clippy --target $(WASM_TARGET) --release -- -D warnings
	@echo "Linting complete!"

# Format the code
.PHONY: format
format:
	@echo "Formatting code..."
	cargo fmt
	@echo "Formatting complete!"

# Check formatting
.PHONY: check-format
check-format:
	@echo "Checking code formatting..."
	cargo fmt -- --check
	@echo "Format check complete!"

# Full quality check
.PHONY: check
check: check-format lint test
	@echo "All quality checks passed!"

# Deploy to testnet
.PHONY: deploy-testnet
deploy-testnet: optimize
	@echo "Deploying to $(NETWORK)..."
	soroban contract deploy \
		--wasm $(WASM_DIR)/$(CONTRACT_NAME).wasm \
		--source-account default \
		--network $(NETWORK)
	@echo "Deployment to $(NETWORK) complete!"

# Install contract on testnet
.PHONY: install-testnet
install-testnet: optimize
	@echo "Installing contract on $(NETWORK)..."
	soroban contract install \
		--wasm $(WASM_DIR)/$(CONTRACT_NAME).wasm \
		--source-account default \
		--network $(NETWORK)
	@echo "Installation on $(NETWORK) complete!"

# Initialize contract on testnet
.PHONY: init-testnet
init-testnet:
	@echo "Initializing contract on $(NETWORK)..."
	soroban contract invoke \
		--source-account default \
		--network $(NETWORK) \
		--id $(CONTRACT_ID) \
		-- initialize
	@echo "Contract initialized on $(NETWORK)!"

# Generate bindings for JavaScript/TypeScript
.PHONY: bindings
bindings: optimize
	@echo "Generating contract bindings..."
	soroban contract bindings typescript \
		--wasm $(WASM_DIR)/$(CONTRACT_NAME).wasm \
		--output-dir ./bindings
	@echo "Bindings generated in ./bindings directory!"

# Generate contract documentation
.PHONY: docs
docs:
	@echo "Generating contract documentation..."
	cargo doc --open --no-deps
	@echo "Documentation generated!"

# Setup local development environment
.PHONY: dev-setup
dev-setup:
	@echo "Setting up local development environment..."
	soroban config network add local \
		--rpc-url http://localhost:8000/soroban/rpc \
		--network-passphrase "Standalone Network ; February 2017"
	soroban config identity generate default
	@echo "Local development setup complete!"

# Fund account for testing
.PHONY: fund-account
fund-account:
	@echo "Funding account for testing..."
	soroban config identity fund default --network $(NETWORK)
	@echo "Account funded!"

# Build for production
.PHONY: build-prod
build-prod: clean check optimize
	@echo "Production build complete!"

# Deploy to mainnet (requires confirmation)
.PHONY: deploy-mainnet
deploy-mainnet:
	@echo "WARNING: This will deploy to Stellar mainnet!"
	@echo "Are you sure? (y/N)" && read ans && [ $${ans:-N} = y ]
	@echo "Deploying to mainnet..."
	soroban contract deploy \
		--wasm $(WASM_DIR)/$(CONTRACT_NAME).wasm \
		--source-account mainnet \
		--network mainnet
	@echo "Mainnet deployment complete!"

# Verify contract deployment
.PHONY: verify
verify:
	@echo "Verifying contract deployment..."
	soroban contract invoke \
		--source-account default \
		--network $(NETWORK) \
		--id $(CONTRACT_ID) \
		-- --help
	@echo "Contract verification complete!"

# Watch mode for development
.PHONY: watch
watch:
	@echo "Starting watch mode..."
	cargo watch -x "build --target $(WASM_TARGET) --release"

# Performance benchmarks
.PHONY: bench
bench:
	@echo "Running performance benchmarks..."
	cargo bench --features testutils
	@echo "Benchmarks complete!"

# Security audit
.PHONY: audit
audit:
	@echo "Running security audit..."
	cargo audit
	@echo "Security audit complete!"

# Complete development workflow
.PHONY: dev
dev: format check build optimize test
	@echo "Development workflow complete!"

# Complete CI/CD workflow
.PHONY: ci
ci: install check-format lint test build optimize
	@echo "CI/CD workflow complete!"

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  install          - Install dependencies and setup environment"
	@echo "  build           - Build the contract"
	@echo "  optimize        - Optimize the WebAssembly binary"
	@echo "  test            - Run tests"
	@echo "  lint            - Run clippy linter"
	@echo "  format          - Format code"
	@echo "  check           - Run all quality checks"
	@echo "  clean           - Clean build artifacts"
	@echo "  deploy-testnet  - Deploy to testnet"
	@echo "  deploy-mainnet  - Deploy to mainnet (with confirmation)"
	@echo "  bindings        - Generate TypeScript bindings"
	@echo "  docs            - Generate documentation"
	@echo "  dev-setup       - Setup local development environment"
	@echo "  dev             - Complete development workflow"
	@echo "  ci              - Complete CI/CD workflow"
	@echo "  help            - Show this help message"

# Set default shell
SHELL := /bin/bash

# Ensure targets don't conflict with files
.SUFFIXES: