# Peer-to-Peer Energy Sharing Smart Contract

A Stellar Soroban smart contract that enables direct energy sharing between prosumers (both producers and consumers) on the Stellar network, facilitating secure peer-to-peer energy transactions with real-time settlement.

## Overview

This contract enables prosumers to:
- Create energy sharing agreements directly with other prosumers
- Record energy delivery with smart meter verification
- Execute secure payments using Stellar tokens
- Track complete transaction history
- Handle dispute resolution mechanisms

## Features

### Core Functionality
- **Prosumer Registration**: Single registration for participants who can both produce and consume energy
- **Energy Agreements**: Create bilateral agreements specifying quantity, price, and delivery terms
- **Meter Verification**: Record energy delivery with meter reading validation
- **Secure Settlement**: Automatic payment processing using Stellar tokens
- **Transaction History**: Complete audit trail of all energy sharing activities

### Energy Sharing Features
- **Direct P2P Trading**: No intermediary marketplace required
- **Flexible Agreements**: Custom terms for each energy sharing arrangement
- **Real-time Settlement**: Immediate payment upon energy delivery verification
- **Dispute Handling**: Built-in dispute mechanisms for delivery discrepancies
- **Deadline Management**: Time-bound agreements with expiration handling

## Contract Structure

```
peer-to-peer-energy-sharing/
├── src/
│   ├── lib.rs           # Main contract and exports
│   ├── sharing.rs       # Energy sharing agreement logic
│   ├── payment.rs       # Payment processing and settlement
│   ├── utils.rs         # Data structures and utilities
│   └── test.rs          # Contract tests
├── Cargo.toml           # Dependencies
├── Makefile             # Build and deployment automation
└── README.md            # Documentation
```

## Data Structures

### EnergyAgreement
```rust
pub struct EnergyAgreement {
    pub agreement_id: u64,
    pub provider: Address,
    pub consumer: Address,
    pub energy_amount_kwh: u64,
    pub price_per_kwh: u64,
    pub total_amount: u64,
    pub delivery_deadline: u64,
    pub status: AgreementStatus,
    pub created_at: u64,
}
```

### EnergyTransaction
```rust
pub struct EnergyTransaction {
    pub transaction_id: u64,
    pub agreement_id: u64,
    pub provider: Address,
    pub consumer: Address,
    pub energy_delivered_kwh: u64,
    pub meter_reading: u64,
    pub payment_amount: u64,
    pub delivered_at: u64,
    pub settled_at: Option<u64>,
    pub status: TransactionStatus,
}
```

### Status Types
- **AgreementStatus**: Active, Delivered, Settled, Cancelled, Expired
- **TransactionStatus**: Pending, Delivered, Settled, Disputed

## Key Functions

### Initialization
```rust
pub fn initialize(
    env: Env,
    admin: Address,
    token_contract: Address,
) -> Result<(), SharingError>
```
Sets up the contract with admin and payment token configuration.

### Prosumer Management
```rust
pub fn register_prosumer(env: Env, prosumer: Address) -> Result<(), SharingError>
```
Registers a prosumer who can both produce and consume energy.

### Energy Sharing
```rust
pub fn create_agreement(
    env: Env,
    provider: Address,
    consumer: Address,
    energy_amount_kwh: u64,
    price_per_kwh: u64,
    delivery_deadline: u64,
) -> Result<u64, SharingError>
```
Creates a bilateral energy sharing agreement between prosumers.

```rust
pub fn deliver_energy(
    env: Env,
    agreement_id: u64,
    energy_delivered_kwh: u64,
    meter_reading: u64,
    provider: Address,
) -> Result<u64, SharingError>
```
Records energy delivery with smart meter verification data.

### Payment Processing
```rust
pub fn settle_payment(
    env: Env,
    transaction_id: u64,
    settler: Address,
) -> Result<(), SharingError>
```
Executes payment transfer from consumer to provider using Stellar tokens.

### Query Functions
```rust
pub fn get_agreement(env: Env, agreement_id: u64) -> Result<EnergyAgreement, SharingError>
```
Retrieves agreement details.

```rust
pub fn get_transaction_history(
    env: Env,
    prosumer: Address,
) -> Result<Vec<EnergyTransaction>, SharingError>
```
Gets complete transaction history for a prosumer.

## Installation and Setup

### Prerequisites
- Rust 1.70+
- Stellar CLI
- WebAssembly target support

### Build the Contract
```bash
# Standard build
make build

# Build with Soroban CLI
make soroban-build

# Optimized build for deployment
make optimize
```

### Run Tests
```bash
make test
```

### Code Quality Checks
```bash
# Format code
make format

# Run linter
make clippy

# Run all checks
make check-all
```

## Deployment

### Deploy to Testnet
```bash
make deploy-testnet
```

