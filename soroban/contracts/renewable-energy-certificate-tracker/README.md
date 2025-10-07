# Renewable Energy Certificate (REC) Tracker

A comprehensive smart contract for tracking Renewable Energy Certificates (RECs) on the Stellar network using Soroban. This contract ensures transparent issuance, transfer, and retirement of RECs for renewable energy compliance, following standards like I-REC and RE100.

## Overview

The Renewable Energy Certificate Tracker provides a tamper-proof, decentralized system for managing RECs throughout their lifecycle. It enables authorized issuers to create certificates for verified renewable energy production, facilitates secure transfers between parties, and allows certificate retirement for renewable energy claims.

## Features

- **Certificate Issuance**: Authorized issuers can create RECs for verified renewable energy production
- **Ownership Transfer**: Secure transfer of RECs between parties with full audit trail
- **Certificate Retirement**: Retirement of RECs to claim renewable energy usage
- **Verification & Authentication**: Built-in verification mechanisms for REC authenticity
- **Role-Based Access Control**: Issuer authorization and owner-based permissions
- **Complete Audit Trail**: Full lifecycle tracking with timestamped events
- **Standards Compliance**: Support for I-REC, RE100, and other verification standards
- **Multiple Energy Sources**: Support for Solar, Wind, Hydro, Geothermal, Biomass, and Tidal

## Contract Structure

```
renewable-energy-certificate-tracker/
├── src/
│   ├── lib.rs          # Main contract, data structures, and exports
│   ├── certificate.rs  # REC issuance and verification logic
│   ├── transfer.rs     # Transfer and retirement operations
│   └── utils.rs        # Validation and utility functions
├── Cargo.toml          # Dependencies and package configuration
├── Makefile            # Build and deployment automation
└── README.md           # This file
```

## Data Structures

### REC (Renewable Energy Certificate)
```rust
pub struct REC {
    pub id: BytesN<32>,                    // Unique certificate ID
    pub issuer: Address,                   // Issuer address
    pub energy_source: EnergySource,       // Type of renewable energy
    pub production_date: u64,              // Energy production timestamp
    pub production_location: String,       // Geographic location
    pub capacity_mwh: i128,                // Capacity in megawatt-hours
    pub current_owner: Address,            // Current owner
    pub status: RECStatus,                 // Current status
    pub verification_standard: String,     // I-REC, RE100, etc.
    pub verification_hash: BytesN<32>,     // Verification document hash
    pub issuance_date: u64,                // Certificate issuance timestamp
    pub metadata: Map<String, String>,     // Additional metadata
}
```

### Energy Source Types
- Solar
- Wind
- Hydro
- Geothermal
- Biomass
- Tidal

### REC Status
- **Issued**: Newly created certificate
- **Transferred**: Certificate has been transferred
- **Retired**: Certificate retired for compliance claims
- **Suspended**: Certificate suspended by admin

## Key Functions

### Administrative Functions

#### `initialize(env: Env, admin: Address)`
Initializes the contract with an admin address. Must be called once before any other operations.

**Parameters:**
- `admin`: Address of the contract administrator

#### `register_issuer(env: Env, issuer: Address, name: String)`
Registers an authorized issuer who can create RECs.

**Parameters:**
- `issuer`: Address to authorize as an issuer
- `name`: Name of the issuing organization

**Authorization:** Admin only

### Certificate Operations

#### `issue_rec(...) -> BytesN<32>`
Issues a new REC for verified renewable energy production.

**Parameters:**
- `issuer`: Address of the authorized issuer
- `energy_source`: Type of renewable energy (Solar, Wind, etc.)
- `production_date`: Timestamp of energy production
- `production_location`: Geographic location of production
- `capacity_mwh`: Energy capacity in megawatt-hours
- `verification_standard`: Verification standard (I-REC, RE100, etc.)
- `verification_hash`: Hash of verification documents
- `metadata`: Additional certificate metadata

**Returns:** Unique REC ID

**Authorization:** Authorized issuer only

#### `transfer_rec(env: Env, rec_id: BytesN<32>, from: Address, to: Address, capacity_mwh: i128) -> bool`
Transfers a REC to a new owner.

**Parameters:**
- `rec_id`: ID of the REC to transfer
- `from`: Current owner address
- `to`: New owner address
- `capacity_mwh`: Capacity to transfer

