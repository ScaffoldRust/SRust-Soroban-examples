# Energy Trading Marketplace Contract Makefile
# This Makefile provides commands for building, testing, and deploying the energy trading contract

# Configuration
CONTRACT_NAME = energy-trading-marketplace
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
		--admin $(ADMIN_ADDRESS) \
		--token_contract $(TOKEN_CONTRACT_ADDRESS) \
		--min_trade_size $(MIN_TRADE_SIZE) \
		--max_trade_size $(MAX_TRADE_SIZE)
	@echo "Contract initialized!"

# Register producer on testnet
.PHONY: register-producer-testnet
register-producer-testnet:
	@echo "Registering producer on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(PRODUCER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- register_producer \
		--producer $(PRODUCER_ADDRESS)
	@echo "Producer registered!"

# Register consumer on testnet
.PHONY: register-consumer-testnet
register-consumer-testnet:
	@echo "Registering consumer on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(CONSUMER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- register_consumer \
		--consumer $(CONSUMER_ADDRESS)
	@echo "Consumer registered!"

# Place energy order on testnet
.PHONY: place-order-testnet
place-order-testnet:
	@echo "Placing energy order on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(TRADER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- place_order \
		--trader $(TRADER_ADDRESS) \
		--order_type $(ORDER_TYPE) \
		--quantity_kwh $(QUANTITY_KWH) \
		--price_per_kwh $(PRICE_PER_KWH)
	@echo "Order placed!"

# Get order info from testnet
.PHONY: get-order-testnet
get-order-testnet:
	@echo "Getting order info from testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- get_order \
		--order_id $(ORDER_ID)
	@echo "Order info retrieved!"

# Cancel order on testnet
.PHONY: cancel-order-testnet
cancel-order-testnet:
	@echo "Cancelling order on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(TRADER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- cancel_order \
		--trader $(TRADER_ADDRESS) \
		--order_id $(ORDER_ID)
	@echo "Order cancelled!"

# Settle trade on testnet
.PHONY: settle-trade-testnet
settle-trade-testnet:
	@echo "Settling trade on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(SETTLER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- settle_trade \
		--trade_id $(TRADE_ID) \
		--settler $(SETTLER_ADDRESS)
	@echo "Trade settled!"

# Get trade history from testnet
.PHONY: get-trade-history-testnet
get-trade-history-testnet:
	@echo "Getting trade history from testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- get_trade_history \
		--trader $(TRADER_ADDRESS)
	@echo "Trade history retrieved!"

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
	@echo "  register-producer-testnet - Register a producer on testnet"
	@echo "  register-consumer-testnet - Register a consumer on testnet"
	@echo "  place-order-testnet - Place an energy order on testnet"
	@echo "  get-order-testnet  - Get order info from testnet"
	@echo "  cancel-order-testnet - Cancel order on testnet"
	@echo "  settle-trade-testnet - Settle trade on testnet"
	@echo "  get-trade-history-testnet - Get trade history from testnet"
	@echo "  generate-test-account - Generate test account"
	@echo "  fund-test-account  - Fund test account"
	@echo "  info               - Show contract information"
	@echo "  help               - Show this help message"
	@echo ""
	@echo "Environment variables:"
	@echo "  ADMIN_KEY          - Admin account secret key"
	@echo "  ADMIN_ADDRESS      - Admin account address"
	@echo "  PRODUCER_KEY       - Producer account secret key"
	@echo "  PRODUCER_ADDRESS   - Producer account address"
	@echo "  CONSUMER_KEY       - Consumer account secret key"
	@echo "  CONSUMER_ADDRESS   - Consumer account address"
	@echo "  TRADER_KEY         - Trader account secret key"
	@echo "  TRADER_ADDRESS     - Trader account address"
	@echo "  SETTLER_KEY        - Settler account secret key"
	@echo "  SETTLER_ADDRESS    - Settler account address"
	@echo "  CONTRACT_ID        - Deployed contract ID"
	@echo "  TOKEN_CONTRACT_ADDRESS - Token contract address"
	@echo "  ORDER_ID           - Order identifier"
	@echo "  TRADE_ID           - Trade identifier"
	@echo "  ORDER_TYPE         - Order type (Buy or Sell)"
	@echo "  QUANTITY_KWH       - Energy quantity in kWh"
	@echo "  PRICE_PER_KWH      - Price per kWh"
	@echo "  MIN_TRADE_SIZE     - Minimum trade size"
	@echo "  MAX_TRADE_SIZE     - Maximum trade size"
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

# Validate contract
.PHONY: validate
validate: build test clippy
	@echo "Contract validation completed successfully!"

# Full deployment pipeline
.PHONY: deploy-pipeline
deploy-pipeline: clean check-all optimize deploy-testnet init-testnet
	@echo "Full deployment pipeline completed!"