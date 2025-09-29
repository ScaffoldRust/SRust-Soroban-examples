# EHR Interoperability Bridge Smart Contract

A comprehensive smart contract for enabling seamless data exchange between Electronic Health Record (EHR) systems on the Stellar network using Soroban. This contract ensures secure, standardized, and compliant data transfers while adhering to healthcare interoperability standards like HL7 FHIR and SMART on FHIR.

## Overview

The EHR Interoperability Bridge acts as a secure middleware layer that enables different healthcare systems to share patient data while maintaining strict compliance with healthcare regulations and privacy standards.

## Features

### 🏥 Core EHR Interoperability
- **Multi-System Registration**: Register and manage multiple EHR systems with their capabilities
- **Data Request Management**: Initiate and track data requests between healthcare systems
- **Format Validation**: Support for HL7 FHIR, HL7 v2, CDA, DICOM, and custom formats
- **Secure Data Transfer**: Encrypted and verified data exchange with audit trails

### 🔐 Healthcare Compliance
- **Patient Consent Management**: Comprehensive consent verification before any data sharing
- **HIPAA Compliance**: Built-in privacy protections and access controls
- **HL7 FHIR Standard**: Native support for FHIR resource types and operations
- **SMART on FHIR**: Authorization framework integration for healthcare applications
- **Role-Based Access**: Different permissions for patients, doctors, nurses, and administrators

### 🛡️ Security & Privacy
- **Cryptographic Verification**: Digital signatures for all transactions
- **Audit Logging**: Complete audit trail for compliance reporting
- **Data Integrity**: Hash-based verification of all transferred data
- **Access Control**: Multi-level authorization and authentication

## Architecture

```
ehr-interoperability-bridge/
├── src/
│   ├── lib.rs              # Main contract interface and data structures
│   ├── bridge.rs           # Core EHR bridging and data flow logic
│   ├── interoperability.rs # Consent management and compliance protocols
│   ├── utils.rs            # Format conversion and validation utilities
│   └── test.rs             # Comprehensive test suite
├── Cargo.toml              # Rust dependencies and configuration
├── Makefile                # Build and deployment automation
└── README.md               # This documentation
```

## Key Data Structures

### EhrSystem
Represents a registered healthcare system with its capabilities:
```rust
struct EhrSystem {
    system_id: String,           // Unique system identifier
    name: String,                // Human-readable system name
    endpoint: String,            // API endpoint URL
    supported_formats: Vec<String>, // Supported data formats
    public_key: BytesN<32>,      // System's public key for verification
    admin: Address,              // System administrator address
    is_active: bool,             // System activation status
}
```

### DataRequest
Manages data exchange requests between systems:
```rust
struct DataRequest {
    request_id: BytesN<32>,      // Unique request identifier
    sender_system: String,       // Requesting system
    receiver_system: String,     // Target system
    patient_id: String,          // Patient identifier
    data_types: Vec<String>,     // Requested data types
    requester: Address,          // Address of requester
    consent_verified: bool,      // Patient consent status
    status: RequestStatus,       // Current request status
    timestamp: u64,              // Request creation time
    expiry: u64,                 // Request expiration time
}
```

### ConsentRecord
Tracks patient consent for data sharing:
```rust
struct ConsentRecord {
    patient_id: String,              // Patient identifier
    patient_address: Address,        // Patient's blockchain address
    authorized_systems: Vec<String>, // Systems authorized to access data
    data_types_permitted: Vec<String>, // Types of data that can be shared
    consent_expiry: u64,             // When consent expires
    revoked: bool,                   // Whether consent has been revoked
}
```

## Core Functions

### System Management

#### `initialize(admin: Address)`
Initializes the bridge contract with an administrator.

#### `register_ehr_system(...)`
Registers a new EHR system with the bridge.
```rust
register_ehr_system(
    admin: Address,
    system_id: String,
    name: String,
    endpoint: String,
    supported_formats: Vec<String>,
    public_key: BytesN<32>,
    system_admin: Address,
) -> bool
```

