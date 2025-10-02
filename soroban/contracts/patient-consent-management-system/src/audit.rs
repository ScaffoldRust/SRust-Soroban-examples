use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

use crate::consent::{ConsentStatus, DataScope};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditEventType {
    ConsentCreated,
    ConsentUpdated,
    ConsentRevoked,
    ConsentSuspended,
    ConsentResumed,
    ConsentAccessed,
    ConsentExpired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEvent {
    pub event_id: u64,
    pub consent_id: u64,
    pub event_type: AuditEventType,
    pub actor: Address,
    pub timestamp: u64,
    pub details: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsentAccessLog {
    pub consent_id: u64,
    pub accessed_by: Address,
    pub data_scope: DataScope,
    pub timestamp: u64,
    pub purpose: String,
}

// Storage keys
const AUDIT_EVENTS: Symbol = symbol_short!("AUD_EVTS");
const CONSENT_AUDIT: Symbol = symbol_short!("CONS_AUD");
const ACCESS_LOGS: Symbol = symbol_short!("ACC_LOGS");
const CONSENT_ACCESS_LOGS: Symbol = symbol_short!("CNS_ALGS");
const NEXT_EVENT_ID: Symbol = symbol_short!("NEXT_EID");
const NEXT_ACCESS_LOG_ID: Symbol = symbol_short!("NEXT_ALG");

pub fn log_consent_created(
    env: &Env,
    consent_id: u64,
    patient: Address,
) {
    log_audit_event(
        env,
        consent_id,
        AuditEventType::ConsentCreated,
        patient,
        None,
    );
}

pub fn log_consent_updated(
    env: &Env,
    consent_id: u64,
    patient: Address,
    details: Option<String>,
) {
    log_audit_event(
        env,
        consent_id,
        AuditEventType::ConsentUpdated,
        patient,
        details,
    );
}

pub fn log_consent_revoked(
    env: &Env,
    consent_id: u64,
    patient: Address,
) {
    log_audit_event(
        env,
        consent_id,
        AuditEventType::ConsentRevoked,
        patient,
        None,
    );
}

pub fn log_consent_suspended(
    env: &Env,
    consent_id: u64,
    patient: Address,
) {
    log_audit_event(
        env,
        consent_id,
        AuditEventType::ConsentSuspended,
        patient,
        None,
    );
}

pub fn log_consent_resumed(
    env: &Env,
    consent_id: u64,
    patient: Address,
) {
    log_audit_event(
        env,
        consent_id,
        AuditEventType::ConsentResumed,
        patient,
        None,
    );
}

pub fn log_consent_accessed(
    env: &Env,
    consent_id: u64,
    accessed_by: Address,
    data_scope: DataScope,
    purpose: String,
) {
    // Log audit event
    log_audit_event(
        env,
        consent_id,
        AuditEventType::ConsentAccessed,
        accessed_by.clone(),
        Some(purpose.clone()),
    );

    // Create access log entry
    let access_log_id = get_next_access_log_id(env);

    let access_log = ConsentAccessLog {
        consent_id,
        accessed_by,
        data_scope,
        timestamp: env.ledger().timestamp(),
        purpose,
    };

    // Store access log
    env.storage()
        .persistent()
        .set(&storage_key_access_log(access_log_id), &access_log);

    // Add to consent's access logs
    add_consent_access_log(env, consent_id, access_log_id);
}

pub fn log_consent_expired(
    env: &Env,
    consent_id: u64,
    patient: Address,
) {
    log_audit_event(
        env,
        consent_id,
        AuditEventType::ConsentExpired,
        patient,
        None,
    );
}

pub fn get_audit_log(env: &Env, consent_id: u64) -> Vec<AuditEvent> {
    let event_ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&storage_key_consent_audit(consent_id))
        .unwrap_or_else(|| Vec::new(&env));

    let mut events = Vec::new(&env);
    for i in 0..event_ids.len() {
        let event_id = event_ids.get(i).unwrap();
        if let Some(event) = get_audit_event(env, event_id) {
            events.push_back(event);
        }
    }

    events
}

pub fn get_access_logs(env: &Env, consent_id: u64) -> Vec<ConsentAccessLog> {
    let access_log_ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&storage_key_consent_access_logs(consent_id))
        .unwrap_or_else(|| Vec::new(&env));

    let mut logs = Vec::new(&env);
    for i in 0..access_log_ids.len() {
        let log_id = access_log_ids.get(i).unwrap();
        if let Some(log) = get_access_log(env, log_id) {
            logs.push_back(log);
        }
    }

    logs
}

