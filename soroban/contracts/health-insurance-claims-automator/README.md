# Health Insurance Claims Automator

A Soroban smart contract for automating health insurance claims processing on the Stellar network. This contract streamlines claim submissions, approvals, and payouts while ensuring compliance with insurance regulations including HIPAA.

## Features

### Core Functionality

- **Automated Claim Processing**: Rule-based verification and auto-approval for eligible claims
- **Manual Review Support**: Insurance providers can review and approve/reject claims manually
- **Secure Payout Execution**: Automated payout processing for approved claims
- **Dispute Resolution**: Built-in mechanism for handling contested claims
- **HIPAA Compliant**: Secure handling of sensitive claim data with evidence hash pointers

### Role-Based Access Control

- **Patients**: Submit claims with medical evidence and track claim status
- **Insurers**: Process claims, approve/reject, and execute payouts
- **Administrators**: Configure auto-approval rules and resolve disputes

## Architecture

```
health-insurance-claims-automator/
├── src/
│   ├── lib.rs        # Contract interface and main entry points
│   ├── claims.rs     # Claim submission, status tracking, and dispute handling
│   ├── automator.rs  # Auto-approval rules and payout automation
│   ├── utils.rs      # Validation, calculations, and compliance helpers
│   └── test.rs       # Comprehensive test suite
├── Cargo.toml        # Rust package configuration
├── Makefile          # Build and deployment automation
└── README.md         # This file
```

## Data Structures

### Claim

```rust
pub struct Claim {
    pub claim_id: u64,
    pub patient: Address,
    pub insurer: Address,
    pub claim_amount: i128,
    pub approved_amount: i128,
    pub currency: Address,
    pub diagnosis_code: String,
    pub evidence_hash: BytesN<32>,  // Hash pointer to medical records
    pub status: ClaimStatus,
    pub submitted_at: u64,
    pub reviewed_at: Option<u64>,
    pub approved_at: Option<u64>,
    pub paid_at: Option<u64>,
    pub rejection_reason: Option<String>,
    pub auto_approved: bool,
}
```

### ClaimStatus

- `Submitted` - Claim submitted by patient
- `UnderReview` - Claim is being reviewed
- `Approved` - Claim approved for payout
- `Rejected` - Claim rejected
- `Paid` - Payout completed
- `Disputed` - Claim is under dispute
- `Cancelled` - Claim cancelled by patient

### AutoApprovalRule

```rust
pub struct AutoApprovalRule {
    pub rule_id: u64,
    pub max_amount: i128,
    pub diagnosis_codes: Vec<String>,
    pub enabled: bool,
}
```

## Key Functions

### Initialization

```rust
initialize(admin: Address, default_max_auto_amount: i128)
```

Set up the contract with initial configuration and default auto-approval rules.

### Claim Submission

```rust
submit_claim(
    patient: Address,
    insurer: Address,
    claim_amount: i128,
    currency: Address,
    diagnosis_code: String,
    evidence_hash: BytesN<32>
) -> u64
```

Patients submit claims with required medical information and evidence hash.

### Claim Processing

```rust
process_claim(claim_id: u64, insurer: Address)
```

Initiates claim processing and checks auto-approval rules. Claims meeting criteria are automatically approved.

### Manual Approval

```rust
approve_claim(claim_id: u64, insurer: Address, approved_amount: i128)
```

Insurers manually approve claims that require review, potentially with adjusted amounts.

### Rejection

```rust
reject_claim(claim_id: u64, insurer: Address, rejection_reason: String)
```

Insurers reject claims that don't meet coverage criteria.

### Payout Execution

```rust
payout(claim_id: u64, insurer: Address)
```

Execute payout to patient for approved claims.

### Dispute Resolution

```rust
file_dispute(claim_id: u64, disputer: Address, reason: String)
resolve_dispute(claim_id: u64, admin: Address, resolution: String, final_status: ClaimStatus)
```

Patients or insurers can file disputes, which administrators resolve.

### Auto-Approval Rules

```rust
add_auto_approval_rule(admin: Address, max_amount: i128, diagnosis_codes: Vec<String>) -> u64
update_auto_approval_rule(admin: Address, rule_id: u64, max_amount: Option<i128>, diagnosis_codes: Option<Vec<String>>, enabled: Option<bool>)
```

Configure rules for automatic claim approval based on amount thresholds and diagnosis codes.

### Query Functions

