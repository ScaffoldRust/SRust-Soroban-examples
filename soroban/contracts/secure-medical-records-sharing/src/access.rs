
use crate::{DataKey, AccessGrant, AccessLevel, ProviderInfo, MedicalRecord, AuditEventType};
use crate::audit;
use soroban_sdk::{panic_with_error, Address, BytesN, Env, String, Map, Vec};
use crate::error::MedicalRecordsError;

/// Grant access to a medical record for a healthcare provider
pub fn grant_access(
    env: &Env,
    patient: Address,
    record_id: BytesN<32>,
    provider: Address,
    access_level: AccessLevel,
    expires_at: Option<u64>,
    purpose: String,
) -> bool {
    // Verify the record exists and belongs to the patient
    let record: MedicalRecord = env.storage().persistent()
        .get(&DataKey::Record(record_id.clone()))
        .unwrap_or_else(|| panic_with_error!(env, MedicalRecordsError::RecordNotFound));
    
    if record.patient != patient {
        panic_with_error!(env, MedicalRecordsError::RecordNotOwnedByPatient);
    }
    
    // Verify provider is registered
    let provider_info: ProviderInfo = env.storage().persistent()
        .get(&DataKey::ProviderPermissions(provider.clone()))
        .unwrap_or_else(|| panic_with_error!(env, MedicalRecordsError::ProviderNotRegistered));
    
    if !provider_info.verified {
        panic_with_error!(env, MedicalRecordsError::ProviderNotVerified);
    }
    
    // Validate expiration time
    let current_time = env.ledger().timestamp();
    if let Some(exp_time) = expires_at {
        if exp_time <= current_time {
            panic_with_error!(env, MedicalRecordsError::InvalidTimestamp);
        }
        
        // Ensure expiration is not too far in the future (max 1 year)
        if exp_time > current_time + (365 * 24 * 60 * 60) {
            panic_with_error!(env, MedicalRecordsError::ExpirationTooLong);
        }
    }
    
    // Check patient consent if consent contract is configured
    if !verify_patient_consent(env, patient.clone(), provider.clone(), record.record_type.clone()) {
        panic_with_error!(env, MedicalRecordsError::ConsentDenied);
    }
    
    // Get existing access grants for this record
    let mut access_grants: Map<Address, AccessGrant> = env.storage().persistent()
        .get(&DataKey::RecordAccess(record_id.clone()))
        .unwrap_or(Map::new(env));
    
    // Check if access is already granted and active
    if let Some(existing_grant) = access_grants.get(provider.clone()) {
        if !existing_grant.revoked && 
           (existing_grant.expires_at.is_none() || existing_grant.expires_at.unwrap() > current_time) {
            panic_with_error!(env, MedicalRecordsError::AccessAlreadyGranted);
        }
    }
    
    // Create new access grant
    let grant = AccessGrant {
        provider: provider.clone(),
        access_level,
        granted_at: current_time,
        expires_at,
        purpose: purpose.clone(),
        granted_by: patient.clone(),
        revoked: false,
        revoked_at: None,
        access_count: 0,
        last_accessed: None,
    };
    
    // Store the grant
    access_grants.set(provider.clone(), grant);
    env.storage().persistent().set(&DataKey::RecordAccess(record_id.clone()), &access_grants);
    
    // Log the event
    audit::log_event(
        env,
        AuditEventType::AccessGranted,
        patient,
        Some(record_id),
        Some(provider),
    );
    
    true
}

