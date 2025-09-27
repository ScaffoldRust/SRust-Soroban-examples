WASM_NAME=pharmaceutical_supply_chain_tracker.wasm
CARGO_TARGET_DIR ?= target

.PHONY: all
all: build test

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release
	@ls -l "$(CARGO_TARGET_DIR)/wasm32-unknown-unknown/release/$(WASM_NAME)"

.PHONY: test
test:
	cargo test -- --nocapture

.PHONY: test-integration
test-integration:
	cargo test --test integration -- --nocapture

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: lint
lint:
	cargo clippy -- -D warnings

.PHONY: audit
audit:
	cargo audit

.PHONY: clean
clean:
	cargo clean
	rm -f "$(WASM_NAME)"

.PHONY: deploy-testnet
deploy-testnet:
	@if [ -z "$$SOROBAN_RPC_URL" ]; then \
		echo "Error: SOROBAN_RPC_URL environment variable not set"; \
		exit 1; \
	fi
	@if [ -z "$$SOROBAN_NETWORK_PASSPHRASE" ]; then \
		echo "Error: SOROBAN_NETWORK_PASSPHRASE environment variable not set"; \
		exit 1; \
	fi
	@if [ -z "$$DEFAULT_ADMIN_SECRET_KEY" ]; then \
		echo "Error: DEFAULT_ADMIN_SECRET_KEY environment variable not set"; \
		exit 1; \
	fi
	soroban contract deploy \
		--wasm "$(CARGO_TARGET_DIR)/wasm32-unknown-unknown/release/$(WASM_NAME)" \
		--source-account "$$DEFAULT_ADMIN_SECRET_KEY" \
		--rpc-url "$$SOROBAN_RPC_URL" \
		--network-passphrase "$$SOROBAN_NETWORK_PASSPHRASE"

.PHONY: optimize
optimize: build
	wasm-opt -Oz \
		"$(CARGO_TARGET_DIR)/wasm32-unknown-unknown/release/$(WASM_NAME)" \
		-o "$(CARGO_TARGET_DIR)/wasm32-unknown-unknown/release/$(WASM_NAME).optimized"
	mv "$(CARGO_TARGET_DIR)/wasm32-unknown-unknown/release/$(WASM_NAME).optimized" \
		"$(CARGO_TARGET_DIR)/wasm32-unknown-unknown/release/$(WASM_NAME)"