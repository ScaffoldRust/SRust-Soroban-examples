use soroban_sdk::{contracttype, Address, BytesN, String, Map};

/// Medical record structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecord {
    pub id: BytesN<32>,
    pub patient: Address,
    pub record_hash: BytesN<32>,        // Hash of the actual medical data (stored off-chain)
    pub record_type: String,            // e.g., "lab_result", "prescription", "diagnosis"
    pub metadata: RecordMetadata,
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u32,
}

/// Metadata associated with a medical record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordMetadata {
    pub title: String,
    pub description: String,
    pub provider_id: Option<Address>,    // Healthcare provider who created this record
    pub category: String,               // Medical category (cardiology, dermatology, etc.)
    pub sensitivity_level: SensitivityLevel,
    pub retention_period: u64,          // How long the record should be kept (in seconds)
    pub patient_notes: Option<String>,  // Patient's own notes about the record
}

/// Access levels for medical records
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessLevel {
    Read,           // Can read the record
    ReadWrite,      // Can read and update the record (rare)
    Emergency,      // Emergency access (special logging)
    Audit,          // Can only view audit logs
}

/// Sensitivity levels for medical data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SensitivityLevel {
    Low,            // Basic health information
    Medium,         // Standard medical records
    High,           // Sensitive conditions (mental health, genetic, etc.)
    Restricted,     // Highly restricted (substance abuse, etc.)
}

/// Access grant structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessGrant {
    pub provider: Address,
    pub access_level: AccessLevel,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
    pub purpose: String,                // Reason for access (treatment, consultation, etc.)
    pub granted_by: Address,            // Usually the patient
    pub revoked: bool,
    pub revoked_at: Option<u64>,
    pub access_count: u32,             // How many times this grant has been used
    pub last_accessed: Option<u64>,
}

/// Healthcare provider information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderInfo {
    pub license_number: String,
    pub name: String,
    pub specialty: String,
    pub organization: String,
    pub verified: bool,
    pub verification_date: u64,
    pub emergency_contact: bool,        // Can this provider access records in emergencies?
}

/// Audit event types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditEventType {
    RecordCreated,
    RecordUpdated,
    RecordAccessed,
    AccessGranted,
    AccessRevoked,
    AccessDenied,
    EmergencyAccess,
    ProviderRegistered,
    SystemInitialized,
    ConsentVerified,
    ConsentDenied,
    BulkAccessGranted,
}

/// Audit log entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEntry {
    pub sequence: u64,
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub actor: Address,                 // Who performed the action
    pub has_record_id: bool,           // Whether record_id field has a value
    pub record_id: BytesN<32>,         // Which record was involved (only valid if has_record_id is true)
    pub has_target: bool,              // Whether target field has a value  
    pub target: Address,               // Target of the action (only valid if has_target is true)
    pub details: String,               // Additional details about the event
}

/// Patient consent status (for integration with consent management system)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsentStatus {
    pub patient: Address,
    pub provider: Address,
    pub record_types: Map<String, bool>, // Map of record type to consent status
    pub general_consent: bool,
    pub expires_at: Option<u64>,
    pub last_updated: u64,
}

/// Emergency access request
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyAccessRequest {
    pub provider: Address,
    pub record_id: BytesN<32>,
    pub reason: String,
    pub emergency_contact: Address,
    pub timestamp: u64,
    pub approved: bool,
    pub approved_by: Option<Address>,
}

/// Bulk operation result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkOperationResult {
    pub total_requested: u32,
    pub successful: u32,
    pub failed: u32,
    pub failed_records: Map<BytesN<32>, String>, // Failed record ID to error message
}