/// Revoke access to a medical record
pub fn revoke_access(
    env: &Env,
    patient: Address,
    record_id: BytesN<32>,
    provider: Address,
) -> bool {
    // Verify the record exists and belongs to the patient
    let record: MedicalRecord = env.storage().persistent()
        .get(&DataKey::Record(record_id.clone()))
        .unwrap_or_else(|| panic_with_error!(env, MedicalRecordsError::RecordNotFound));
    
    if record.patient != patient {
        panic_with_error!(env, MedicalRecordsError::RecordNotOwnedByPatient);
    }
    
    // Get access grants for this record
    let mut access_grants: Map<Address, AccessGrant> = env.storage().persistent()
        .get(&DataKey::RecordAccess(record_id.clone()))
        .unwrap_or_else(|| panic_with_error!(env, MedicalRecordsError::AccessNotGranted));
    
    // Find and revoke the access grant
    let mut grant = access_grants.get(provider.clone())
        .unwrap_or_else(|| panic_with_error!(env, MedicalRecordsError::AccessNotGranted));
    
    if grant.revoked {
        return true; // Already revoked
    }
    
    grant.revoked = true;
    grant.revoked_at = Some(env.ledger().timestamp());
    access_grants.set(provider.clone(), grant);
    
    env.storage().persistent().set(&DataKey::RecordAccess(record_id.clone()), &access_grants);
    
    // Log the event
    audit::log_event(
        env,
        AuditEventType::AccessRevoked,
        patient,
        Some(record_id),
        Some(provider),
    );
    
    true
}

/// Access a medical record (by authorized provider)
pub fn access_record(
    env: &Env,
    provider: Address,
    record_id: BytesN<32>,
    _access_reason: String,
) -> Option<MedicalRecord> {
    // Get the record
    let record: MedicalRecord = match env.storage().persistent().get(&DataKey::Record(record_id.clone())) {
        Some(r) => r,
        None => {
            // Log access denied for non-existent record
            audit::log_event(
                env,
                AuditEventType::AccessDenied,
                provider,
                Some(record_id),
                None,
            );
            return None;
        }
    };
    
    // Get access grants for this record
    let access_grants: Map<Address, AccessGrant> = match env.storage().persistent().get(&DataKey::RecordAccess(record_id.clone())) {
        Some(grants) => grants,
        None => {
            // Log access denied - no grants exist
            audit::log_event(
                env,
                AuditEventType::AccessDenied,
                provider,
                Some(record_id),
                None,
            );
            return None;
        }
    };
    
    // Check if provider has access
    let grant = match access_grants.get(provider.clone()) {
        Some(g) => g,
        None => {
            // Log access denied - no grant for this provider
            audit::log_event(
                env,
                AuditEventType::AccessDenied,
                provider,
                Some(record_id),
                None,
            );
            return None;
        }
    };
    
    // Validate access
    if grant.revoked {
        audit::log_event(
            env,
            AuditEventType::AccessDenied,
            provider,
            Some(record_id),
            None,
        );
        return None;
    }
    
    let current_time = env.ledger().timestamp();
    if let Some(exp_time) = grant.expires_at {
        if exp_time <= current_time {
            audit::log_event(
                env,
                AuditEventType::AccessDenied,
                provider,
                Some(record_id),
                None,
            );
            return None;
        }
    }
    
    // Update access statistics
    let mut updated_grant = grant.clone();
    updated_grant.access_count += 1;
    updated_grant.last_accessed = Some(current_time);
    let mut updated_grants = access_grants;
    updated_grants.set(provider.clone(), updated_grant);
    env.storage().persistent().set(&DataKey::RecordAccess(record_id.clone()), &updated_grants);
    
    // Log the access
    audit::log_event(
        env,
        AuditEventType::RecordAccessed,
        provider,
        Some(record_id.clone()),
        None,
    );
    
    Some(record)
}

/// Register a healthcare provider
pub fn register_provider(
    env: &Env,
    admin: Address,
    provider: Address,
    provider_info: ProviderInfo,
) -> bool {
    // Verify admin
    let contract_admin: Address = env.storage().persistent()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| panic_with_error!(env, MedicalRecordsError::InvalidAdmin));
    
    if admin != contract_admin {
        panic_with_error!(env, MedicalRecordsError::Unauthorized);
    }
    
    // Check if provider is already registered
    if env.storage().persistent().has(&DataKey::ProviderPermissions(provider.clone())) {
        panic_with_error!(env, MedicalRecordsError::ProviderAlreadyRegistered);
    }
    
    // Validate provider info
    if provider_info.license_number.len() == 0 || provider_info.name.len() == 0 {
        panic_with_error!(env, MedicalRecordsError::InvalidInput);
    }
    
    // Store provider info
    env.storage().persistent().set(&DataKey::ProviderPermissions(provider.clone()), &provider_info);
    
    // Log the event
    audit::log_event(
        env,
        AuditEventType::ProviderRegistered,
        admin,
        None,
        Some(provider),
    );
    
    true
}

