use crate::{DataKey, MedicalRecord, RecordMetadata, AuditEventType, AccessGrant};
use crate::audit;
use soroban_sdk::{panic_with_error, Address, BytesN, Env, String, Vec, Map};
use crate::error::MedicalRecordsError;

/// Create a new medical record
pub fn create_record(
    env: &Env,
    patient: Address,
    record_hash: BytesN<32>,
    record_type: String,
    metadata: RecordMetadata,
) -> BytesN<32> {
    // Validate input
    if record_type.len() == 0 {
        panic_with_error!(env, MedicalRecordsError::InvalidRecordType);
    }
    
    // Generate record ID
    let mut next_id: u64 = env.storage().persistent()
        .get(&DataKey::NextRecordId)
        .unwrap_or(1);
    
    let record_id = BytesN::from_array(env, &generate_record_id(next_id));
    
    // Create the medical record
    let record = MedicalRecord {
        id: record_id.clone(),
        patient: patient.clone(),
        record_hash,
        record_type: record_type.clone(),
        metadata,
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
        version: 1,
    };
    
    // Store the record
    env.storage().persistent().set(&DataKey::Record(record_id.clone()), &record);
    
    // Update patient's record list
    let mut patient_records: Vec<BytesN<32>> = env.storage().persistent()
        .get(&DataKey::PatientRecords(patient.clone()))
        .unwrap_or(Vec::new(env));
    patient_records.push_back(record_id.clone());
    env.storage().persistent().set(&DataKey::PatientRecords(patient.clone()), &patient_records);
    
    // Update counters
    let record_count: u64 = env.storage().persistent()
        .get(&DataKey::RecordCount)
        .unwrap_or(0) + 1;
    env.storage().persistent().set(&DataKey::RecordCount, &record_count);
    env.storage().persistent().set(&DataKey::NextRecordId, &(next_id + 1));
    
    // Log the event
    audit::log_event(
        env,
        AuditEventType::RecordCreated,
        patient,
        Some(record_id.clone()),
        None,
    );
    
    record_id
}

/// Update an existing medical record (patient only)
pub fn update_record(
    env: &Env,
    patient: Address,
    record_id: BytesN<32>,
    new_record_hash: BytesN<32>,
    metadata: RecordMetadata,
) -> bool {
    // Get existing record
    let mut record: MedicalRecord = env.storage().persistent()
        .get(&DataKey::Record(record_id.clone()))
        .unwrap_or_else(|| panic_with_error!(env, MedicalRecordsError::RecordNotFound));
    
    // Verify ownership
    if record.patient != patient {
        panic_with_error!(env, MedicalRecordsError::RecordNotOwnedByPatient);
    }
    
    // Update the record
    record.record_hash = new_record_hash;
    record.metadata = metadata;
    record.updated_at = env.ledger().timestamp();
    record.version += 1;
    
    // Store the updated record
    env.storage().persistent().set(&DataKey::Record(record_id.clone()), &record);
    
    // Log the event
    audit::log_event(
        env,
        AuditEventType::RecordUpdated,
        patient,
        Some(record_id),
        None,
    );
    
    true
}

/// Get record metadata (with access control)
pub fn get_record_metadata(
    env: &Env,
    caller: Address,
    record_id: BytesN<32>,
) -> Option<RecordMetadata> {
    let record: MedicalRecord = env.storage().persistent()
        .get(&DataKey::Record(record_id.clone()))?;
    
    // Check if caller is the patient or has access
    if record.patient == caller {
        return Some(record.metadata);
    }
    
    // Check if caller has access permissions
    let access_grants: Map<Address, AccessGrant> = env.storage().persistent()
        .get(&DataKey::RecordAccess(record_id))
        .unwrap_or(Map::new(env));
    
    if let Some(grant) = access_grants.get(caller) {
        if !grant.revoked && 
           (grant.expires_at.is_none() || grant.expires_at.unwrap() > env.ledger().timestamp()) {
            return Some(record.metadata);
        }
    }
    
    None
}

/// Get all records for a patient
pub fn get_patient_records(env: &Env, patient: Address) -> Vec<BytesN<32>> {
    env.storage().persistent()
        .get(&DataKey::PatientRecords(patient))
        .unwrap_or(Vec::new(env))
}

/// Helper function to generate deterministic record IDs
fn generate_record_id(id: u64) -> [u8; 32] {
    let mut result = [0u8; 32];
    let id_bytes = id.to_be_bytes();
    result[24..].copy_from_slice(&id_bytes);
    result[0] = 0x01; // Version prefix for record IDs
    result
}

/// Validate record type
pub fn validate_record_type(_record_type: &String) -> bool {
    // In a real implementation, you would validate against allowed types
    // For now, we'll accept all record types for testing
    true
}

/// Get record by ID (with access control)
pub fn get_record(env: &Env, caller: Address, record_id: BytesN<32>) -> Option<MedicalRecord> {
    let record: MedicalRecord = env.storage().persistent()
        .get(&DataKey::Record(record_id.clone()))?;
    
    // Check if caller is the patient
    if record.patient == caller {
        return Some(record);
    }
    
    // This function should only return records for patients themselves
    // For provider access, use the access control functions
    None
}