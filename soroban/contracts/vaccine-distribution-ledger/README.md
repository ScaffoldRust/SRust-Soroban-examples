# Vaccine Distribution Ledger Smart Contract

A comprehensive smart contract for tracking vaccine distribution on the Stellar network using Soroban, ensuring transparency and compliance with public health standards.

## Overview

This smart contract maintains a transparent and tamper-proof ledger for vaccine distribution, tracking vaccines from production to administration while ensuring compliance with WHO guidelines and other public health standards.

## Features

### Core Functionality
- **Batch Management**: Initialize and track vaccine batches from production
- **Distribution Tracking**: Log distribution events with cold chain monitoring
- **Administration Verification**: Record and verify vaccine administrations
- **Status Management**: Update batch statuses throughout the supply chain
- **Inventory Tracking**: Real-time inventory management and monitoring

### Compliance & Security
- **WHO Guidelines Compliance**: Built-in validation for health standards
- **Role-based Access Control**: Different permissions for manufacturers, distributors, and administrators
- **Tamper-proof Logging**: Immutable event history on the blockchain
- **Cold Chain Monitoring**: Alert system for temperature breaches
- **Audit Trail**: Complete traceability from production to patient

## Contract Structure

```
vaccine-distribution-ledger/src/
├── lib.rs                 # Main contract entry point and public interface
├── error.rs              # Contract error types and definitions
├── events.rs             # Event emissions for transparency
├── storage.rs            # Basic admin and configuration storage
├── distribution_storage.rs # Data structures and storage management
├── ledger.rs             # Core ledger logic and batch management
├── distribution.rs       # Distribution event management
├── utils.rs              # Validation and helper functions
├── Makefile              # Build and deployment automation
└── README.md             # This documentation file
```

## Key Data Structures

### VaccineBatch
Represents a vaccine batch with complete metadata:
- Batch ID, manufacturer, vaccine type
- Production and expiry dates
- Quantity tracking (initial and current)
- Status and compliance information

### DistributionEvent
Records all events in the vaccine lifecycle:
- Production, distribution, administration events
- Actor identification and timestamps
- Quantity movements and locations
- Temperature logs for cold chain compliance

### AdministrationRecord
Tracks individual vaccine administrations:
- Patient identification (anonymized)
- Administrator details and location
- Verification status and timestamps

## Public Functions

### Initialization
```rust
initialize(env: Env, admin: Address) -> Result<(), ContractError>
```
Initialize the contract with an admin address.

### Batch Management
```rust
initialize_batch(
    env: Env,
    batch_id: String,
    manufacturer: Address,
    vaccine_type: String,
    production_date: u64,
    quantity: u32,
    expiry_date: u64,
) -> Result<(), ContractError>
```
Create a new vaccine batch in the system.

```rust
update_batch_status(
    env: Env,
    batch_id: String,
    updater: Address,
    new_status: BatchStatus,
    notes: Option<String>,
) -> Result<(), ContractError>
```
Update the status of a vaccine batch.

### Distribution Tracking
```rust
log_distribution(
    env: Env,
    batch_id: String,
    distributor: Address,
    destination: String,
    quantity: u32,
    temperature_log: Option<String>,
) -> Result<(), ContractError>
```
Log a distribution event for a vaccine batch.

### Administration Verification
```rust
verify_administration(
    env: Env,
    batch_id: String,
    administrator: Address,
    patient_id: String,
    administered_quantity: u32,
    location: String,
) -> Result<(), ContractError>
```
Verify and record vaccine administration.

### Query Functions
```rust
get_history(env: Env, batch_id: String, offset: u32, limit: u32) 
    -> Result<Vec<DistributionEvent>, ContractError>

get_batch(env: Env, batch_id: String) 
    -> Result<VaccineBatch, ContractError>

inventory_check(env: Env, batch_id: String) 
    -> Result<u32, ContractError>
```

## Batch Status Flow

```
Produced → InTransit → Distributed → Administered
    ↓           ↓           ↓
 Expired  ColdChainBreach  Recalled
```

## Usage Examples

### Deploy and Initialize
```bash
# Build the contract
make stellar-build

# Deploy to testnet
make deploy-testnet

# Initialize with admin
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- initialize --admin $ADMIN_ADDRESS
```

### Create a Vaccine Batch
```bash
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- initialize_batch \
  --batch_id "PFIZER_001" \
  --manufacturer $MANUFACTURER_ADDRESS \
  --vaccine_type "COVID-19 mRNA" \
  --production_date 1640995200 \
  --quantity 1000 \
  --expiry_date 1672531200
```

### Log Distribution
```bash
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- log_distribution \
  --batch_id "PFIZER_001" \
  --distributor $DISTRIBUTOR_ADDRESS \
  --destination "Regional Health Center" \
  --quantity 100 \
  --temperature_log "2-8°C maintained"
```

### Verify Administration
```bash
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- verify_administration \
  --batch_id "PFIZER_001" \
  --administrator $ADMIN_ADDRESS \
  --patient_id "PATIENT_12345" \
  --administered_quantity 1 \
  --location "City Hospital"
```

## Security Considerations

### Access Control
- **Manufacturers**: Can create batches and update their status
- **Distributors**: Can log distribution events and update inventory
- **Administrators**: Can verify administrations and report issues
- **Admin**: Can perform emergency status updates and system maintenance

### Data Validation
- All inputs are validated for format and reasonableness
- Duplicate administrations to the same patient are prevented
- Expired or recalled batches cannot be distributed
- Cold chain breaches are automatically flagged

### Compliance Features
- WHO guideline compliance checks
- Automatic expiry date monitoring
- Cold chain breach reporting
- Complete audit trail maintenance

## Development

### Prerequisites
- Rust with `wasm32-unknown-unknown` target
- Stellar CLI
- Soroban SDK

### Building
```bash
# Install dependencies
make setup

# Build the contract
make build

# Run tests
make test

# Deploy to testnet
make deploy-testnet
```

### Testing
```bash
# Run all checks
make check

# Generate test data
make generate-keypairs
make fund-accounts
```

## Integration

### Supply Chain Integration
The contract provides hooks for integration with:
- Pharmaceutical manufacturing systems
- Cold chain monitoring IoT devices
- Healthcare management systems
- Government health departments

### Event Monitoring
All contract interactions emit events that can be monitored for:
- Real-time inventory tracking
- Compliance monitoring
- Supply chain analytics
- Public health reporting

## Compliance

This smart contract is designed to support compliance with:
- WHO vaccine distribution guidelines
- National health authority requirements
- Cold chain management standards
- Data privacy regulations (through patient ID anonymization)