### Initialize Contract
```bash
make init-testnet \
    ADMIN_ADDRESS=<admin_address> \
    TOKEN_CONTRACT_ADDRESS=<token_address> \
    CONTRACT_ID=<contract_id>
```

### Register Prosumers
```bash
# Register prosumer
make register-prosumer-testnet \
    PROSUMER_ADDRESS=<prosumer_address> \
    CONTRACT_ID=<contract_id>
```

## Usage Examples

### Complete Energy Sharing Flow
```bash
# 1. Prosumer A creates agreement to share energy with Prosumer B
make create-agreement-testnet \
    PROVIDER_ADDRESS=<prosumer_a_address> \
    CONSUMER_ADDRESS=<prosumer_b_address> \
    ENERGY_AMOUNT_KWH=100 \
    PRICE_PER_KWH=50 \
    DELIVERY_DEADLINE=<timestamp> \
    CONTRACT_ID=<contract_id>

# 2. Prosumer A delivers energy with meter verification
make deliver-energy-testnet \
    AGREEMENT_ID=1 \
    ENERGY_DELIVERED_KWH=100 \
    METER_READING=<meter_value> \
    PROVIDER_ADDRESS=<prosumer_a_address> \
    CONTRACT_ID=<contract_id>

# 3. Settlement payment (can be triggered by either party)
make settle-payment-testnet \
    TRANSACTION_ID=1 \
    SETTLER_ADDRESS=<prosumer_b_address> \
    CONTRACT_ID=<contract_id>

# 4. View transaction history
make get-history-testnet \
    PROSUMER_ADDRESS=<prosumer_address> \
    CONTRACT_ID=<contract_id>
```

### Agreement Management
```bash
# Get agreement details
make get-agreement-testnet \
    AGREEMENT_ID=1 \
    CONTRACT_ID=<contract_id>
```

## Integration Guide

### Smart Meter Integration
1. Register prosumers using `register_prosumer()`
2. Create agreements based on energy availability and demand
3. Use meter data in `deliver_energy()` for verification
4. Implement automated settlement based on delivery confirmation

### Payment Integration
The contract uses Stellar tokens for settlement:
- Configure token contract address during initialization
- Ensure prosumers have sufficient token balances
- Settlement transfers tokens from consumer to provider

### Example Integration Flow
```rust
// 1. Register prosumers
contract.register_prosumer(prosumer_a)?;
contract.register_prosumer(prosumer_b)?;

// 2. Create energy sharing agreement
let agreement_id = contract.create_agreement(
    prosumer_a,     // provider
    prosumer_b,     // consumer
    100,            // kWh
    50,             // price per kWh
    deadline,       // delivery deadline
)?;

// 3. Deliver energy with meter verification
let transaction_id = contract.deliver_energy(
    agreement_id,
    100,            // kWh delivered
    meter_reading,  // smart meter value
    prosumer_a,     // provider
)?;

// 4. Settle payment
contract.settle_payment(transaction_id, prosumer_b)?;

// 5. Check transaction history
let history = contract.get_transaction_history(prosumer_a)?;
```

## Security Features

### Access Control
- Address-based authentication for all operations
- Only registered prosumers can participate
- Provider/consumer authorization for respective actions

### Transaction Integrity
- Immutable agreement and transaction records
- Smart meter verification for energy delivery
- Atomic payment operations with Stellar tokens
- Complete audit trail through events

### Dispute Resolution
- Built-in dispute mechanism for delivery discrepancies
- Transaction status tracking for dispute management
- Provider and consumer can dispute transactions

## Error Handling

The contract includes comprehensive error handling:
- `NotInitialized`: Contract not yet initialized
- `ProsumerNotRegistered`: Prosumer not registered
- `AgreementNotFound`: Agreement doesn't exist
- `DeliveryDeadlinePassed`: Delivery deadline exceeded
- `TransactionAlreadySettled`: Payment already processed
- `SelfSharingNotAllowed`: Prosumer cannot share with themselves
- `PaymentFailed`: Token transfer failed

## Performance Features

### Storage Optimization
- Efficient data structures for minimal storage cost
- Optimized maps for fast agreement and transaction lookups
- Minimal storage updates per operation

### Scalability
- Direct P2P model eliminates centralized bottlenecks
- Efficient transaction processing
- Optimized for frequent energy sharing activities

## Compliance and Standards

### Energy Trading Standards
- Compatible with peer-to-peer energy sharing regulations
- Transparent pricing and settlement
- Verifiable energy delivery records

### Blockchain Standards
- Standard Soroban contract patterns
- Stellar token integration
- Event emission for transparency

## Development

### Testing Strategy
- Unit tests for individual functions
- Integration tests for complete sharing workflows
- Edge case testing for agreement and payment scenarios
- Meter verification testing

## Support

For issues, questions, or contributions, please refer to the project repository and follow the contribution guidelines.