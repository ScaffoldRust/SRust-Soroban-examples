# Patient Consent Management System

A Soroban smart contract for managing patient consents for medical data usage on the Stellar network. This contract enables patients to control and audit consents while ensuring compliance with privacy regulations like GDPR and HIPAA.

## Features

### Core Functionality

- **Consent Creation**: Patients create granular consents for specific parties and data types
- **Consent Management**: Update, suspend, resume, or revoke consents at any time
- **Consent Verification**: Check if a party has valid consent for specific data access
- **Automatic Expiration**: Time-bound consents automatically expire
- **Comprehensive Auditing**: Complete audit trail of all consent activities

### Privacy & Compliance

- **GDPR Compliant**: Freely given, specific, informed, and unambiguous consent
- **HIPAA Compliant**: Authorization for PHI disclosure with specific data scopes
- **Patient-Owned**: Patients maintain full control over their consents
- **Transparent**: Complete audit logs for regulatory compliance

### Data Scopes

- **Diagnostics**: Diagnostic test results
- **Imaging**: Medical imaging (X-rays, MRI, CT scans, etc.)
- **LabResults**: Laboratory test results
- **Prescriptions**: Medication and prescription data
- **Research**: Data for research purposes
- **Treatment**: Treatment and procedure data
- **AllData**: All medical data (comprehensive access)

## Architecture

```
patient-consent-management-system/
├── src/
│   ├── lib.rs        # Contract interface and main entry points
│   ├── consent.rs    # Consent creation, update, revocation logic
│   ├── audit.rs      # Audit logging and access tracking
│   ├── utils.rs      # Validation, compliance checks, utilities
│   └── test.rs       # Comprehensive test suite
├── Cargo.toml        # Rust package configuration
├── Makefile          # Build and deployment automation
└── README.md         # This file
```

## Data Structures

### Consent

```rust
pub struct Consent {
    pub consent_id: u64,
    pub patient: Address,
    pub authorized_party: Address,
    pub data_scopes: Vec<DataScope>,
    pub purpose: String,
    pub status: ConsentStatus,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub revoked_at: Option<u64>,
    pub last_updated: u64,
}
```

### ConsentStatus

- `Active` - Consent is currently active and valid
- `Revoked` - Consent has been revoked by patient
- `Expired` - Consent has expired (time-bound)
- `Suspended` - Temporarily suspended by patient

### AuditEvent

```rust
pub struct AuditEvent {
    pub event_id: u64,
    pub consent_id: u64,
    pub event_type: AuditEventType,
    pub actor: Address,
    pub timestamp: u64,
    pub details: Option<String>,
}
```

### ConsentAccessLog

```rust
pub struct ConsentAccessLog {
    pub consent_id: u64,
    pub accessed_by: Address,
    pub data_scope: DataScope,
    pub timestamp: u64,
    pub purpose: String,
}
```

## Key Functions

### Initialization

```rust
initialize()
```

Set up the contract with initial configuration.

### Consent Creation

```rust
create_consent(
    patient: Address,
    authorized_party: Address,
    data_scopes: Vec<DataScope>,
    purpose: String,
    expires_at: Option<u64>
) -> u64
```

Create a new consent. Patients specify who can access their data, what data types, and for what purpose. Optional expiration date for time-bound consents.

### Consent Management

```rust
update_consent(consent_id: u64, patient: Address, new_data_scopes: Option<Vec<DataScope>>, new_purpose: Option<String>, new_expires_at: Option<Option<u64>>)
revoke_consent(consent_id: u64, patient: Address)
suspend_consent(consent_id: u64, patient: Address)
resume_consent(consent_id: u64, patient: Address)
```

Patients can update consent details, suspend temporarily, resume, or permanently revoke at any time.

### Consent Verification

```rust
check_consent(consent_id: u64, authorized_party: Address, data_scope: DataScope) -> bool
is_consent_active(consent_id: u64) -> bool
```

Verify if a party has valid, active consent for specific data access. Automatically checks expiration and status.

### Query Functions

```rust
get_consent(consent_id: u64) -> Option<Consent>
get_patient_consents(patient: Address) -> Vec<u64>
get_party_consents(party: Address) -> Vec<u64>
```

Retrieve consent details and lists of consents for patients or authorized parties.

### Auditing

```rust
audit_log(consent_id: u64) -> Vec<AuditEvent>
get_access_logs(consent_id: u64) -> Vec<ConsentAccessLog>
log_access(consent_id: u64, accessed_by: Address, data_scope: DataScope, purpose: String)
get_audit_summary(patient: Address, consent_id: u64) -> Option<AuditSummary>
```

Complete audit trail of consent lifecycle events and data access logs.

### Utility Functions

```rust
validate_consent_params(purpose: String, expires_at: Option<u64>) -> bool
is_expired(consent_id: u64) -> bool
get_remaining_validity(consent_id: u64) -> Option<u64>
is_expiring_soon(consent_id: u64, warning_threshold: u64) -> bool
get_consent_age(consent_id: u64) -> u64
check_gdpr_compliance(consent_id: u64) -> bool
check_hipaa_compliance(consent_id: u64) -> bool
days_until_expiration(consent_id: u64) -> Option<u64>
```

