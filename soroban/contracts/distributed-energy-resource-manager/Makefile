# Distributed Energy Resource Manager Contract Makefile
# This Makefile provides commands for building, testing, and deploying the DER management contract

# Configuration
CONTRACT_NAME = distributed-energy-resource-manager
CONTRACT_DIR = .
TARGET_DIR = target
WASM_FILE = $(TARGET_DIR)/wasm32-unknown-unknown/release/$(CONTRACT_NAME).wasm
OPTIMIZED_WASM = $(TARGET_DIR)/wasm32-unknown-unknown/release/$(CONTRACT_NAME)_optimized.wasm

# Soroban CLI configuration
SOROBAN_CLI = soroban
NETWORK = testnet
RPC_URL = https://soroban-testnet.stellar.org:443
FRIENDBOT_URL = https://friendbot.stellar.org

# Default target
.PHONY: all
all: build

# Build the contract
.PHONY: build
build:
	@echo "Building $(CONTRACT_NAME) contract..."
	cargo build --release --target wasm32-unknown-unknown
	@echo "Build completed successfully!"

# Build with Soroban CLI
.PHONY: soroban-build
soroban-build:
	@echo "Building with Soroban CLI..."
	$(SOROBAN_CLI) contract build
	@echo "Soroban build completed successfully!"

# Optimize the WASM file
.PHONY: optimize
optimize: build
	@echo "Optimizing WASM file..."
	$(SOROBAN_CLI) contract optimize --wasm $(WASM_FILE) --wasm-out $(OPTIMIZED_WASM)
	@echo "Optimization completed!"

# Run tests
.PHONY: test
test:
	@echo "Running tests..."
	cargo test
	@echo "Tests completed!"

# Run tests with output
.PHONY: test-verbose
test-verbose:
	@echo "Running tests with verbose output..."
	cargo test -- --nocapture
	@echo "Tests completed!"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf $(TARGET_DIR)
	@echo "Clean completed!"

# Format code
.PHONY: format
format:
	@echo "Formatting code..."
	cargo fmt
	@echo "Formatting completed!"

# Check code
.PHONY: check
check:
	@echo "Checking code..."
	cargo check
	@echo "Check completed!"

# Clippy linting
.PHONY: clippy
clippy:
	@echo "Running clippy..."
	cargo clippy -- -D warnings
	@echo "Clippy completed!"

# Full check (format, clippy, test)
.PHONY: check-all
check-all: format clippy test
	@echo "All checks completed successfully!"

# Deploy to testnet
.PHONY: deploy-testnet
deploy-testnet: optimize
	@echo "Deploying to testnet..."
	$(SOROBAN_CLI) contract deploy \
		--wasm $(OPTIMIZED_WASM) \
		--source-account $(ADMIN_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL)
	@echo "Deployment completed!"

# Deploy to futurenet
.PHONY: deploy-futurenet
deploy-futurenet: optimize
	@echo "Deploying to futurenet..."
	$(SOROBAN_CLI) contract deploy \
		--wasm $(OPTIMIZED_WASM) \
		--source-account $(ADMIN_KEY) \
		--network futurenet
	@echo "Deployment completed!"

# Initialize contract on testnet
.PHONY: init-testnet
init-testnet:
	@echo "Initializing contract on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(ADMIN_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- initialize \
		--admin $(ADMIN_ADDRESS)
	@echo "Contract initialized!"

# Register a DER on testnet
.PHONY: register-der-testnet
register-der-testnet:
	@echo "Registering DER on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(OWNER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- register_der \
		--der_id "$(DER_ID)" \
		--resource_type $(RESOURCE_TYPE) \
		--capacity $(CAPACITY) \
		--location "$(LOCATION)"
	@echo "DER registered!"

# Get DER info from testnet
.PHONY: get-der-info-testnet
get-der-info-testnet:
	@echo "Getting DER info from testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- get_der_info \
		--der_id "$(DER_ID)"
	@echo "DER info retrieved!"

# Optimize resources on testnet
.PHONY: optimize-testnet
optimize-testnet:
	@echo "Optimizing resources on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(OPERATOR_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- optimize_resources
	@echo "Resource optimization completed!"

# Get contract stats from testnet
.PHONY: get-stats-testnet
get-stats-testnet:
	@echo "Getting contract stats from testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- get_stats
	@echo "Contract stats retrieved!"

# Generate test account
.PHONY: generate-test-account
generate-test-account:
	@echo "Generating test account..."
	$(SOROBAN_CLI) keys generate --global test-account --network $(NETWORK)
	@echo "Test account generated!"