### Data Exchange

#### `request_data(...)`
Initiates a data request between EHR systems.
```rust
request_data(
    requester: Address,
    sender_system: String,
    receiver_system: String,
    patient_id: String,
    data_types: Vec<String>,
    expiry_hours: u64,
) -> BytesN<32>
```

#### `transfer_data(...)`
Executes data transfer after consent verification.
```rust
transfer_data(
    request_id: BytesN<32>,
    data_hash: BytesN<32>,
    source_format: String,
    target_format: String,
    validator: Address,
) -> BytesN<32>
```

### Consent Management

#### `set_patient_consent(...)`
Sets patient consent for data sharing.
```rust
set_patient_consent(
    patient: Address,
    patient_id: String,
    authorized_systems: Vec<String>,
    data_types_permitted: Vec<String>,
    consent_duration_hours: u64,
) -> bool
```

#### `verify_consent(...)`
Verifies patient consent for a specific data request.
```rust
verify_consent(
    patient: Address,
    request_id: BytesN<32>,
    consent_signature: BytesN<64>,
) -> bool
```

### Format Validation

#### `validate_format(...)`
Validates and converts data between formats.
```rust
validate_format(
    data_hash: BytesN<32>,
    source_format: String,
    target_format: String,
) -> bool
```

#### `check_compatibility(...)`
Checks if two systems can exchange specific data types.
```rust
check_compatibility(
    source_system: String,
    target_system: String,
    data_type: String,
) -> bool
```

## Supported Data Formats

### HL7 FHIR (Fast Healthcare Interoperability Resources)
- **HL7_FHIR_R4**: Latest FHIR specification
- **HL7_FHIR_STU3**: Previous FHIR version
- **Supported Resources**: Patient, Observation, Condition, Medication, Procedure, DiagnosticReport, Encounter, AllergyIntolerance, Immunization

### HL7 v2
- **Message Types**: ADT (Admission/Discharge/Transfer), ORM (Order Management), ORU (Observation Result), SIU (Scheduling), DFT (Detailed Financial Transaction)

### Clinical Document Architecture (CDA)
- **CDA_R2**: Clinical Document Architecture Release 2
- **Document Types**: Clinical Documents, Continuity of Care Documents, Discharge Summaries, Progress Notes

### DICOM (Digital Imaging and Communications in Medicine)
- **Modalities**: CT, MR, US, XR, NM, PT, RF, DX

### Generic Formats
- **JSON**: JavaScript Object Notation
- **XML**: Extensible Markup Language

## Usage Examples

### 1. Initialize the Contract
```rust
let admin = Address::generate(&env);
contract.initialize(&admin);
```

### 2. Register Healthcare Systems
```rust
// Register Hospital A
let system_id = String::from_str(&env, "hospital_a");
let name = String::from_str(&env, "Metropolitan General Hospital");
let endpoint = String::from_str(&env, "https://hospital-a.com/fhir");
let mut formats = Vec::new(&env);
formats.push_back(String::from_str(&env, "HL7_FHIR_R4"));
formats.push_back(String::from_str(&env, "HL7_V2"));

let public_key = env.crypto().sha256(b"hospital_a_public_key");
let system_admin = Address::generate(&env);

contract.register_ehr_system(
    &admin,
    &system_id,
    &name,
    &endpoint,
    &formats,
    &public_key,
    &system_admin,
);
```

### 3. Set Patient Consent
```rust
let patient = Address::generate(&env);
let patient_id = String::from_str(&env, "patient_12345");
let mut authorized_systems = Vec::new(&env);
authorized_systems.push_back(String::from_str(&env, "hospital_a"));
authorized_systems.push_back(String::from_str(&env, "clinic_b"));

let mut data_types = Vec::new(&env);
data_types.push_back(String::from_str(&env, "Patient"));
data_types.push_back(String::from_str(&env, "Observation"));

contract.set_patient_consent(
    &patient,
    &patient_id,
    &authorized_systems,
    &data_types,
    &720, // 30 days in hours
);
```

