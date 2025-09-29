# Energy Consumption Verifier Smart Contract

A Stellar Soroban smart contract for verifying energy consumption data on the blockchain, ensuring accurate and tamper-proof reporting for billing and efficiency programs.

## Overview

The Energy Consumption Verifier contract enables:
- Secure submission of energy consumption data from smart meters
- Role-based verification by authorized verifiers
- Tamper-proof data integrity through cryptographic hashing
- Comprehensive audit trails for transparency
- Integration hooks for smart meter and billing contracts

## Features

### Core Functionality
- **Data Submission**: Consumers and meters can submit energy consumption data
- **Verification System**: Authorized verifiers can validate and approve/reject data
- **Audit Trail**: Complete logging of all verification events
- **Data Integrity**: Cryptographic verification of data authenticity
- **Role Management**: Admin-controlled verifier registration

### Energy-Specific Features
- **Meter Data Validation**: Compliance with smart meter standards
- **Environmental Monitoring**: Temperature and voltage tracking
- **Consumption Anomaly Detection**: Automated flagging of suspicious data
- **Historical Data Analysis**: Trend analysis and consistency checks
- **Billing Integration**: Support for consumption-based billing systems

## Contract Structure

```
energy-consumption-verifier/
├── src/
│   ├── lib.rs           # Main contract and exports
│   ├── verifier.rs      # Core verification logic
│   ├── reporting.rs     # Report management
│   ├── utils.rs         # Shared utilities
│   └── test.rs          # Contract tests
├── Cargo.toml           # Dependencies
├── Makefile             # Build automation
└── README.md            # Documentation
```

## Data Structures

### ConsumptionRecord
```rust
pub struct ConsumptionRecord {
    pub consumer: Address,
    pub meter_id: String,
    pub consumption_kwh: u64,
    pub timestamp: u64,
    pub meter_reading: u64,
    pub temperature: Option<i32>,
    pub voltage: Option<u32>,
    pub data_hash: Bytes,
}
```

### VerificationRecord
```rust
pub struct VerificationRecord {
    pub record_id: u64,
    pub verifier: Address,
    pub status: VerificationStatus,
    pub timestamp: u64,
    pub comments: Option<String>,
}
```

### VerificationStatus
- `Pending`: Awaiting verification
- `Verified`: Data confirmed as authentic
- `Rejected`: Data rejected due to issues
- `Flagged`: Data marked for further review

## Key Functions

### Initialization
```rust
pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError>
```
Sets up the contract with initial admin and verifier roles.

### Data Management
```rust
pub fn submit_data(
    env: Env,
    consumer: Address,
    meter_id: String,
    consumption_kwh: u64,
    meter_reading: u64,
    temperature: Option<i32>,
    voltage: Option<u32>,
) -> Result<u64, ContractError>
```
Submits energy consumption data for verification.

```rust
pub fn verify_data(
    env: Env,
    verifier: Address,
    record_id: u64,
    status: VerificationStatus,
    comments: Option<String>,
) -> Result<(), ContractError>
```
Validates and marks data as authentic (verifiers only).

### Query Functions
```rust
pub fn get_verification(env: Env, record_id: u64) -> Result<VerificationRecord, ContractError>
```
Queries the verification status of consumption data.

```rust
pub fn audit_log(env: Env, offset: u32, limit: u32) -> Result<Vec<AuditLogEntry>, ContractError>
```
Retrieves audit log for verification events.

### Administration
```rust
pub fn register_verifier(env: Env, admin: Address, verifier: Address) -> Result<(), ContractError>
```
Registers a new verifier (admin only).

```rust
pub fn register_meter(env: Env, admin: Address, meter_id: String, meter_address: Address) -> Result<(), ContractError>
```
Registers a smart meter (admin only).

## Installation and Setup

### Prerequisites
- Rust 1.70+
- Stellar CLI
- WebAssembly target support

### Setup Development Environment
```bash
make setup
```

This will:
- Install WASM target
- Install Stellar CLI
- Configure network settings

### Generate Test Keypairs
```bash
make generate-keypairs
```

### Fund Test Accounts
```bash
make fund-accounts
```

## Building and Testing

