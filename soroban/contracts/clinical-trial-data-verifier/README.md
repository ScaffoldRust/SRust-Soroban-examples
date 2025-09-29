# Clinical Trial Data Verifier - Soroban Smart Contract

A comprehensive smart contract for verifying clinical trial data on the Stellar network using Soroban, ensuring data integrity and compliance with ICH-GCP (International Council for Harmonisation - Good Clinical Practice) standards.

## Overview

This contract enables transparent, immutable verification of clinical trial data while maintaining compliance with research standards. It supports role-based access control, comprehensive audit trails, and integration with patient consent and supply chain contracts.

## Features

### ✅ ICH-GCP Compliance
- Requires ethics approval ID and protocol version
- Validates source data verification
- Enforces proper metadata for all submissions
- Maintains complete audit trails for regulatory inspections

### 🔐 Role-Based Access Control
- **Admin**: Configure trial, manage verifiers, link external contracts
- **Verifiers**: Authorized researchers/medical professionals who validate data
- **Submitters**: Researchers who submit trial data

### 📊 Data Types Supported
- Patient outcomes
- Adverse events
- Laboratory measurements
- Vital signs
- Medication dosage records
- Patient compliance data

### 🔗 Integration Capabilities
- Link to patient consent contracts (HIPAA compliant)
- Link to supply chain contracts (drug batch tracking)
- Event emission for external monitoring systems

## Contract Structure

```
src/
├── lib.rs          # Main contract logic and public API
├── verifier.rs     # Core verification logic
├── data.rs         # Data structures and validation
└── utils.rs        # Utility functions (hashing, validation)
```

## Installation & Build

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Stellar CLI
cargo install stellar-cli

# Add WebAssembly target
rustup target add wasm32-unknown-unknown
```

### Build Commands
```bash
# Build the contract
make build
# or
stellar contract build

# Run tests
make test

# Clean build artifacts
make clean
```

## Usage

### 1. Initialize Contract

```rust
let trial_config = TrialConfiguration {
    trial_id: String::from_str(&env, "TRIAL-2025-001"),
    start_date: 1704067200,  // Unix timestamp
    end_date: 1735689600,
    required_verifications: 2,
    protocol_hash: String::from_str(&env, "abc123..."),
    protocol_version: String::from_str(&env, "ICH-GCP-E6-R2"),
    ethics_approval_id: String::from_str(&env, "IRB-2025-001"),
    study_phase: StudyPhase::Phase3,
};

contract.initialize(&admin_address, &trial_config);
```

### 2. Add Verifiers

```rust
contract.add_verifier(&admin_address, &verifier_address);
```

### 3. Submit Clinical Data

```rust
let metadata = TrialMetadata {
    data_type: DataType::PatientOutcome,
    patient_id_hash: String::from_str(&env, "hash_of_patient_id"),
    visit_number: 3,
    measurement_date: 1704153600,
    protocol_deviation: false,
    source_verified: true,
};

contract.submit_data(
    &submitter_address,
    &trial_id,
    &data_hash,
    &metadata_hash,
    &metadata
);
```

### 4. Verify Data

```rust
contract.verify_data(
    &verifier_address,
    &data_hash,
    true,  // approved
    &String::from_str(&env, "Data verified against protocol v2.1")
);
```

### 5. Link to External Contracts

```rust
// Link patient consent
contract.link_consent_contract(
    &admin_address,
    &data_hash,
    &consent_contract_address,
    &patient_id_hash
);

// Link supply chain
contract.link_supply_chain(
    &admin_address,
    &data_hash,
    &supply_contract_address,
    &String::from_str(&env, "BATCH-2025-001")
);
```

### 6. Query Data

```rust
// Get verification status
let status = contract.get_verification_status(&data_hash);

// Get complete audit trail
let audit_trail = contract.get_audit_trail(&data_hash);

// Get consent links
let consent_links = contract.get_consent_links(&data_hash);

// Get supply chain links
let supply_links = contract.get_supply_chain_links(&data_hash);

