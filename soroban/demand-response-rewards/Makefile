# Makefile for Demand Response Rewards Soroban Contract

# Default target
all: build test

# Build the contract
build:
	cargo build --target wasm32-unknown-unknown --release

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Deploy to Stellar testnet (requires Stellar CLI configured)
deploy: build
	soroban contract deploy \
		--wasm target/wasm32-unknown-unknown/release/demand_response_rewards.wasm \
		--source . \
		--network testnet \
		--manifest-path Cargo.toml \
		--yes

# Simulate deployment (dry run)
simulate-deploy:
	soroban contract deploy \
		--wasm target/wasm32-unknown-unknown/release/demand_response_rewards.wasm \
		--source . \
		--network testnet \
		--manifest-path Cargo.toml \
		--dry-run

# Generate ABI
abi:
	soroban contract bindings \
		--wasm target/wasm32-unknown-unknown/release/demand_response_rewards.wasm \
		--output abi.json

.PHONY: all build test clean deploy simulate-deploy abi
