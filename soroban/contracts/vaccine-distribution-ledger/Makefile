# Vaccine Distribution Ledger Smart Contract
# Makefile for building and deploying on Stellar Soroban

# Default target
.DEFAULT_GOAL := help

# Contract name
CONTRACT_NAME := vaccine-distribution-ledger

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

.PHONY: create-batch
create-batch: ## Create a test vaccine batch
	@echo "Creating test batch..."
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source-account default \
		--network testnet \
		-- \
		initialize_batch \
		--batch_id "TEST_BATCH_001" \
		--manufacturer $(MANUFACTURER_ADDRESS) \
		--vaccine_type "COVID-19 mRNA" \
		--production_date 1640995200 \
		--quantity 1000 \
		--expiry_date 1672531200

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
	stellar keys generate manufacturer --network testnet
	stellar keys generate distributor --network testnet
	stellar keys generate administrator --network testnet
	@echo "Test keypairs generated!"

# Fund test accounts
.PHONY: fund-accounts
fund-accounts: ## Fund test accounts with XLM
	@echo "Funding test accounts..."
	stellar keys fund admin --network testnet
	stellar keys fund manufacturer --network testnet
	stellar keys fund distributor --network testnet
	stellar keys fund administrator --network testnet
	@echo "Test accounts funded!"

# Full deployment workflow
.PHONY: full-deploy
full-deploy: check deploy-testnet ## Full deployment workflow (check + deploy)
	@echo "Full deployment complete!"

# Help target
.PHONY: help
help: ## Show this help message
	@echo "Vaccine Distribution Ledger Smart Contract"
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)