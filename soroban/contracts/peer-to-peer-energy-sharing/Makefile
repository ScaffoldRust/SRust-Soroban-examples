# Peer-to-Peer Energy Sharing Contract Makefile

# Configuration
CONTRACT_NAME = peer-to-peer-energy-sharing
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
		--admin $(ADMIN_ADDRESS) \
		--token_contract $(TOKEN_CONTRACT_ADDRESS)
	@echo "Contract initialized!"

# Register prosumer on testnet
.PHONY: register-prosumer-testnet
register-prosumer-testnet:
	@echo "Registering prosumer on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(PROSUMER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- register_prosumer \
		--prosumer $(PROSUMER_ADDRESS)
	@echo "Prosumer registered!"

# Create energy sharing agreement
.PHONY: create-agreement-testnet
create-agreement-testnet:
	@echo "Creating energy sharing agreement on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(PROVIDER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- create_agreement \
		--provider $(PROVIDER_ADDRESS) \
		--consumer $(CONSUMER_ADDRESS) \
		--energy_amount_kwh $(ENERGY_AMOUNT_KWH) \
		--price_per_kwh $(PRICE_PER_KWH) \
		--delivery_deadline $(DELIVERY_DEADLINE)
	@echo "Agreement created!"

# Deliver energy
.PHONY: deliver-energy-testnet
deliver-energy-testnet:
	@echo "Delivering energy on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(PROVIDER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- deliver_energy \
		--agreement_id $(AGREEMENT_ID) \
		--energy_delivered_kwh $(ENERGY_DELIVERED_KWH) \
		--meter_reading $(METER_READING) \
		--provider $(PROVIDER_ADDRESS)
	@echo "Energy delivered!"

# Settle payment
.PHONY: settle-payment-testnet
settle-payment-testnet:
	@echo "Settling payment on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(SETTLER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- settle_payment \
		--transaction_id $(TRANSACTION_ID) \
		--settler $(SETTLER_ADDRESS)
	@echo "Payment settled!"

# Get transaction history
.PHONY: get-history-testnet
get-history-testnet:
	@echo "Getting transaction history from testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- get_transaction_history \
		--prosumer $(PROSUMER_ADDRESS)
	@echo "Transaction history retrieved!"

# Get agreement details
.PHONY: get-agreement-testnet
get-agreement-testnet:
	@echo "Getting agreement details from testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- get_agreement \
		--agreement_id $(AGREEMENT_ID)
	@echo "Agreement details retrieved!"

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
	@echo "  build                     - Build the contract"
	@echo "  soroban-build            - Build with Soroban CLI"
	@echo "  optimize                 - Optimize the WASM file"
	@echo "  test                     - Run tests"
	@echo "  test-verbose             - Run tests with verbose output"
	@echo "  clean                    - Clean build artifacts"
	@echo "  format                   - Format code"
	@echo "  check                    - Check code"
	@echo "  clippy                   - Run clippy linting"
	@echo "  check-all                - Run all checks (format, clippy, test)"
	@echo "  deploy-testnet           - Deploy to testnet"
	@echo "  init-testnet             - Initialize contract on testnet"
	@echo "  register-prosumer-testnet - Register a prosumer on testnet"
	@echo "  create-agreement-testnet - Create energy sharing agreement"
	@echo "  deliver-energy-testnet   - Deliver energy with meter verification"
	@echo "  settle-payment-testnet   - Settle payment for delivered energy"
	@echo "  get-history-testnet      - Get transaction history"
	@echo "  get-agreement-testnet    - Get agreement details"
	@echo "  info                     - Show contract information"
	@echo "  help                     - Show this help message"

# Development workflow
.PHONY: dev
dev: clean build test
	@echo "Development build completed!"

# Production workflow
.PHONY: prod
prod: clean check-all optimize
	@echo "Production build completed!"