// Get trial configuration
let config = contract.get_trial_config();
```

## Data Structures

### TrialConfiguration
Contains trial setup including ICH-GCP required fields (ethics approval, protocol version, study phase).

### ClinicalData
Stores submitted trial data with metadata, verification status, and timestamps.

### TrialMetadata
ICH-GCP compliant metadata including:
- Data type (outcome, adverse event, lab result, etc.)
- Hashed patient identifier (HIPAA compliant)
- Visit/timepoint number
- Measurement date
- Protocol deviation flag
- Source verification status

### VerificationEvent
Audit trail entry recording each verification action with GCP compliance flag.

### ConsentLink
Links trial data to patient consent contract for compliance tracking.

### SupplyChainLink
Links trial data to supply chain contract for drug/device traceability.

## Error Codes

| Code | Description |
|------|-------------|
| 1 | Contract already initialized / Data already exists |
| 2 | Unauthorized verifier |
| 3 | Caller is not admin |
| 4 | Data not found |
| 5 | Trial configuration not found |
| 10 | Missing ethics approval ID |
| 11 | Missing protocol version |
| 12 | Invalid data hash format |
| 13 | GCP compliance validation failed |
| 14 | Cannot verify rejected data |

## ICH-GCP Compliance Features

### Required Fields Validation
- Ethics approval ID must be provided
- Protocol version must be specified
- Patient identifiers must be hashed
- Source data verification required

### Audit Trail
- Complete history of all verification events
- Timestamps for all actions
- Verifier identity recorded
- GCP compliance flag per verification

### Data Integrity
- Cryptographic hashing (SHA3-256)
- Immutable storage on blockchain
- Multi-verifier requirement configurable

## Security Considerations

1. **Authentication**: All state-changing functions require `require_auth()`
2. **Authorization**: Role-based access enforced (admin, verifier)
3. **Data Privacy**: Patient IDs are hashed before storage
4. **Immutability**: Data cannot be modified after submission, only verified/rejected
5. **Audit Trail**: All actions permanently recorded for regulatory compliance

## Integration Examples

### Patient Consent Integration
Ensures that all clinical data is linked to valid patient consent:
```rust
// Submit data
contract.submit_data(...);

// Link to consent contract
contract.link_consent_contract(
    admin, 
    data_hash, 
    consent_contract_id,
    patient_id_hash
);

// Verify consent exists before data verification
let consents = contract.get_consent_links(data_hash);
if consents.is_empty() {
    // Reject verification - no consent on file
}
```

### Supply Chain Integration
Track which drug batches were used for which patients:
```rust
contract.link_supply_chain(
    admin,
    data_hash,
    supply_contract_id,
    batch_number
);

// Query for safety monitoring
let supply_links = contract.get_supply_chain_links(data_hash);
// Cross-reference with supply chain contract for batch quality data
```

## Events Emitted

The contract emits events for external monitoring:
- `data_submitted`: When new data is submitted
- `data_verified`: When data is verified/rejected
- `discrepancy`: When data integrity issues detected (via utils)

## Deployment

```bash
# Build optimized WASM
stellar contract build

# Deploy to testnet
stellar contract deploy \
    --wasm target/wasm32v1-none/release/clinical_trial_data_verifier.wasm \
    --network testnet

# Initialize the deployed contract
stellar contract invoke \
    --id <CONTRACT_ID> \
    --network testnet \
    -- initialize \
    --admin <ADMIN_ADDRESS> \
    --trial_config <CONFIG_JSON>
```

## Contributing

This contract is designed for clinical trial data verification. When contributing:
1. Maintain ICH-GCP compliance
2. Add comprehensive tests
3. Document all public functions
4. Follow Rust/Soroban best practices

## Disclaimer

This smart contract is provided for educational and development purposes. For production use in clinical trials, ensure compliance with all applicable regulations (FDA, EMA, ICH-GCP, HIPAA, GDPR, etc.) and conduct thorough security audits.