```rust
get_status(claim_id: u64) -> Option<ClaimStatus>
get_claim(claim_id: u64) -> Option<Claim>
get_patient_claims(patient: Address) -> Vec<u64>
get_insurer_claims(insurer: Address) -> Vec<u64>
get_pending_queue() -> Vec<u64>
get_payout_record(claim_id: u64) -> Option<PayoutRecord>
get_dispute(claim_id: u64) -> Option<DisputeRecord>
```

## Build and Test

### Prerequisites

- Rust toolchain
- Stellar CLI (`stellar`)
- Soroban SDK

### Building

```bash
# Build the contract
make build

# Or using stellar CLI directly
stellar contract build
```

### Testing

```bash
# Run all tests
make test

# Or using cargo directly
cargo test
```

### Formatting and Linting

```bash
# Format code
make fmt

# Run linter
make lint

# Run all checks
make check
```

## Usage Example

### 1. Initialize Contract

```rust
contract.initialize(&admin, &10000); // Max auto-approval: 10,000 units
```

### 2. Submit a Claim

```rust
let claim_id = contract.submit_claim(
    &patient,
    &insurer,
    &5000,                    // Claim amount
    &currency_token,
    &"J20.9",                 // ICD-10 diagnosis code
    &evidence_hash            // Hash of medical records
);
```

### 3. Process Claim (Auto-Approval)

```rust
contract.process_claim(&claim_id, &insurer);
// If amount ≤ 10,000, claim is auto-approved
```

### 4. Execute Payout

```rust
contract.payout(&claim_id, &insurer);
```

## Security and Compliance

### HIPAA Compliance

- **Data Privacy**: Medical evidence stored off-chain; only hashes stored on-chain
- **Access Control**: Role-based permissions ensure only authorized parties access claims
- **Audit Trail**: Complete timestamp history for all claim status changes

### Security Best Practices

- **Authentication**: All sensitive operations require caller authentication
- **Input Validation**: Comprehensive validation of all claim parameters
- **Storage Optimization**: Distributed storage across multiple slots to avoid entry size limits
- **Error Handling**: Clear error messages without exposing sensitive information

## Storage Design

The contract uses Soroban's persistent storage with optimized key structures:

- **Claims**: `(CLAIMS, claim_id)` - Individual claim records
- **Patient Claims**: `(PAT_CLM, patient_address)` - List of claims per patient
- **Insurer Claims**: `(INS_CLM, insurer_address)` - List of claims per insurer
- **Pending Queue**: `PEND_Q` - Queue of claims awaiting processing
- **Disputes**: `(DISPUTES, claim_id)` - Dispute records
- **Auto-Approval Rules**: `(AUTO_RLS, rule_id)` - Approval automation rules
- **Payouts**: `(PAYOUTS, claim_id)` - Payout transaction records

Instance storage is used for:
- `ADMIN` - Contract administrator address
- `NEXT_CLAIM_ID` - Counter for generating unique claim IDs
- `NEXT_RULE_ID` - Counter for generating unique rule IDs

## Integration

### Medical Records Integration

The contract accepts evidence hashes (`BytesN<32>`), allowing integration with:
- Medical records sharing contracts (e.g., `secure-medical-records-sharing`)
- Off-chain encrypted storage systems
- IPFS or other decentralized storage solutions

### Payment Integration

The contract can be extended to integrate with:
- Telemedicine payment gateways
- Token transfer contracts
- Multi-currency payment systems

## Future Enhancements

- **Oracle Integration**: Real-time verification of medical procedures and pricing
- **Multi-Signature Approvals**: Require multiple approvers for high-value claims
- **Periodic Coverage Limits**: Track and enforce annual/monthly coverage caps
- **Provider Networks**: Verify claims against in-network providers
- **Automated Fraud Detection**: Pattern analysis for suspicious claim activity

## Gas Optimization

- Efficient storage key design minimizes storage costs
- Distributed data structures prevent large single entries
- Lazy loading of claim details reduces transaction costs
- Batch processing capabilities for high-volume periods

## License

This contract is part of the SRust-Soroban-examples repository.

## References

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar Developer Resources](https://stellar.org/developers)
- [HIPAA Guidelines](https://www.hhs.gov/hipaa/index.html)
- [ICD-10 Diagnosis Codes](https://www.cdc.gov/nchs/icd/icd-10-cm.htm)

## Support

For issues, questions, or contributions, please visit the project repository.
