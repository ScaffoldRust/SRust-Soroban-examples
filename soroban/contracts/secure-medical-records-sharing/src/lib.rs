#![no_std]


use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Vec};

mod access;
mod audit;
mod records;
mod error;
mod types;

pub use access::*;
pub use audit::*;
pub use records::*;
pub use error::*;
pub use types::*;

// Storage keys for the contract
#[contracttype]
pub enum DataKey {
    // Medical records storage
    Record(BytesN<32>),              // Record ID -> MedicalRecord
    PatientRecords(Address),          // Patient address -> Vec<RecordId>
    
    // Access control storage  
    RecordAccess(BytesN<32>),        // Record ID -> Map<Provider, AccessGrant>
    ProviderPermissions(Address),     // Provider address -> ProviderInfo
    
    // Audit logging storage
    AuditLog(u64),                   // Sequence number -> AuditEntry
    AuditCount,                      // Current audit log count
    
    // Administrative storage
    Admin,                           // Contract admin
    PatientConsentContract,          // Address of patient consent management contract
    
    // Counters
    RecordCount,                     // Total number of records
    NextRecordId,                    // Next available record ID
}

#[contract]
pub struct SecureMedicalRecordsContract;

#[contractimpl]
impl SecureMedicalRecordsContract {
    /// Initialize the contract with an admin and optional patient consent contract
    pub fn initialize(
        env: Env,
        admin: Address,
        patient_consent_contract: Option<Address>,
    ) {
        admin.require_auth();
        
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::RecordCount, &0u64);
        env.storage().persistent().set(&DataKey::NextRecordId, &1u64);
        env.storage().persistent().set(&DataKey::AuditCount, &0u64);
        
        if let Some(consent_contract) = patient_consent_contract {
            env.storage().persistent().set(&DataKey::PatientConsentContract, &consent_contract);
        }
        
        // Log initialization
        audit::log_event(&env, AuditEventType::SystemInitialized, admin.clone(), None, None);
    }
    
    /// Create a new medical record
    pub fn create_record(
        env: Env,
        patient: Address,
        record_hash: BytesN<32>,
        record_type: String,
        metadata: RecordMetadata,
    ) -> BytesN<32> {
        patient.require_auth();
        records::create_record(&env, patient, record_hash, record_type, metadata)
    }
    
    /// Grant access to a medical record for a healthcare provider
    pub fn grant_access(
        env: Env,
        patient: Address,
        record_id: BytesN<32>,
        provider: Address,
        access_level: AccessLevel,
        expires_at: Option<u64>,
        purpose: String,
    ) -> bool {
        patient.require_auth();
        access::grant_access(&env, patient, record_id, provider, access_level, expires_at, purpose)
    }
    
    /// Revoke access to a medical record
    pub fn revoke_access(
        env: Env,
        patient: Address,
        record_id: BytesN<32>,
        provider: Address,
    ) -> bool {
        patient.require_auth();
        access::revoke_access(&env, patient, record_id, provider)
    }
    
    /// Access a medical record (by authorized provider)
    pub fn access_record(
        env: Env,
        provider: Address,
        record_id: BytesN<32>,
        access_reason: String,
    ) -> Option<MedicalRecord> {
        provider.require_auth();
        access::access_record(&env, provider, record_id, access_reason)
    }
    
    /// Get record metadata (limited info for authorized parties)
    pub fn get_record_metadata(
        env: Env,
        caller: Address,
        record_id: BytesN<32>,
    ) -> Option<RecordMetadata> {
        caller.require_auth();
        records::get_record_metadata(&env, caller, record_id)
    }
    
    /// Get access permissions for a record
    pub fn get_access_permissions(
        env: Env,
        caller: Address,
        record_id: BytesN<32>,
    ) -> Option<Map<Address, AccessGrant>> {
        caller.require_auth();
        access::get_access_permissions(&env, caller, record_id)
    }
    
    /// Register a healthcare provider
    pub fn register_provider(
        env: Env,
        admin: Address,
        provider: Address,
        provider_info: ProviderInfo,
    ) -> bool {
        admin.require_auth();
        access::register_provider(&env, admin, provider, provider_info)
    }
    
    /// Get audit logs for a record
    pub fn get_audit_logs(
        env: Env,
        caller: Address,
        record_id: Option<BytesN<32>>,
        from_sequence: u64,
        limit: u32,
    ) -> Vec<AuditEntry> {
        caller.require_auth();
        audit::get_audit_logs(&env, caller, record_id, from_sequence, limit)
    }
    
    /// Verify patient consent (integration point)
    pub fn verify_patient_consent(
        env: Env,
        patient: Address,
        provider: Address,
        record_type: String,
    ) -> bool {
        access::verify_patient_consent(&env, patient, provider, record_type)
    }
    
    /// Get patient's records (for the patient themselves)
    pub fn get_patient_records(
        env: Env,
        patient: Address,
    ) -> Vec<BytesN<32>> {
        patient.require_auth();
        records::get_patient_records(&env, patient)
    }
    
    /// Emergency access (with special logging)
    pub fn emergency_access(
        env: Env,
        provider: Address,
        record_id: BytesN<32>,
        emergency_reason: String,
        emergency_contact: Address,
    ) -> Option<MedicalRecord> {
        provider.require_auth();
        access::emergency_access(&env, provider, record_id, emergency_reason, emergency_contact)
    }
    
    /// Update record (by patient only)
    pub fn update_record(
        env: Env,
        patient: Address,
        record_id: BytesN<32>,
        new_record_hash: BytesN<32>,
        metadata: RecordMetadata,
    ) -> bool {
        patient.require_auth();
        records::update_record(&env, patient, record_id, new_record_hash, metadata)
    }
    
    /// Bulk access grant (for efficiency)
    pub fn bulk_grant_access(
        env: Env,
        patient: Address,
        record_ids: Vec<BytesN<32>>,
        provider: Address,
        access_level: AccessLevel,
        expires_at: Option<u64>,
        purpose: String,
    ) -> u32 {
        patient.require_auth();
        access::bulk_grant_access(&env, patient, record_ids, provider, access_level, expires_at, purpose)
    }
}

#[cfg(test)]
mod tests;