### Build the Contract
```bash
# Standard build
make build

# Build with Stellar CLI
make stellar-build

# Optimized build
make optimize
```

### Run Tests
```bash
make test
```

### Code Quality Checks
```bash
# Format code
make fmt

# Run linter
make clippy

# Run all checks
make check
```

## Deployment

### Deploy to Testnet
```bash
make deploy-testnet
```

### Initialize Contract
```bash
make init-contract ADMIN_ADDRESS=<admin_address> CONTRACT_ID=<contract_id>
```

### Register Verifiers and Meters
```bash
make register-verifier ADMIN_ADDRESS=<admin> VERIFIER_ADDRESS=<verifier> CONTRACT_ID=<contract_id>
make register-meter ADMIN_ADDRESS=<admin> METER_ADDRESS=<meter> CONTRACT_ID=<contract_id>
```

## Usage Examples

### Submit Consumption Data
```bash
make submit-test-data CONSUMER_ADDRESS=<consumer> CONTRACT_ID=<contract_id>
```

### Verify Data
```bash
make verify-test-data VERIFIER_ADDRESS=<verifier> RECORD_ID=<record_id> CONTRACT_ID=<contract_id>
```

### Query Verification Status
```bash
make get-verification RECORD_ID=<record_id> CONTRACT_ID=<contract_id>
```

### View Audit Log
```bash
make get-audit-log CONTRACT_ID=<contract_id>
```

## Integration Guide

### Smart Meter Integration
1. Register meters using `register_meter()`
2. Configure meters to submit data via `submit_data()`
3. Implement automated data validation

### Billing System Integration
1. Query verified consumption data
2. Calculate costs based on consumption
3. Generate billing records

### Example Integration Flow
```rust
// 1. Submit meter data
let record_id = contract.submit_data(
    consumer_address,
    "METER_001",
    consumption_kwh,
    meter_reading,
    temperature,
    voltage
)?;

// 2. Verify data
contract.verify_data(
    verifier_address,
    record_id,
    VerificationStatus::Verified,
    Some("Data validated successfully")
)?;

// 3. Query for billing
let verification = contract.get_verification(record_id)?;
if verification.status == VerificationStatus::Verified {
    // Process for billing
}
```

## Security Features

### Data Integrity
- Cryptographic hashing of consumption data
- Tamper detection through hash verification
- Immutable audit trail

### Access Control
- Role-based permissions (admin, verifier, consumer)
- Address-based authentication
- Function-level authorization

### Validation Rules
- Consumption limits and range checks
- Meter reading consistency validation
- Environmental parameter validation
- Timestamp validation

## Error Handling

The contract includes comprehensive error handling:

- `AlreadyInitialized`: Contract already initialized
- `NotAuthorized`: Insufficient permissions
- `InvalidInput`: Invalid parameter values
- `RecordNotFound`: Requested record doesn't exist
- `InvalidMeterData`: Meter data validation failed
- `VerifierNotRegistered`: Verifier not authorized
- `DataIntegrityError`: Data hash mismatch
- `InvalidTimestamp`: Invalid time data

## Performance Optimizations

### Storage Efficiency
- Optimized data structures for minimal storage cost
- Efficient indexing for fast queries
- Automatic cleanup of old audit logs

### Transaction Costs
- Batch operations where possible
- Minimal storage updates
- Gas-efficient algorithms

## Compliance and Standards

### IEC Smart Meter Standards
- Compliant with IEC 62056 series
- Support for standard meter data formats
- Validation according to metering standards

### Billing Accuracy
- Tamper-proof verification process
- Multiple validation layers
- Audit trail for regulatory compliance

## Development

### Project Structure
The contract follows standard Soroban patterns:
- Modular design with separated concerns
- Comprehensive test coverage
- Standard error handling
- Efficient storage patterns

### Contributing
1. Fork the repository
2. Create feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit pull request

### Testing Strategy
- Unit tests for individual functions
- Integration tests for complete workflows
- Edge case testing
- Performance testing

## Documentation

Generate contract documentation:
```bash
make docs
```

## Support

For issues, questions, or contributions, please refer to the project repository and follow the contribution guidelines.

## License

This project is part of the SRust-Soroban-examples repository and follows the same licensing terms.