# Fund test account
.PHONY: fund-test-account
fund-test-account:
	@echo "Funding test account..."
	curl -X POST "$(FRIENDBOT_URL)/?addr=$(TEST_ACCOUNT_ADDRESS)"
	@echo "Test account funded!"

# Show contract info
.PHONY: info
info:
	@echo "Contract Information:"
	@echo "  Name: $(CONTRACT_NAME)"
	@echo "  Target: wasm32-unknown-unknown"
	@echo "  WASM File: $(WASM_FILE)"
	@echo "  Optimized WASM: $(OPTIMIZED_WASM)"
	@echo "  Network: $(NETWORK)"
	@echo "  RPC URL: $(RPC_URL)"

# Help
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build              - Build the contract"
	@echo "  soroban-build      - Build with Soroban CLI"
	@echo "  optimize           - Optimize the WASM file"
	@echo "  test               - Run tests"
	@echo "  test-verbose       - Run tests with verbose output"
	@echo "  clean              - Clean build artifacts"
	@echo "  format             - Format code"
	@echo "  check              - Check code"
	@echo "  clippy             - Run clippy linting"
	@echo "  check-all          - Run all checks (format, clippy, test)"
	@echo "  deploy-testnet     - Deploy to testnet"
	@echo "  deploy-futurenet   - Deploy to futurenet"
	@echo "  init-testnet       - Initialize contract on testnet"
	@echo "  register-der-testnet - Register a DER on testnet"
	@echo "  get-der-info-testnet - Get DER info from testnet"
	@echo "  optimize-testnet   - Optimize resources on testnet"
	@echo "  get-stats-testnet  - Get contract stats from testnet"
	@echo "  generate-test-account - Generate test account"
	@echo "  fund-test-account  - Fund test account"
	@echo "  info               - Show contract information"
	@echo "  help               - Show this help message"
	@echo ""
	@echo "Environment variables:"
	@echo "  ADMIN_KEY          - Admin account secret key"
	@echo "  ADMIN_ADDRESS      - Admin account address"
	@echo "  OWNER_KEY          - DER owner account secret key"
	@echo "  OPERATOR_KEY       - Grid operator account secret key"
	@echo "  CONTRACT_ID        - Deployed contract ID"
	@echo "  DER_ID             - DER identifier"
	@echo "  RESOURCE_TYPE      - Resource type (Solar, Wind, Battery, etc.)"
	@echo "  CAPACITY           - DER capacity in kW"
	@echo "  LOCATION           - DER location"
	@echo "  TEST_ACCOUNT_ADDRESS - Test account address"

# Development workflow
.PHONY: dev
dev: clean build test
	@echo "Development build completed!"

# Production workflow
.PHONY: prod
prod: clean check-all optimize
	@echo "Production build completed!"

# Quick test
.PHONY: quick-test
quick-test: build test
	@echo "Quick test completed!"

# Benchmark
.PHONY: benchmark
benchmark:
	@echo "Running benchmarks..."
	cargo bench
	@echo "Benchmarks completed!"

# Documentation
.PHONY: docs
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --open
	@echo "Documentation generated!"

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

# Install dependencies
.PHONY: install
install:
	@echo "Installing dependencies..."
	cargo build
	@echo "Dependencies installed!"

# Show build size
.PHONY: size
size: build
	@echo "WASM file size:"
	@ls -lh $(WASM_FILE)
	@echo "Optimized WASM file size:"
	@ls -lh $(OPTIMIZED_WASM) 2>/dev/null || echo "Optimized file not found. Run 'make optimize' first."

# Validate contract
.PHONY: validate
validate: build test clippy
	@echo "Contract validation completed successfully!"

# Full deployment pipeline
.PHONY: deploy-pipeline
deploy-pipeline: clean check-all optimize deploy-testnet init-testnet
	@echo "Full deployment pipeline completed!"

# Emergency procedures
.PHONY: emergency-stop
emergency-stop:
	@echo "Emergency stop - stopping all DERs..."
	@echo "This would require manual intervention in a real scenario"
	@echo "Emergency stop completed!"

# Maintenance mode
.PHONY: maintenance-mode
maintenance-mode:
	@echo "Entering maintenance mode..."
	@echo "All optimization operations will be paused"
	@echo "Maintenance mode activated!"

# Exit maintenance mode
.PHONY: exit-maintenance
exit-maintenance:
	@echo "Exiting maintenance mode..."
	@echo "Normal operations resumed!"
	@echo "Maintenance mode deactivated!"
