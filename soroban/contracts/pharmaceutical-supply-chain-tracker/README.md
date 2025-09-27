# Pharmaceutical Supply Chain Tracker

A Soroban smart contract for tracking pharmaceutical supply chain events on the Stellar blockchain, ensuring transparent and tamper-proof logging of drugs from manufacturer to patient while complying with supply chain traceability standards.

## Features

- Role-based access control for supply chain participants
- Complete supply chain event tracking and verification
- GS1 standard compliance for pharmaceutical tracking
- Cold chain monitoring support
- Batch tracking with detailed event history
- Integration with industry standards (DSCSA compliance)
- Comprehensive audit trail

## Contract Structure

```
src/
├── lib.rs          # Core contract configuration and data structures
├── tracker.rs      # Supply chain event tracking implementation
├── stages.rs       # Supply chain stage management
└── utils.rs        # Utility functions and validation
```

## Getting Started

### Prerequisites

1. Install Rust and add WASM target:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

2. Install Soroban CLI:
```bash
cargo install --locked soroban-cli
# Or via Homebrew
brew install soroban-cli
```

### Building

```bash
# Build the contract
make build

# Run tests
make test

# Format code
make fmt

# Run linter
make clippy

# Run security audit
make audit
```

## Contract Deployment

1. Set up environment variables:
```bash
export SOROBAN_RPC_URL="https://your-testnet-url"
export SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export DEFAULT_ADMIN_SECRET_KEY="your-admin-secret-key"
```

2. Deploy to testnet:
```bash
make deploy-testnet
```

## Usage

### Contract Initialization

```rust
// Initialize contract with admin
PharmaceuticalSupplyChain::initialize(env, admin);

// Assign roles to entities
PharmaceuticalSupplyChain::assign_role(env, admin, manufacturer, Role::Manufacturer);
PharmaceuticalSupplyChain::assign_role(env, admin, distributor, Role::Distributor);
PharmaceuticalSupplyChain::assign_role(env, admin, pharmacy, Role::Pharmacy);
```

### Supply Chain Events

```rust
// Create new batch
PharmaceuticalSupplyChain::create_batch(
    env,
    manufacturer,
    batch_id,
    location,
    metadata,
);

// Ship batch
PharmaceuticalSupplyChain::ship_batch(
    env,
    distributor,
    batch_id,
    location,
    metadata,
);

// Receive batch
PharmaceuticalSupplyChain::receive_batch(
    env,
    receiver,
    batch_id,
    location,
    metadata,
);
```

### Batch Verification

```rust
// Get batch history
let history = PharmaceuticalSupplyChain::get_batch_history(env, batch_id);

// Verify batch integrity
let is_valid = PharmaceuticalSupplyChain::verify_batch(env, batch_id);

// Get current status
let status = PharmaceuticalSupplyChain::get_batch_status(env, batch_id);
```

## Data Structures

### Supply Chain Roles

```rust
pub enum Role {
    Manufacturer,
    Distributor,
    Wholesaler,
    Pharmacy,
    Hospital,
}
```

### Batch Status

```rust
pub enum Status {
    Created,
    InTransit,
    Received,
    Quarantined,
    Approved,
    Rejected,
    Dispensed,
}
```

### Event Structure

```rust
pub struct Event {
    timestamp: u64,
    entity: Address,
    event_type: Symbol,
    location: String,
    status: Status,
    metadata: Vec<String>,
}
```

## Testing

The contract includes comprehensive test coverage:

- Unit tests for each component
- Integration tests for complete supply chain flow
- Role-based access control tests
- Event tracking validation
- Batch verification tests

Run tests with:
```bash
make test           # Run all tests
make test-integration # Run integration tests
```

## Security Considerations

- Role-based access control for all operations
- Event immutability and auditability
- Cryptographic verification of entities
- Soroban security best practices
- Regular security audits

## Standards Compliance

- GS1 GTIN implementation for product identification
- DSCSA compliance features
- Industry-standard event tracking
- Cold chain monitoring support

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.