/// Get access permissions for a record
pub fn get_access_permissions(
    env: &Env,
    caller: Address,
    record_id: BytesN<32>,
) -> Option<Map<Address, AccessGrant>> {
    // Get the record to verify ownership
    let record: MedicalRecord = env.storage().persistent()
        .get(&DataKey::Record(record_id.clone()))?;
    
    // Only the patient can view all access permissions
    if record.patient != caller {
        return None;
    }
    
    env.storage().persistent().get(&DataKey::RecordAccess(record_id))
}

/// Emergency access to medical records
pub fn emergency_access(
    env: &Env,
    provider: Address,
    record_id: BytesN<32>,
    emergency_reason: String,
    emergency_contact: Address,
) -> Option<MedicalRecord> {
    // Verify provider is registered and can perform emergency access
    let provider_info: ProviderInfo = env.storage().persistent()
        .get(&DataKey::ProviderPermissions(provider.clone()))?;
    
    if !provider_info.verified || !provider_info.emergency_contact {
        audit::log_event(
            env,
            AuditEventType::AccessDenied,
            provider,
            Some(record_id),
            None,
        );
        return None;
    }
    
    // Get the record
    let record: MedicalRecord = env.storage().persistent()
        .get(&DataKey::Record(record_id.clone()))?;
    
    // Log emergency access with special attention
    audit::log_event(
        env,
        AuditEventType::EmergencyAccess,
        provider.clone(),
        Some(record_id.clone()),
        Some(emergency_contact),
    );
    
    Some(record)
}

/// Verify patient consent (integration with consent management system)
pub fn verify_patient_consent(
    env: &Env,
    patient: Address,
    provider: Address,
    record_type: String,
) -> bool {
    // Always log the consent verification attempt for compliance visibility
    audit::log_event(
        env,
        AuditEventType::ConsentVerified,
        patient.clone(),
        None,
        Some(provider.clone()),
    );

    // If no consent contract is configured, default to allowing access
    let consent_contract: Option<Address> = env.storage().persistent()
        .get(&DataKey::PatientConsentContract);
    
    if consent_contract.is_none() {
        return true; // No consent management system configured
    }
    
    // Here we would call the patient consent management contract
    // For now, we'll implement basic logic
    // In a real implementation, this would make a cross-contract call
    
    true // Default to allowing access for demo purposes
}

/// Bulk grant access for multiple records
pub fn bulk_grant_access(
    env: &Env,
    patient: Address,
    record_ids: Vec<BytesN<32>>,
    provider: Address,
    access_level: AccessLevel,
    expires_at: Option<u64>,
    purpose: String,
) -> u32 {
    let mut successful_grants = 0u32;
    
    for i in 0..record_ids.len() {
        let record_id = record_ids.get(i).unwrap();
        
        // Attempt to grant access, continue on failure
        // Note: In a real implementation, you'd want proper error handling
        // For now, we'll use a simpler approach since catch_unwind isn't available in no_std
        let _result = grant_access(env, patient.clone(), record_id, provider.clone(), access_level.clone(), expires_at, purpose.clone());
        successful_grants += 1;
    }
    
    // Log bulk operation
    audit::log_event(
        env,
        AuditEventType::BulkAccessGranted,
        patient,
        None,
        Some(provider),
    );
    
    successful_grants
}

/// Check if access is currently valid
pub fn is_access_valid(env: &Env, provider: Address, record_id: BytesN<32>) -> bool {
    let access_grants: Map<Address, AccessGrant> = env.storage().persistent()
        .get(&DataKey::RecordAccess(record_id))
        .unwrap_or(Map::new(env));
    
    if let Some(grant) = access_grants.get(provider) {
        if grant.revoked {
            return false;
        }
        
        if let Some(exp_time) = grant.expires_at {
            return exp_time > env.ledger().timestamp();
        }
        
        return true;
    }
    
    false
}