pub fn get_patient_audit_summary(env: &Env, patient: Address, consent_id: u64) -> Option<AuditSummary> {
    let consent = crate::consent::get_consent(env, consent_id)?;

    // Verify patient owns this consent
    if consent.patient != patient {
        return None;
    }

    let audit_events = get_audit_log(env, consent_id);
    let access_logs = get_access_logs(env, consent_id);

    Some(AuditSummary {
        consent_id,
        total_events: audit_events.len(),
        total_accesses: access_logs.len(),
        created_at: consent.created_at,
        last_updated: consent.last_updated,
        status: consent.status,
    })
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditSummary {
    pub consent_id: u64,
    pub total_events: u32,
    pub total_accesses: u32,
    pub created_at: u64,
    pub last_updated: u64,
    pub status: ConsentStatus,
}

fn log_audit_event(
    env: &Env,
    consent_id: u64,
    event_type: AuditEventType,
    actor: Address,
    details: Option<String>,
) {
    let event_id = get_next_event_id(env);

    let event = AuditEvent {
        event_id,
        consent_id,
        event_type,
        actor,
        timestamp: env.ledger().timestamp(),
        details,
    };

    // Store event
    env.storage()
        .persistent()
        .set(&storage_key_audit_event(event_id), &event);

    // Add to consent's audit trail
    add_consent_audit_event(env, consent_id, event_id);
}

fn get_audit_event(env: &Env, event_id: u64) -> Option<AuditEvent> {
    env.storage()
        .persistent()
        .get(&storage_key_audit_event(event_id))
}

fn get_access_log(env: &Env, log_id: u64) -> Option<ConsentAccessLog> {
    env.storage()
        .persistent()
        .get(&storage_key_access_log(log_id))
}

fn get_next_event_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&NEXT_EVENT_ID)
        .unwrap_or(1);

    env.storage().instance().set(&NEXT_EVENT_ID, &(current_id + 1));

    current_id
}

fn get_next_access_log_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&NEXT_ACCESS_LOG_ID)
        .unwrap_or(1);

    env.storage().instance().set(&NEXT_ACCESS_LOG_ID, &(current_id + 1));

    current_id
}

fn add_consent_audit_event(env: &Env, consent_id: u64, event_id: u64) {
    let mut event_ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&storage_key_consent_audit(consent_id))
        .unwrap_or_else(|| Vec::new(&env));

    event_ids.push_back(event_id);

    env.storage()
        .persistent()
        .set(&storage_key_consent_audit(consent_id), &event_ids);
}

fn add_consent_access_log(env: &Env, consent_id: u64, log_id: u64) {
    let mut log_ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&storage_key_consent_access_logs(consent_id))
        .unwrap_or_else(|| Vec::new(&env));

    log_ids.push_back(log_id);

    env.storage()
        .persistent()
        .set(&storage_key_consent_access_logs(consent_id), &log_ids);
}

fn storage_key_audit_event(event_id: u64) -> (Symbol, u64) {
    (AUDIT_EVENTS, event_id)
}

fn storage_key_consent_audit(consent_id: u64) -> (Symbol, u64) {
    (CONSENT_AUDIT, consent_id)
}

fn storage_key_access_log(log_id: u64) -> (Symbol, u64) {
    (ACCESS_LOGS, log_id)
}

fn storage_key_consent_access_logs(consent_id: u64) -> (Symbol, u64) {
    (CONSENT_ACCESS_LOGS, consent_id)
}
