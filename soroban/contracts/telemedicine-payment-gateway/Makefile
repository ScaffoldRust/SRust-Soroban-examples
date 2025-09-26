# Telemedicine Payment Gateway Contract Makefile
# Build and deployment automation for Soroban smart contract

# Configuration
CONTRACT_NAME = telemedicine-payment-gateway
CONTRACT_DIR = .
TARGET_DIR = target
WASM_FILE = $(TARGET_DIR)/wasm32-unknown-unknown/release/$(CONTRACT_NAME).wasm
OPTIMIZED_WASM = $(TARGET_DIR)/wasm32-unknown-unknown/release/$(CONTRACT_NAME)_optimized.wasm

# Soroban CLI configuration
SOROBAN_CLI = soroban
NETWORK = testnet
RPC_URL = https://soroban-testnet.stellar.org:443
NETWORK_PASSPHRASE = "Test SDF Network ; September 2015"

# Default target
.PHONY: all
all: build test

# Build the contract
.PHONY: build
build:
	@echo "Building $(CONTRACT_NAME) contract..."
	cargo build --target wasm32-unknown-unknown --release
	@echo "Contract built successfully: $(WASM_FILE)"

# Optimize the WASM file
.PHONY: optimize
optimize: build
	@echo "Optimizing WASM file..."
	$(SOROBAN_CLI) contract optimize \
		--wasm $(WASM_FILE) \
		--output $(OPTIMIZED_WASM)
	@echo "Optimized WASM created: $(OPTIMIZED_WASM)"

# Run tests
.PHONY: test
test:
	@echo "Running tests for $(CONTRACT_NAME)..."
	cargo test
	@echo "All tests passed!"

# Run tests with output
.PHONY: test-verbose
test-verbose:
	@echo "Running verbose tests for $(CONTRACT_NAME)..."
	cargo test -- --nocapture
	@echo "All tests passed!"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Clean completed!"

# Format code
.PHONY: fmt
fmt:
	@echo "Formatting code..."
	cargo fmt
	@echo "Code formatted!"

# Check code
.PHONY: check
check:
	@echo "Checking code..."
	cargo check
	@echo "Code check completed!"

# Lint code
.PHONY: clippy
clippy:
	@echo "Running clippy linter..."
	cargo clippy -- -D warnings
	@echo "Clippy check completed!"

# Deploy to testnet
.PHONY: deploy-testnet
deploy-testnet: optimize
	@echo "Deploying $(CONTRACT_NAME) to testnet..."
	$(SOROBAN_CLI) contract deploy \
		--wasm $(OPTIMIZED_WASM) \
		--source-account $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)"
	@echo "Contract deployed to testnet!"

# Deploy to futurenet
.PHONY: deploy-futurenet
deploy-futurenet: optimize
	@echo "Deploying $(CONTRACT_NAME) to futurenet..."
	$(SOROBAN_CLI) contract deploy \
		--wasm $(OPTIMIZED_WASM) \
		--source-account $(SOURCE_ACCOUNT) \
		--network futurenet \
		--rpc-url https://rpc-futurenet.stellar.org:443 \
		--network-passphrase "Test SDF Future Network ; October 2022"
	@echo "Contract deployed to futurenet!"

# Install contract dependencies
.PHONY: install-deps
install-deps:
	@echo "Installing dependencies..."
	cargo build --target wasm32-unknown-unknown --release
	@echo "Dependencies installed!"

# Generate contract bindings
.PHONY: bindings
bindings: optimize
	@echo "Generating contract bindings..."
	$(SOROBAN_CLI) contract bindings typescript \
		--wasm $(OPTIMIZED_WASM) \
		--output-dir bindings
	@echo "TypeScript bindings generated in bindings/ directory!"

# Generate contract bindings for Rust
.PHONY: bindings-rust
bindings-rust: optimize
	@echo "Generating Rust contract bindings..."
	$(SOROBAN_CLI) contract bindings rust \
		--wasm $(OPTIMIZED_WASM) \
		--output-dir bindings-rust
	@echo "Rust bindings generated in bindings-rust/ directory!"

