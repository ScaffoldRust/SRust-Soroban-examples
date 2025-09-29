# Makefile for Organ Donation Matching Protocol
# Soroban smart contract build and deployment automation

# Contract configuration
CONTRACT_NAME = organ-donation-matching-protocol
CONTRACT_DIR = .
TARGET_DIR = target/wasm32-unknown-unknown/release
WASM_FILE = $(TARGET_DIR)/$(CONTRACT_NAME).wasm
OPTIMIZED_WASM = $(TARGET_DIR)/$(CONTRACT_NAME)-optimized.wasm

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
build: $(WASM_FILE)

$(WASM_FILE):
	@echo "Building $(CONTRACT_NAME)..."
	cargo build --target wasm32-unknown-unknown --release
	@echo "Build completed: $(WASM_FILE)"

# Optimize the WASM file
.PHONY: optimize
optimize: $(OPTIMIZED_WASM)

$(OPTIMIZED_WASM): $(WASM_FILE)
	@echo "Optimizing WASM file..."
	$(SOROBAN_CLI) contract optimize \
		--wasm $(WASM_FILE) \
		--output $(OPTIMIZED_WASM)
	@echo "Optimization completed: $(OPTIMIZED_WASM)"

# Run tests
.PHONY: test
test:
	@echo "Running tests for $(CONTRACT_NAME)..."
	cargo test
	@echo "Tests completed successfully"

# Run tests with output
.PHONY: test-verbose
test-verbose:
	@echo "Running verbose tests for $(CONTRACT_NAME)..."
	cargo test -- --nocapture
	@echo "Verbose tests completed successfully"

# Check code formatting
.PHONY: fmt
fmt:
	@echo "Checking code formatting..."
	cargo fmt --check
	@echo "Code formatting check completed"

# Fix code formatting
.PHONY: fmt-fix
fmt-fix:
	@echo "Fixing code formatting..."
	cargo fmt
	@echo "Code formatting fixed"

# Run clippy lints
.PHONY: clippy
clippy:
	@echo "Running clippy lints..."
	cargo clippy --target wasm32-unknown-unknown -- -D warnings
	@echo "Clippy lints completed"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Clean completed"

# Deploy to testnet
.PHONY: deploy-testnet
deploy-testnet: $(OPTIMIZED_WASM)
	@echo "Deploying $(CONTRACT_NAME) to testnet..."
	$(SOROBAN_CLI) contract deploy \
		--wasm $(OPTIMIZED_WASM) \
		--source-account $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)"
	@echo "Deployment to testnet completed"

# Deploy to futurenet
.PHONY: deploy-futurenet
deploy-futurenet: $(OPTIMIZED_WASM)
	@echo "Deploying $(CONTRACT_NAME) to futurenet..."
	$(SOROBAN_CLI) contract deploy \
		--wasm $(OPTIMIZED_WASM) \
		--source-account $(SOURCE_ACCOUNT) \
		--network futurenet \
		--rpc-url https://rpc-futurenet.stellar.org:443 \
		--network-passphrase "Test SDF Future Network ; October 2022"
	@echo "Deployment to futurenet completed"

# Install contract (for local development)
.PHONY: install
install: $(OPTIMIZED_WASM)
	@echo "Installing $(CONTRACT_NAME) locally..."
	$(SOROBAN_CLI) contract install \
		--wasm $(OPTIMIZED_WASM) \
		--source-account $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)"
	@echo "Installation completed"

# Initialize contract
.PHONY: init
init:
	@echo "Initializing $(CONTRACT_NAME)..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)" \
		-- \
		initialize \
		--admin $(ADMIN_ADDRESS) \
		--medical_authority $(MEDICAL_AUTHORITY_ADDRESS) \
		--max_donors 1000 \
		--max_recipients 2000 \
		--matching_fee 1000
	@echo "Contract initialization completed"

# Register a test donor
.PHONY: register-test-donor
register-test-donor:
	@echo "Registering test donor..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(DONOR_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)" \
		-- \
		register_donor \
		--donor_address $(DONOR_ADDRESS) \
		--blood_type "O+" \
		--hla_type "A1,A2,B1,B2,C1,C2" \
		--organ_types '[{"Kidney":{}},{"Liver":{}}]' \
		--medical_clearance true
	@echo "Test donor registration completed"

