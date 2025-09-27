use crate::{DataKey, AuditEntry, AuditEventType, MedicalRecord, AccessGrant};
use soroban_sdk::{panic_with_error, Address, BytesN, Env, String, Vec, Map, testutils::Address as _};
use crate::error::MedicalRecordsError;
use core::cmp;

/// Log an audit event
pub fn log_event(
    env: &Env,
    event_type: AuditEventType,
    actor: Address,
    record_id: Option<BytesN<32>>,
    target: Option<Address>,
) {
    // Get current audit count
    let mut audit_count: u64 = env.storage().persistent()
        .get(&DataKey::AuditCount)
        .unwrap_or(0);
    
    audit_count += 1;
    
    // Create audit entry
    let details = generate_event_details(env, &event_type, &record_id, &target);
    
    let (has_record_id, record_id_val) = if let Some(rid) = record_id {
        (true, rid)
    } else {
        (false, BytesN::from_array(env, &[0u8; 32]))
    };
    
    let (has_target, target_val) = if let Some(tgt) = target {
        (true, tgt)
    } else {
        (false, Address::generate(env))
    };
    
    let entry = AuditEntry {
        sequence: audit_count,
        timestamp: env.ledger().timestamp(),
        event_type,
        actor,
        has_record_id,
        record_id: record_id_val,
        has_target,
        target: target_val,
        details,
    };
    
    // Store the audit entry
    env.storage().persistent().set(&DataKey::AuditLog(audit_count), &entry);
    env.storage().persistent().set(&DataKey::AuditCount, &audit_count);
}

/// Get audit logs with access control
pub fn get_audit_logs(
    env: &Env,
    caller: Address,
    record_id: Option<BytesN<32>>,
    from_sequence: u64,
    limit: u32,
) -> Vec<AuditEntry> {
    let mut logs = Vec::new(env);
    
    // Verify caller has permission to view audit logs
    if let Some(rid) = &record_id {
        // Check if caller owns the record or has audit access
        if !can_access_audit_logs(env, &caller, rid) {
            return logs; // Empty logs if no access
        }
    } else {
        // For system-wide logs, only admin can access
        let admin: Address = env.storage().persistent()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, MedicalRecordsError::InvalidAdmin));
        
        if caller != admin {
            return logs; // Empty logs for non-admin
        }
    }
    
    let current_audit_count: u64 = env.storage().persistent()
        .get(&DataKey::AuditCount)
        .unwrap_or(0);
    
    let end_sequence = cmp::min(current_audit_count, from_sequence + limit as u64);
    
    for seq in from_sequence..=end_sequence {
        if let Some(entry) = env.storage().persistent().get::<DataKey, AuditEntry>(&DataKey::AuditLog(seq)) {
            // Filter by record_id if specified
            let should_include = if let Some(rid) = &record_id {
                entry.has_record_id && entry.record_id == *rid
            } else {
                true
            };
            
            if should_include {
                logs.push_back(entry);
            }
        }
    }
    
    logs
}

/// Check if caller can access audit logs for a specific record
fn can_access_audit_logs(env: &Env, caller: &Address, record_id: &BytesN<32>) -> bool {
    // Get the record
    let record: Option<MedicalRecord> = env.storage().persistent()
        .get(&DataKey::Record(record_id.clone()));
    
    if let Some(rec) = record {
        // Patient can always access their own audit logs
        if rec.patient == *caller {
            return true;
        }
        
        // Check if caller has audit access permission
        let access_grants: Map<Address, AccessGrant> = env.storage().persistent()
            .get(&DataKey::RecordAccess(record_id.clone()))
            .unwrap_or(Map::new(env));
        
        if let Some(grant) = access_grants.get(caller.clone()) {
            // Check if access is valid and includes audit permissions
            if !grant.revoked && 
               (grant.expires_at.is_none() || grant.expires_at.unwrap() > env.ledger().timestamp()) {
                match grant.access_level {
                    crate::AccessLevel::Audit => return true,
                    _ => return false,
                }
            }
        }
    }
    
    false
}

/// Generate detailed event description
fn generate_event_details(
    env: &Env,
    event_type: &AuditEventType,
    _record_id: &Option<BytesN<32>>,
    _target: &Option<Address>,
) -> String {
    match event_type {
        AuditEventType::RecordCreated => {
            String::from_str(env, "Medical record created")
        },
        AuditEventType::RecordUpdated => {
            String::from_str(env, "Medical record updated")
        },
        AuditEventType::RecordAccessed => {
            String::from_str(env, "Medical record accessed")
        },
        AuditEventType::AccessGranted => {
            String::from_str(env, "Access granted")
        },
        AuditEventType::AccessRevoked => {
            String::from_str(env, "Access revoked")
        },
        AuditEventType::AccessDenied => {
            String::from_str(env, "Access denied")
        },
        AuditEventType::EmergencyAccess => {
            String::from_str(env, "Emergency access performed")
        },
        AuditEventType::ProviderRegistered => {
            String::from_str(env, "Healthcare provider registered")
        },
        AuditEventType::SystemInitialized => {
            String::from_str(env, "Medical records system initialized")
        },
        AuditEventType::ConsentVerified => {
            String::from_str(env, "Patient consent verified")
        },
        AuditEventType::ConsentDenied => {
            String::from_str(env, "Patient consent denied")
        },
        AuditEventType::BulkAccessGranted => {
            String::from_str(env, "Bulk access granted")
        },
    }
}

/// Verify audit log integrity (simplified version)
pub fn verify_audit_integrity(env: &Env, from_sequence: u64, to_sequence: u64) -> bool {
    if to_sequence < from_sequence {
        return false;
    }
    
    let mut previous_timestamp = 0u64;
    
    for seq in from_sequence..=to_sequence {
        if let Some(entry) = env.storage().persistent().get::<DataKey, AuditEntry>(&DataKey::AuditLog(seq)) {
            // Check sequence continuity
            if entry.sequence != seq {
                return false;
            }
            
            // Check timestamp ordering (should be non-decreasing)
            if entry.timestamp < previous_timestamp {
                return false;
            }
            
            previous_timestamp = entry.timestamp;
        } else {
            return false; // Missing entry
        }
    }
    
    true
}

/// Get audit statistics
pub fn get_audit_statistics(env: &Env, caller: Address) -> Option<AuditStatistics> {
    // Only admin can access system statistics
    let admin: Address = env.storage().persistent()
        .get(&DataKey::Admin)?;
    
    if caller != admin {
        return None;
    }
    
    let total_entries: u64 = env.storage().persistent()
        .get(&DataKey::AuditCount)
        .unwrap_or(0);
    
    // For a real implementation, you'd calculate more detailed statistics
    Some(AuditStatistics {
        total_entries,
        entries_last_24h: 0, // Would require scanning recent entries
        unique_actors: 0,    // Would require scanning all entries
        most_accessed_records: Vec::new(env),
    })
}

/// Audit statistics structure
#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditStatistics {
    pub total_entries: u64,
    pub entries_last_24h: u64,
    pub unique_actors: u32,
    pub most_accessed_records: Vec<BytesN<32>>,
}