### 4. Request Patient Data
```rust
let requester = Address::generate(&env);
let sender_system = String::from_str(&env, "hospital_a");
let receiver_system = String::from_str(&env, "clinic_b");
let patient_id = String::from_str(&env, "patient_12345");

let mut requested_data = Vec::new(&env);
requested_data.push_back(String::from_str(&env, "Patient"));
requested_data.push_back(String::from_str(&env, "Observation"));

let request_id = contract.request_data(
    &requester,
    &sender_system,
    &receiver_system,
    &patient_id,
    &requested_data,
    &24, // 24 hours expiry
);
```

### 5. Verify Consent and Transfer Data
```rust
// Patient verifies consent
let consent_signature = env.crypto().sha256(b"patient_consent_signature");
contract.verify_consent(&patient, &request_id, &consent_signature);

// Transfer data after validation
let data_hash = env.crypto().sha256(b"patient_data_content");
let validator = Address::generate(&env);

let transfer_id = contract.transfer_data(
    &request_id,
    &data_hash,
    &String::from_str(&env, "HL7_FHIR_R4"),
    &String::from_str(&env, "JSON"),
    &validator,
);
```

## Building and Testing

### Prerequisites
```bash
# Install Rust and required targets
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install --locked soroban-cli
```

### Build Commands
```bash
# Standard build
make build

# Build with Soroban CLI
make stellar-build

# Run tests
make test

# Run all checks
make all

# Install all dependencies
make install-deps
```

### Development Workflow
```bash
# Format code
make fmt

# Run linter
make clippy

# Watch for changes (requires cargo-watch)
make watch

# Build optimized WASM
make optimize

# Check contract size
make size
```

## Testing

The contract includes comprehensive tests covering:

- **System Registration**: Testing EHR system registration and management
- **Data Requests**: Validating request creation and management
- **Consent Verification**: Testing patient consent workflows
- **Data Transfer**: Validating secure data transfer mechanisms
- **Format Validation**: Testing data format conversion and validation
- **Error Handling**: Testing edge cases and error conditions
- **Access Control**: Verifying authorization and authentication

Run specific tests:
```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_register_ehr_system -- --nocapture
```

## Security Considerations

### Implemented Protections
- ✅ **Patient Consent Verification**: No data transfers without explicit patient consent
- ✅ **System Authentication**: Digital signature verification for all operations
- ✅ **Data Integrity**: Hash-based verification of all transferred data
- ✅ **Access Control**: Role-based permissions and authorization checks
- ✅ **Audit Logging**: Complete audit trail for all operations
- ✅ **Time-bound Requests**: Automatic expiration of data requests
- ✅ **Format Validation**: Strict validation of data formats and conversions

### Best Practices
- Always verify patient consent before data access
- Use strong cryptographic signatures for system authentication
- Regularly audit system access logs
- Monitor for unauthorized access attempts
- Keep system credentials secure and rotate regularly
- Ensure HIPAA compliance in all data handling

## Compliance Standards

### HL7 FHIR Compliance
- Support for FHIR R4 and STU3 specifications
- Standard FHIR resource types and operations
- RESTful API patterns for data exchange

### SMART on FHIR
- OAuth2-based authorization framework
- Scoped access controls (patient/*.read, user/*.read, etc.)
- Integration with healthcare applications

### HIPAA Compliance
- Patient privacy protections
- Audit logging requirements
- Access control and authentication
- Data encryption and integrity verification

## Contributing

1. Follow Rust best practices and coding standards
2. Ensure all tests pass before submitting changes
3. Add comprehensive tests for new functionality
4. Update documentation for any API changes
5. Maintain HIPAA and FHIR compliance in all changes

## References

- [HL7 FHIR Specification](https://hl7.org/fhir/)
- [SMART on FHIR](https://docs.smarthealthit.org/)
- [HIPAA Security Rule](https://www.hhs.gov/hipaa/for-professionals/security/index.html)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Network](https://developers.stellar.org/)

## License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**⚕️ Healthcare Data Interoperability Made Secure and Compliant**