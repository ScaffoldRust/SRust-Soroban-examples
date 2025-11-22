# EHR Interoperability Bridge Makefile

# Default target
.DEFAULT_GOAL := help

# Variables
CONTRACT_NAME = ehr-interoperability-bridge
WASM_TARGET = wasm32-unknown-unknown
BUILD_DIR = target/$(WASM_TARGET)/release
OPTIMIZED_DIR = target/optimized

# Colors for output
GREEN = \033[0;32m
YELLOW = \033[1;33m
RED = \033[0;31m
NC = \033[0m # No Color

## help: Show this help message
.PHONY: help
help:
	@echo "EHR Interoperability Bridge Smart Contract"
	@echo "=========================================="
	@echo ""
	@echo "Available commands:"
	@echo ""
	@sed -n 's/^##//p' $(MAKEFILE_LIST) | column -t -s ':' | sed -e 's/^/ /'

## build: Build the smart contract
.PHONY: build
build:
	@echo "$(GREEN)Building $(CONTRACT_NAME)...$(NC)"
	@cargo build --target $(WASM_TARGET) --release
	@echo "$(GREEN)Build completed successfully!$(NC)"

## test: Run all tests
.PHONY: test
test:
	@echo "$(GREEN)Running tests for $(CONTRACT_NAME)...$(NC)"
	@cargo test
	@echo "$(GREEN)All tests passed!$(NC)"

## test-verbose: Run tests with verbose output
.PHONY: test-verbose
test-verbose:
	@echo "$(GREEN)Running tests with verbose output...$(NC)"
	@cargo test -- --nocapture

## clean: Clean build artifacts
.PHONY: clean
clean:
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	@cargo clean
	@rm -rf $(OPTIMIZED_DIR)
	@echo "$(GREEN)Clean completed!$(NC)"

## check: Check code without building
.PHONY: check
check:
	@echo "$(GREEN)Checking code...$(NC)"
	@cargo check --target $(WASM_TARGET)
	@echo "$(GREEN)Code check completed!$(NC)"

## fmt: Format code
.PHONY: fmt
fmt:
	@echo "$(GREEN)Formatting code...$(NC)"
	@cargo fmt
	@echo "$(GREEN)Code formatted!$(NC)"

## clippy: Run clippy linter
.PHONY: clippy
clippy:
	@echo "$(GREEN)Running clippy...$(NC)"
	@cargo clippy --target $(WASM_TARGET) -- -D warnings
	@echo "$(GREEN)Clippy check completed!$(NC)"

## optimize: Build optimized WASM
.PHONY: optimize
optimize: build
	@echo "$(GREEN)Optimizing WASM...$(NC)"
	@mkdir -p $(OPTIMIZED_DIR)
	@if command -v wasm-opt >/dev/null 2>&1; then \
		wasm-opt -Oz --enable-bulk-memory $(BUILD_DIR)/$(CONTRACT_NAME).wasm -o $(OPTIMIZED_DIR)/$(CONTRACT_NAME).wasm; \
		echo "$(GREEN)WASM optimization completed!$(NC)"; \
	else \
		echo "$(YELLOW)wasm-opt not found, copying unoptimized WASM...$(NC)"; \
		cp $(BUILD_DIR)/$(CONTRACT_NAME).wasm $(OPTIMIZED_DIR)/$(CONTRACT_NAME).wasm; \
	fi

## stellar-build: Build using Soroban CLI
.PHONY: stellar-build
stellar-build:
	@echo "$(GREEN)Building with Soroban CLI...$(NC)"
	@if command -v soroban >/dev/null 2>&1; then \
		soroban contract build; \
		echo "$(GREEN)Soroban build completed!$(NC)"; \
	else \
		echo "$(RED)Soroban CLI not found. Please install it first.$(NC)"; \
		echo "$(YELLOW)Run: cargo install --locked soroban-cli$(NC)"; \
		exit 1; \
	fi

## all: Run all checks and build
.PHONY: all
all: check fmt clippy test build
	@echo "$(GREEN)All tasks completed successfully!$(NC)"

## deploy-testnet: Deploy to Stellar testnet (requires setup)
.PHONY: deploy-testnet
deploy-testnet: stellar-build
	@echo "$(GREEN)Deploying to Stellar testnet...$(NC)"
	@echo "$(YELLOW)Make sure you have configured your Soroban CLI with testnet credentials$(NC)"
	@if command -v soroban >/dev/null 2>&1; then \
		soroban contract deploy \
			--wasm target/wasm32-unknown-unknown/release/$(CONTRACT_NAME).wasm \
			--source alice \
			--network testnet; \
	else \
		echo "$(RED)Soroban CLI not found.$(NC)"; \
		exit 1; \
	fi

## install-deps: Install required dependencies
.PHONY: install-deps
install-deps:
	@echo "$(GREEN)Installing dependencies...$(NC)"
	@rustup target add $(WASM_TARGET)
	@echo "$(GREEN)Installing Soroban CLI...$(NC)"
	@cargo install --locked soroban-cli || echo "$(YELLOW)Soroban CLI already installed$(NC)"
	@echo "$(GREEN)Installing wasm-opt...$(NC)"
	@if command -v npm >/dev/null 2>&1; then \
		npm install -g binaryen; \
	elif command -v brew >/dev/null 2>&1; then \
		brew install binaryen; \
	else \
		echo "$(YELLOW)Please install binaryen manually for WASM optimization$(NC)"; \
	fi
	@echo "$(GREEN)Dependencies installation completed!$(NC)"

## size: Show contract size
.PHONY: size
size: build
	@echo "$(GREEN)Contract size information:$(NC)"
	@ls -lh $(BUILD_DIR)/$(CONTRACT_NAME).wasm | awk '{print "Unoptimized: " $$5}'
	@if [ -f $(OPTIMIZED_DIR)/$(CONTRACT_NAME).wasm ]; then \
		ls -lh $(OPTIMIZED_DIR)/$(CONTRACT_NAME).wasm | awk '{print "Optimized:   " $$5}'; \
	else \
		echo "$(YELLOW)Run 'make optimize' to see optimized size$(NC)"; \
	fi

## watch: Watch for changes and rebuild
.PHONY: watch
watch:
	@echo "$(GREEN)Watching for changes...$(NC)"
	@echo "$(YELLOW)Press Ctrl+C to stop$(NC)"
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x 'check --target $(WASM_TARGET)' -x test; \
	else \
		echo "$(RED)cargo-watch not found. Install with: cargo install cargo-watch$(NC)"; \
		exit 1; \
	fi