# Register a test recipient
.PHONY: register-test-recipient
register-test-recipient:
	@echo "Registering test recipient..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(RECIPIENT_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)" \
		-- \
		register_recipient \
		--recipient_address $(RECIPIENT_ADDRESS) \
		--blood_type "O+" \
		--hla_type "A1,A2,B1,B2,C1,C2" \
		--organ_type '{"Kidney":{}}' \
		--urgency_level 3 \
		--medical_clearance true
	@echo "Test recipient registration completed"

# Find matches
.PHONY: find-matches
find-matches:
	@echo "Finding matches..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(MEDICAL_AUTHORITY_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)" \
		-- \
		find_match \
		--organ_type '{"Kidney":{}}' \
		--max_matches 5
	@echo "Match finding completed"

# Get contract configuration
.PHONY: get-config
get-config:
	@echo "Getting contract configuration..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)" \
		-- \
		get_config
	@echo "Configuration retrieved"

# Get active donors
.PHONY: get-donors
get-donors:
	@echo "Getting active donors..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)" \
		-- \
		get_active_donors
	@echo "Active donors retrieved"

# Get active recipients
.PHONY: get-recipients
get-recipients:
	@echo "Getting active recipients..."
	$(SOROBAN_CLI) contract invoke \
		--id $(CONTRACT_ID) \
		--source-account $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		--rpc-url $(RPC_URL) \
		--network-passphrase "$(NETWORK_PASSPHRASE)" \
		-- \
		get_active_recipients
	@echo "Active recipients retrieved"

# Full deployment workflow
.PHONY: deploy-full
deploy-full: clean build optimize test deploy-testnet init
	@echo "Full deployment workflow completed"

# Development workflow
.PHONY: dev
dev: clean build test
	@echo "Development workflow completed"

# CI/CD workflow
.PHONY: ci
ci: fmt clippy test build optimize
	@echo "CI/CD workflow completed"

# Show help
.PHONY: help
help:
	@echo "Organ Donation Matching Protocol - Available Commands:"
	@echo ""
	@echo "Build Commands:"
	@echo "  build          - Build the contract"
	@echo "  optimize       - Optimize the WASM file"
	@echo "  clean          - Clean build artifacts"
	@echo ""
	@echo "Test Commands:"
	@echo "  test           - Run tests"
	@echo "  test-verbose   - Run tests with output"
	@echo ""
	@echo "Code Quality:"
	@echo "  fmt            - Check code formatting"
	@echo "  fmt-fix        - Fix code formatting"
	@echo "  clippy         - Run clippy lints"
	@echo ""
	@echo "Deployment:"
	@echo "  deploy-testnet - Deploy to testnet"
	@echo "  deploy-futurenet - Deploy to futurenet"
	@echo "  install        - Install contract locally"
	@echo "  init           - Initialize contract"
	@echo ""
	@echo "Contract Operations:"
	@echo "  register-test-donor     - Register a test donor"
	@echo "  register-test-recipient - Register a test recipient"
	@echo "  find-matches           - Find organ matches"
	@echo "  get-config             - Get contract configuration"
	@echo "  get-donors             - Get active donors"
	@echo "  get-recipients         - Get active recipients"
	@echo ""
	@echo "Workflows:"
	@echo "  deploy-full    - Full deployment workflow"
	@echo "  dev            - Development workflow"
	@echo "  ci             - CI/CD workflow"
	@echo ""
	@echo "Environment Variables Required:"
	@echo "  SOURCE_ACCOUNT              - Source account for deployment"
	@echo "  ADMIN_ADDRESS               - Admin address for initialization"
	@echo "  MEDICAL_AUTHORITY_ADDRESS   - Medical authority address"
	@echo "  DONOR_ACCOUNT               - Donor account for registration"
	@echo "  DONOR_ADDRESS               - Donor address"
	@echo "  RECIPIENT_ACCOUNT           - Recipient account for registration"
	@echo "  RECIPIENT_ADDRESS           - Recipient address"
	@echo "  MEDICAL_AUTHORITY_ACCOUNT   - Medical authority account"
	@echo "  CONTRACT_ID                 - Contract ID for operations"
	@echo ""
	@echo "Example Usage:"
	@echo "  make build"
	@echo "  make test"
	@echo "  make deploy-testnet"
	@echo "  make help"