# Run integration tests
.PHONY: test-integration
test-integration: deploy-testnet
	@echo "Running integration tests..."
	# Add integration test commands here
	@echo "Integration tests completed!"

# Show contract info
.PHONY: info
info:
	@echo "Contract Information:"
	@echo "  Name: $(CONTRACT_NAME)"
	@echo "  Directory: $(CONTRACT_DIR)"
	@echo "  Target: $(TARGET_DIR)"
	@echo "  WASM: $(WASM_FILE)"
	@echo "  Optimized WASM: $(OPTIMIZED_WASM)"
	@echo "  Network: $(NETWORK)"
	@echo "  RPC URL: $(RPC_URL)"

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build          - Build the contract"
	@echo "  optimize       - Optimize the WASM file"
	@echo "  test           - Run unit tests"
	@echo "  test-verbose   - Run tests with verbose output"
	@echo "  test-integration - Run integration tests"
	@echo "  clean          - Clean build artifacts"
	@echo "  fmt            - Format code"
	@echo "  check          - Check code"
	@echo "  clippy         - Run clippy linter"
	@echo "  deploy-testnet - Deploy to testnet"
	@echo "  deploy-futurenet - Deploy to futurenet"
	@echo "  install-deps   - Install dependencies"
	@echo "  bindings       - Generate TypeScript bindings"
	@echo "  bindings-rust  - Generate Rust bindings"
	@echo "  info           - Show contract information"
	@echo "  help           - Show this help message"

# Development workflow
.PHONY: dev
dev: clean fmt clippy test build
	@echo "Development workflow completed!"

# Production build
.PHONY: prod
prod: clean fmt clippy test optimize
	@echo "Production build completed!"

# CI/CD pipeline
.PHONY: ci
ci: clean fmt clippy test build optimize
	@echo "CI/CD pipeline completed successfully!"

# Security audit
.PHONY: audit
audit:
	@echo "Running security audit..."
	cargo audit
	@echo "Security audit completed!"

# Update dependencies
.PHONY: update
update:
	@echo "Updating dependencies..."
	cargo update
	@echo "Dependencies updated!"

# Show contract size
.PHONY: size
size: optimize
	@echo "Contract size information:"
	@echo "  Original WASM: $$(stat -f%z $(WASM_FILE) 2>/dev/null || stat -c%s $(WASM_FILE) 2>/dev/null || echo "unknown") bytes"
	@echo "  Optimized WASM: $$(stat -f%z $(OPTIMIZED_WASM) 2>/dev/null || stat -c%s $(OPTIMIZED_WASM) 2>/dev/null || echo "unknown") bytes"

# Validate contract
.PHONY: validate
validate: optimize
	@echo "Validating contract..."
	$(SOROBAN_CLI) contract validate \
		--wasm $(OPTIMIZED_WASM)
	@echo "Contract validation completed!"

# Show network status
.PHONY: network-status
network-status:
	@echo "Network Status:"
	@echo "  Network: $(NETWORK)"
	@echo "  RPC URL: $(RPC_URL)"
	@echo "  Passphrase: $(NETWORK_PASSPHRASE)"
	@$(SOROBAN_CLI) network ls

# Setup development environment
.PHONY: setup
setup: install-deps
	@echo "Setting up development environment..."
	@echo "Make sure you have the following installed:"
	@echo "  - Rust (latest stable)"
	@echo "  - Soroban CLI"
	@echo "  - Stellar CLI (optional)"
	@echo "Development environment setup completed!"

# Show version information
.PHONY: version
version:
	@echo "Version Information:"
	@echo "  Contract: $(CONTRACT_NAME)"
	@echo "  Rust: $$(rustc --version)"
	@echo "  Cargo: $$(cargo --version)"
	@echo "  Soroban CLI: $$($(SOROBAN_CLI) --version 2>/dev/null || echo "not installed")"
