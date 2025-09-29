#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, String, Vec};

mod records;
mod access;
mod utils;

#[cfg(test)]
mod test;

pub use records::*;
pub use access::*;
pub use utils::*;

#[contract]
pub struct SecureMedicalRecordsSharing;

#[contracttype]
#[derive(Clone)]
pub struct ContractConfig { pub initialized: bool }

#[contracttype]
#[derive(Clone)]
pub struct MedicalRecord {
    pub record_id: u64,
    pub patient: Address,
    pub data_type: String,
    pub pointer: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct AccessGrant {
    pub provider: Address,
    pub data_types: Vec<String>,
    pub expires_at: u64,
    pub revoked: bool,
    pub granted_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub actor: Address,
    pub action: Symbol,
    pub record_id: Option<u64>,
    pub detail: Option<String>,
}

#[contracttype]
#[derive(Clone)]
pub struct EmergencyAccessUse { pub last_used: u64, pub uses: u32 }

#[contracttype]
pub enum DataKey {
    Config,
    RecordCounter(Address),
    Record(Address, u64),
    Access(Address, Address),
    Audit(Address),
    EmergencyWhitelist(Address, Address),
    EmergencyUse(Address, Address),
}

pub fn log_event(env: &Env, patient: &Address, action: Symbol, record_id: Option<u64>, detail: Option<String>) {
    let key = DataKey::Audit(patient.clone());
    let mut log: Vec<AuditEvent> = env.storage().instance().get(&key).unwrap_or(Vec::new(env));
    let evt = AuditEvent { timestamp: now(env), actor: patient.clone(), action: action.clone(), record_id, detail };
    if log.len() >= 200 { log.remove(0); }
    log.push_back(evt.clone());
    env.storage().instance().set(&key, &log);
    env.events().publish((Symbol::new(env, "audit"), action), record_id.unwrap_or(0u64));
}

#[contractimpl]
impl SecureMedicalRecordsSharing {
    pub fn initialize(env: Env) {
        if env.storage().instance().has(&DataKey::Config) { panic!("Already initialized"); }
        env.storage().instance().set(&DataKey::Config, &ContractConfig { initialized: true });
    }

    // Record operations
    pub fn add_record(env: Env, patient: Address, data_type: String, pointer: String) -> u64 { records::add_record(env, patient, data_type, pointer) }
    pub fn update_record(env: Env, patient: Address, record_id: u64, new_pointer: String) -> bool { records::update_record(env, patient, record_id, new_pointer) }
    pub fn get_record(env: Env, requester: Address, patient: Address, record_id: u64) -> MedicalRecord { records::get_record(env, requester, patient, record_id) }
    pub fn list_records(env: Env, patient: Address, owner: Address, offset: u32, limit: u32, data_type: Option<String>) -> Vec<MedicalRecord> { records::list_records(env, patient, owner, offset, limit, data_type) }

    // Access management
    pub fn grant_access(env: Env, patient: Address, provider: Address, data_types: Vec<String>, expires_at: u64) -> bool { access::grant_access(env, patient, provider, data_types, expires_at) }
    pub fn revoke_access(env: Env, patient: Address, provider: Address) -> bool { access::revoke_access(env, patient, provider) }
    pub fn verify_access(env: Env, patient: Address, provider: Address, data_type: String) -> bool { access::verify_access(&env, &patient, &provider, &data_type) }
    pub fn add_emergency_provider(env: Env, patient: Address, provider: Address) -> bool { access::add_emergency_provider(env, patient, provider) }
    pub fn remove_emergency_provider(env: Env, patient: Address, provider: Address) -> bool { access::remove_emergency_provider(env, patient, provider) }
    pub fn emergency_read(env: Env, provider: Address, patient: Address, record_id: u64, justification: String) -> MedicalRecord { access::emergency_read(env, provider, patient, record_id, justification) }
    pub fn get_audit_log(env: Env, patient: Address, owner: Address) -> Vec<AuditEvent> { access::get_audit_log(env, patient, owner) }
    pub fn get_access_grant(env: Env, patient: Address, provider: Address) -> Option<AccessGrant> { access::get_access_grant(env, patient, provider) }
}
