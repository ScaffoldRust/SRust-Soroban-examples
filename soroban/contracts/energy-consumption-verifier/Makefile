# Energy Consumption Verifier Smart Contract
# Makefile for building and deploying on Stellar Soroban

# Default target
.DEFAULT_GOAL := help

# Contract name
CONTRACT_NAME := energy-consumption-verifier

# Network configurations
TESTNET_RPC_URL := https://soroban-testnet.stellar.org:443
FUTURENET_RPC_URL := https://rpc-futurenet.stellar.org:443

# Build targets
.PHONY: build
build: ## Build the smart contract
	@echo "Building $(CONTRACT_NAME)..."
	cargo build --target wasm32-unknown-unknown --release
	@echo "Build complete!"

.PHONY: optimize
optimize: build ## Build and optimize the contract
	@echo "Optimizing WASM binary..."
	stellar contract optimize --wasm target/wasm32-unknown-unknown/release/$(CONTRACT_NAME).wasm
	@echo "Optimization complete!"

.PHONY: test
test: ## Run contract tests
	@echo "Running tests..."
	cargo test
	@echo "Tests complete!"

# Stellar CLI targets
.PHONY: stellar-build
stellar-build: ## Build using Stellar CLI
	@echo "Building with Stellar CLI..."
	stellar contract build
	@echo "Stellar build complete!"

# Deployment targets
.PHONY: deploy-testnet
deploy-testnet: stellar-build ## Deploy to Stellar testnet
	@echo "Deploying to testnet..."
	stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/$(CONTRACT_NAME).wasm \
		--source-account default \
		--network testnet
	@echo "Testnet deployment complete!"

# Development utilities
.PHONY: clean
clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf target/
	@echo "Clean complete!"

.PHONY: fmt
fmt: ## Format code
	@echo "Formatting code..."
	cargo fmt
	@echo "Formatting complete!"

.PHONY: clippy
clippy: ## Run clippy linter
	@echo "Running clippy..."
	cargo clippy -- -D warnings
	@echo "Clippy complete!"

.PHONY: check
check: fmt clippy test ## Run all checks (format, lint, test)
	@echo "All checks passed!"

# Contract interaction helpers
.PHONY: init-contract
init-contract: ## Initialize the deployed contract
	@echo "Initializing contract..."
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source-account default \
		--network testnet \
		-- \
		initialize \
		--admin $(ADMIN_ADDRESS)

.PHONY: submit-test-data
submit-test-data: ## Submit test consumption data
	@echo "Submitting test consumption data..."
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source-account default \
		--network testnet \
		-- \
		submit_data \
		--consumer $(CONSUMER_ADDRESS) \
		--meter_id "METER_001" \
		--consumption_kwh 100 \
		--meter_reading 12345 \
		--temperature 25 \
		--voltage 230

.PHONY: verify-test-data
verify-test-data: ## Verify test consumption data
	@echo "Verifying test consumption data..."
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source-account default \
		--network testnet \
		-- \
		verify_data \
		--verifier $(VERIFIER_ADDRESS) \
		--record_id $(RECORD_ID) \
		--status 1 \
		--comments "Data verified successfully"

.PHONY: get-verification
get-verification: ## Get verification status
	@echo "Getting verification status..."
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source-account default \
		--network testnet \
		-- \
		get_verification \
		--record_id $(RECORD_ID)

.PHONY: get-audit-log
get-audit-log: ## Get audit log
	@echo "Getting audit log..."
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source-account default \
		--network testnet \
		-- \
		audit_log \
		--offset 0 \
		--limit 10

# Documentation
.PHONY: docs
docs: ## Generate documentation
	@echo "Generating documentation..."
	cargo doc --no-deps --open
	@echo "Documentation generated!"

# Setup development environment
.PHONY: setup
setup: ## Setup development environment
	@echo "Setting up development environment..."
	rustup target add wasm32-unknown-unknown
	cargo install stellar-cli
	stellar network add testnet --rpc-url $(TESTNET_RPC_URL) --network-passphrase "Test SDF Network ; September 2015"
	stellar network add futurenet --rpc-url $(FUTURENET_RPC_URL) --network-passphrase "Test SDF Future Network ; October 2022"
	@echo "Development environment setup complete!"

# Generate keypairs for testing
.PHONY: generate-keypairs
generate-keypairs: ## Generate test keypairs
	@echo "Generating test keypairs..."
	stellar keys generate admin --network testnet
	stellar keys generate verifier --network testnet
	stellar keys generate consumer --network testnet
	stellar keys generate meter --network testnet
	@echo "Test keypairs generated!"

# Fund test accounts
.PHONY: fund-accounts
fund-accounts: ## Fund test accounts with XLM
	@echo "Funding test accounts..."
	stellar keys fund admin --network testnet
	stellar keys fund verifier --network testnet
	stellar keys fund consumer --network testnet
	stellar keys fund meter --network testnet
	@echo "Test accounts funded!"

# Register test verifier
.PHONY: register-verifier
register-verifier: ## Register a test verifier
	@echo "Registering test verifier..."
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source-account default \
		--network testnet \
		-- \
		register_verifier \
		--admin $(ADMIN_ADDRESS) \
		--verifier $(VERIFIER_ADDRESS)

# Register test meter
.PHONY: register-meter
register-meter: ## Register a test meter
	@echo "Registering test meter..."
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source-account default \
		--network testnet \
		-- \
		register_meter \
		--admin $(ADMIN_ADDRESS) \
		--meter_id "METER_001" \
		--meter_address $(METER_ADDRESS)

# Full deployment workflow
.PHONY: full-deploy
full-deploy: check deploy-testnet ## Full deployment workflow (check + deploy)
	@echo "Full deployment complete!"

# Integration test workflow
.PHONY: integration-test
integration-test: ## Run full integration test
	@echo "Running integration tests..."
	make init-contract ADMIN_ADDRESS=$$ADMIN_ADDR CONTRACT_ID=$$CONTRACT_ID
	make register-verifier ADMIN_ADDRESS=$$ADMIN_ADDR VERIFIER_ADDRESS=$$VERIFIER_ADDR CONTRACT_ID=$$CONTRACT_ID
	make register-meter ADMIN_ADDRESS=$$ADMIN_ADDR METER_ADDRESS=$$METER_ADDR CONTRACT_ID=$$CONTRACT_ID
	make submit-test-data CONSUMER_ADDRESS=$$CONSUMER_ADDR CONTRACT_ID=$$CONTRACT_ID
	@echo "Integration tests complete!"

# Help target
.PHONY: help
help: ## Show this help message
	@echo "Energy Consumption Verifier Smart Contract"
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)