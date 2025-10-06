# Grid Load Balancing Incentives Contract Makefile

# Configuration
CONTRACT_NAME = grid-load-balancing-incentives
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
	cargo build
	@echo "Build completed successfully!"

# Build with Soroban CLI
.PHONY: soroban-build
soroban-build:
	@echo "Building with Soroban CLI..."
	$(SOROBAN_CLI) contract build
	@echo "Soroban build completed successfully!"

# Optimize the WASM file (requires wasm32-unknown-unknown target)
.PHONY: optimize
optimize:
	@echo "Building for WASM target..."
	cargo build --release --target wasm32-unknown-unknown
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

# Register grid operator on testnet
.PHONY: register-grid-operator-testnet
register-grid-operator-testnet:
	@echo "Registering grid operator on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(GRID_OPERATOR_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- register_grid_operator \
		--grid_operator $(GRID_OPERATOR_ADDRESS)
	@echo "Grid operator registered!"

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

# Start demand response event
.PHONY: start-event-testnet
start-event-testnet:
	@echo "Starting demand response event on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(GRID_OPERATOR_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- start_event \
		--grid_operator $(GRID_OPERATOR_ADDRESS) \
		--target_reduction_kw $(TARGET_REDUCTION_KW) \
		--reward_per_kw $(REWARD_PER_KW) \
		--duration_seconds $(DURATION_SECONDS)
	@echo "Event started!"

# Verify load reduction
.PHONY: verify-reduction-testnet
verify-reduction-testnet:
	@echo "Verifying load reduction on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(CONSUMER_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- verify_reduction \
		--event_id $(EVENT_ID) \
		--consumer $(CONSUMER_ADDRESS) \
		--baseline_usage_kw $(BASELINE_USAGE_KW) \
		--actual_usage_kw $(ACTUAL_USAGE_KW) \
		--meter_reading_start $(METER_READING_START) \
		--meter_reading_end $(METER_READING_END)
	@echo "Reduction verified!"

# Distribute rewards
.PHONY: distribute-rewards-testnet
distribute-rewards-testnet:
	@echo "Distributing rewards on testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(DISTRIBUTOR_KEY) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- distribute_rewards \
		--event_id $(EVENT_ID) \
		--distributor $(DISTRIBUTOR_ADDRESS)
	@echo "Rewards distributed!"

# Get consumer participations
.PHONY: get-participations-testnet
get-participations-testnet:
	@echo "Getting consumer participations from testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- get_consumer_participations \
		--consumer $(CONSUMER_ADDRESS)
	@echo "Participations retrieved!"

# Get event details
.PHONY: get-event-testnet
get-event-testnet:
	@echo "Getting event details from testnet..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		-- get_event \
		--event_id $(EVENT_ID)
	@echo "Event details retrieved!"

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
	@echo "  build                        - Build the contract"
	@echo "  soroban-build               - Build with Soroban CLI"
	@echo "  optimize                    - Optimize the WASM file"
	@echo "  test                        - Run tests"
	@echo "  test-verbose                - Run tests with verbose output"
	@echo "  clean                       - Clean build artifacts"
	@echo "  format                      - Format code"
	@echo "  check                       - Check code"
	@echo "  clippy                      - Run clippy linting"
	@echo "  check-all                   - Run all checks (format, clippy, test)"
	@echo "  deploy-testnet              - Deploy to testnet"
	@echo "  init-testnet                - Initialize contract on testnet"
	@echo "  register-grid-operator-testnet - Register a grid operator on testnet"
	@echo "  register-consumer-testnet   - Register a consumer on testnet"
	@echo "  start-event-testnet         - Start demand response event"
	@echo "  verify-reduction-testnet    - Verify consumer load reduction"
	@echo "  distribute-rewards-testnet  - Distribute rewards to consumers"
	@echo "  get-participations-testnet  - Get consumer participations"
	@echo "  get-event-testnet           - Get event details"
	@echo "  info                        - Show contract information"
	@echo "  help                        - Show this help message"

# Development workflow
.PHONY: dev
dev: clean build test
	@echo "Development build completed!"

# Production workflow
.PHONY: prod
prod: clean check-all optimize
	@echo "Production build completed!"