**Returns:** Success status

**Authorization:** Current owner only

#### `retire_rec(env: Env, rec_id: BytesN<32>, owner: Address, capacity_mwh: i128, retirement_reason: String) -> bool`
Retires a REC to claim renewable energy usage.

**Parameters:**
- `rec_id`: ID of the REC to retire
- `owner`: Owner address
- `capacity_mwh`: Capacity to retire
- `retirement_reason`: Reason for retirement

**Returns:** Success status

**Authorization:** Current owner only

### Query Functions

#### `get_rec_status(env: Env, rec_id: BytesN<32>) -> REC`
Retrieves the current status and details of a REC.

#### `get_rec_history(env: Env, rec_id: BytesN<32>) -> Vec<RECEvent>`
Retrieves the complete event history for a REC.

#### `get_issuer_info(env: Env, issuer: Address) -> IssuerInfo`
Retrieves information about an authorized issuer.

#### `get_contract_stats(env: Env) -> (i128, i128, i128, i128)`
Retrieves contract-level statistics:
- Total RECs issued
- Total capacity issued (MWh)
- Total capacity transferred (MWh)
- Total capacity retired (MWh)

#### `suspend_rec(env: Env, rec_id: BytesN<32>) -> bool`
Suspends a REC (admin only).

## Building and Testing

### Prerequisites
- Rust (latest stable)
- Soroban CLI
- wasm32-unknown-unknown target

### Build
```bash
# Standard build
cargo build --target wasm32-unknown-unknown --release

# Or using Stellar CLI
stellar contract build

# Or using Makefile
make build
```

### Test
```bash
# Run all tests
cargo test

# Or using Makefile
make test
```

### Deploy
```bash
# Deploy to network (requires network configuration)
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/renewable_energy_certificate_tracker.wasm

# Or using Makefile
make deploy
```

## Usage Examples

### 1. Initialize Contract
```rust
let admin = Address::generate(&env);
contract.initialize(&admin);
```

### 2. Register Issuer
```rust
let issuer = Address::generate(&env);
contract.register_issuer(&issuer, String::from_str(&env, "Green Energy Co"));
```

### 3. Issue REC
```rust
let rec_id = contract.issue_rec(
    &issuer,
    EnergySource::Solar,
    production_date,
    String::from_str(&env, "California, USA"),
    1000, // 1000 MWh
    String::from_str(&env, "I-REC"),
    verification_hash,
    metadata,
);
```

### 4. Transfer REC
```rust
let buyer = Address::generate(&env);
contract.transfer_rec(&rec_id, &issuer, &buyer, 1000);
```

### 5. Retire REC
```rust
contract.retire_rec(
    &rec_id,
    &buyer,
    1000,
    String::from_str(&env, "2024 renewable energy compliance"),
);
```

## Security Features

- **Authentication**: All sensitive operations require proper authorization
- **Ownership Verification**: Transfers and retirements require owner authentication
- **Issuer Authorization**: Only registered issuers can create RECs
- **Status Validation**: Prevents operations on retired or suspended RECs
- **Immutable History**: Complete audit trail cannot be modified
- **Hash Verification**: Verification documents secured with cryptographic hashes

## Integration

### Carbon Credit Integration
This contract can be integrated with the carbon-credit-registry contract for organizations using renewable energy to offset carbon emissions.

### Energy Production Oracles
Designed to support integration with energy production data oracles for real-time verification of renewable energy generation.

## Storage Optimization

The contract uses efficient storage patterns to minimize transaction fees:
- Persistent storage for RECs and issuer information
- Instance storage for contract-level data
- Event history stored separately from REC data
- Optimized data structures for minimal storage footprint

## Compliance Standards

The contract supports multiple renewable energy compliance standards:
- **I-REC**: International Renewable Energy Certificate Standard
- **RE100**: 100% Renewable Energy Initiative
- Custom verification standards as needed

## Development

### Run Tests
```bash
make test
```

### Format Code
```bash
make fmt
```

### Check Code
```bash
make check
```

### Run Linter
```bash
make clippy
```

### All-in-One
```bash
make all  # Clean, build, and test
```

## License

This contract is provided as-is for renewable energy certificate tracking on the Stellar network.

## Support

For issues, questions, or contributions, please refer to the project repository.
