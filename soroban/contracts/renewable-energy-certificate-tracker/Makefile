# Renewable Energy Certificate Tracker Contract Makefile

.PHONY: build test clean deploy

# Build the contract
build:
	cargo build --target wasm32-unknown-unknown --release

# Build using Stellar CLI
stellar-build:
	stellar contract build

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean
	rm -f *.wasm

# Deploy contract (requires Stellar CLI and network configuration)
deploy:
	stellar contract deploy --wasm target/wasm32-unknown-unknown/release/renewable_energy_certificate_tracker.wasm

# Install dependencies
install:
	cargo install --locked soroban-cli

# Format code
fmt:
	cargo fmt

# Check code
check:
	cargo check

# Run clippy
clippy:
	cargo clippy -- -D warnings

# Build optimized WASM
build-optimized:
	cargo build --target wasm32-unknown-unknown --release
	wasm-opt -Oz target/wasm32-unknown-unknown/release/renewable_energy_certificate_tracker.wasm -o renewable_energy_certificate_tracker_optimized.wasm

# Generate contract bindings
bindings:
	stellar contract bindings typescript --output-dir ./bindings

# Run integration tests
test-integration:
	cargo test --test integration

# All-in-one build and test
all: clean build test

# Help target
help:
	@echo "Available targets:"
	@echo "  build          - Build the contract using cargo"
	@echo "  stellar-build  - Build using Stellar CLI"
	@echo "  test           - Run unit tests"
	@echo "  clean          - Clean build artifacts"
	@echo "  deploy         - Deploy contract to network"
	@echo "  install        - Install Soroban CLI"
	@echo "  fmt            - Format code"
	@echo "  check          - Check code without building"
	@echo "  clippy         - Run clippy linter"
	@echo "  build-optimized - Build optimized WASM"
	@echo "  bindings       - Generate contract bindings"
	@echo "  test-integration - Run integration tests"
	@echo "  all            - Clean, build, and test"
	@echo "  help           - Show this help message"