Helper functions for validation, compliance checking, and consent lifecycle management.

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

## Usage Examples

### 1. Initialize Contract

```rust
contract.initialize();
```

### 2. Create Consent for Treatment

```rust
let mut scopes = Vec::new(&env);
scopes.push_back(DataScope::Diagnostics);
scopes.push_back(DataScope::LabResults);
scopes.push_back(DataScope::Treatment);

let consent_id = contract.create_consent(
    &patient,
    &doctor,
    &scopes,
    &"Treatment and ongoing care",
    &None  // No expiration
);
```

### 3. Create Time-Bound Research Consent

```rust
let mut scopes = Vec::new(&env);
scopes.push_back(DataScope::Research);

let expires_at = Some(env.ledger().timestamp() + (365 * 24 * 60 * 60)); // 1 year

let consent_id = contract.create_consent(
    &patient,
    &research_institution,
    &scopes,
    &"Clinical research study XYZ-123",
    &expires_at
);
```

### 4. Verify Consent Before Access

```rust
if contract.check_consent(&consent_id, &doctor, &DataScope::Diagnostics) {
    // Consent is valid, proceed with access
    contract.log_access(
        &consent_id,
        &doctor,
        &DataScope::Diagnostics,
        &"Viewing test results"
    );
}
```

### 5. Suspend and Resume Consent

```rust
// Patient wants to temporarily suspend consent
contract.suspend_consent(&consent_id, &patient);

// Later, patient resumes consent
contract.resume_consent(&consent_id, &patient);
```

### 6. Revoke Consent

```rust
// Patient permanently revokes consent
contract.revoke_consent(&consent_id, &patient);
```

### 7. Check Audit Trail

```rust
let audit_log = contract.audit_log(&consent_id);
let access_logs = contract.get_access_logs(&consent_id);

let summary = contract.get_audit_summary(&patient, &consent_id);
```

## Privacy & Compliance

### GDPR Compliance

The contract ensures GDPR compliance by:

1. **Freely Given Consent**: Patients create consents voluntarily without coercion
2. **Specific Purpose**: Every consent must specify a clear purpose
3. **Informed**: Data scopes clearly define what data is accessible
4. **Unambiguous**: Explicit consent creation by patient
5. **Right to Withdraw**: Patients can revoke consent at any time
6. **Transparency**: Complete audit trail of all consent activities

### HIPAA Compliance

The contract ensures HIPAA compliance by:

1. **Patient Authorization**: PHI disclosure requires explicit patient consent
2. **Specific Description**: Data scopes specify what information is disclosed
3. **Authorized Parties**: Clear identification of who receives the data
4. **Expiration**: Support for time-bound consents
5. **Right to Revoke**: Patients can revoke authorization at any time
6. **Audit Trail**: Complete logging of all access and modifications

## Storage Design

The contract uses efficient storage patterns:

- **Persistent Storage**:
  - Individual consent records by ID
  - Patient consent lists
  - Authorized party consent lists
  - Audit events and access logs

- **Instance Storage**:
  - Consent ID counter
  - Event ID counter
  - Access log ID counter

All storage keys are optimized to minimize transaction costs.

## Security Considerations

### Authentication

- All consent operations require patient authentication
- Only the consent owner (patient) can modify or revoke consents
- Authorized parties are verified before consent checks

### Data Protection

- No actual medical data stored on-chain
- Only consent metadata and access permissions
- Evidence of consent for compliance and auditing

### Privacy

- Consent records include minimal identifying information
- Access logs track who accessed what data and when
- Patients have complete visibility into data access

## Integration

### Medical Records Systems

Integrate with medical records contracts or off-chain systems:
- Check consent before allowing data access
- Log all access attempts for audit trail
- Support integration with `secure-medical-records-sharing` contract

### Identity Verification

Can be extended to integrate with:
- Decentralized identity systems
- Healthcare provider registries
- Patient identity verification services

### Clinical Research

Support clinical trials and research studies:
- Time-bound research consents
- Granular data scope control
- Automatic expiration handling

## Future Enhancements

- **Multi-Signature Consents**: Require multiple parties for sensitive operations
- **Consent Templates**: Pre-defined consent templates for common scenarios
- **Automated Notifications**: Alert patients before consent expiration
- **Consent Delegation**: Allow patients to delegate consent management
- **Batch Operations**: Process multiple consents efficiently
- **Advanced Analytics**: Consent usage statistics and patterns

## Gas Optimization

- Efficient storage key design minimizes costs
- Distributed data structures prevent large single entries
- Lazy loading of consent details
- Optimized for high-volume healthcare systems

## License

This contract is part of the SRust-Soroban-examples repository.

## References

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar Developer Resources](https://stellar.org/developers)
- [GDPR Data Protection](https://gdpr.eu/)
- [HIPAA Privacy Rule](https://www.hhs.gov/hipaa/for-professionals/privacy/index.html)

## Support

For issues, questions, or contributions, please visit the project repository.
