default: build

all: test

test: build
	cargo test

build:
	stellar contract build
	@ls -l target/wasm32-unknown-unknown/release/*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean

deploy: build
	@echo "Deploying patient-consent-management-system contract..."
	@echo "Contract WASM: target/wasm32-unknown-unknown/release/patient_consent_management_system.wasm"

lint:
	cargo clippy --all-targets --all-features

check: fmt lint test

.PHONY: all test build fmt clean deploy lint check
