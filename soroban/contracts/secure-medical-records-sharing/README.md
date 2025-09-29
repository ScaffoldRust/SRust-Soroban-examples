# Secure Medical Records Sharing Contract

A Soroban smart contract enabling privacy-preserving sharing of medical records with granular, patient-controlled access permissions and emergency access pathways.

## Features
- Patient-owned medical record storage (on-chain pointers / hashes to off-chain encrypted data)
- Granular access grants per provider and per data type (e.g. lab, imaging, rx)
- Time-bound permissions with expiration timestamps
- Revocation of individual grants
- Emergency provider whitelist with monitored usage
- Audit log capped to last 200 events (add/update/grant/revoke/emergency)

## Data Model
```
DataKey::Config -> ContractConfig { initialized }
DataKey::RecordCounter(patient) -> u64
DataKey::Record(patient, id) -> MedicalRecord
DataKey::Access(patient, provider) -> AccessGrant
DataKey::Audit(patient) -> Vec<AuditEvent> (capped 200)
DataKey::EmergencyWhitelist(patient, provider) -> bool
DataKey::EmergencyUse(patient, provider) -> EmergencyAccessUse
```

## Core Types
- MedicalRecord { record_id, patient, data_type, pointer, created_at, updated_at, active }
- AccessGrant { provider, data_types<Vec<String>>, expires_at, revoked, granted_at }
- AuditEvent { timestamp, actor, action(Symbol), record_id? , detail? }
- EmergencyAccessUse { last_used, uses }

## Public Functions
| Function | Description |
|----------|-------------|
| initialize() | One-time contract initialization |
| add_record(patient, data_type, pointer) -> id | Add new medical record pointer |
| update_record(patient, record_id, new_pointer) | Update record pointer |
| grant_access(patient, provider, data_types, expires_at) | Grant scoped access |
| revoke_access(patient, provider) | Revoke provider access |
| verify_access(patient, provider, data_type) -> bool | Check provider permission |
| add_emergency_provider(patient, provider) | Whitelist provider for emergency |
| remove_emergency_provider(patient, provider) | Remove emergency whitelist |
| emergency_read(provider, patient, record_id, justification) -> MedicalRecord | Access record in emergency (logs justification) |
| get_record(requester, patient, record_id) -> MedicalRecord | Fetch a record (caller must be patient or have access) |
| list_records(patient, owner, offset, limit, data_type?) -> Vec<MedicalRecord> | Patient enumerates own records |
| get_audit_log(patient, owner) -> Vec<AuditEvent> | Patient views audit log |
| get_access_grant(patient, provider) -> Option<AccessGrant> | Inspect a grant |

## Privacy & Compliance Notes
- Actual PHI content SHOULD remain encrypted off-chain; contract stores only pointers/hashes.
- Access control verified on-chain to gate retrieval of the off-chain reference.
- Audit log provides immutable evidence of access-related actions (capped for cost control).

## Build & Test
```
make build
make unit-test
```
Or directly:
```
cargo build --target wasm32-unknown-unknown --release
cargo test --package secure-medical-records-sharing -- --nocapture
```

## Gas & Storage Considerations
- Records stored individually to allow partial retrieval.
- Single AccessGrant per (patient, provider) avoids scanning large vectors.
- Capped audit log prevents unbounded growth (oldest dropped when >200).

## Future Extensions
- Encryption key escrow / rotation primitives
- Delegated consent (care team / institution-level roles)
- Fine-grained emergency throttle & multi-sig approval
- Paginated provider-facing discovery of accessible record IDs via opt-in indexing.

## Disclaimer
This sample is illustrative and not a complete HIPAA/GDPR compliance solution. Off-chain processes, encryption, and legal workflows must be implemented appropriately.
