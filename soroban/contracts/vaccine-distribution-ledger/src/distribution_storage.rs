use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchStatus {
    Produced,
    InTransit,
    Distributed,
    Administered,
    Expired,
    Recalled,
    ColdChainBreach,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Production,
    Distribution,
    Administration,
    StatusUpdate,
    ColdChainBreach,
    QualityCheck,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaccineBatch {
    pub batch_id: String,
    pub manufacturer: Address,
    pub vaccine_type: String,
    pub production_date: u64,
    pub expiry_date: u64,
    pub initial_quantity: u32,
    pub current_quantity: u32,
    pub status: BatchStatus,
    pub created_at: u64,
    pub last_updated: u64,
    pub notes: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistributionEvent {
    pub event_id: u64,
    pub batch_id: String,
    pub event_type: EventType,
    pub actor: Address,
    pub timestamp: u64,
    pub quantity: u32,
    pub destination: Option<String>,
    pub temperature_log: Option<String>,
    pub notes: Option<String>,
    pub location: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdministrationRecord {
    pub record_id: u64,
    pub batch_id: String,
    pub administrator: Address,
    pub patient_id: String,
    pub administered_quantity: u32,
    pub location: String,
    pub timestamp: u64,
    pub verified: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColdChainAlert {
    pub alert_id: u64,
    pub batch_id: String,
    pub reporter: Address,
    pub severity: String,
    pub description: String,
    pub timestamp: u64,
    pub resolved: bool,
}

// Storage key types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Batch(String),                           // batch_id
    BatchHistory(String),                    // batch_id -> Vec<event_id>
    DistributionEvent(u64),                  // event_id
    AdministrationRecord(u64),               // record_id
    ColdChainAlert(u64),                     // alert_id
    ManufacturerBatches(Address),            // manufacturer -> Vec<batch_id>
    BatchesByStatus(BatchStatus),            // status -> Vec<batch_id>
    BatchAdministrations(String),            // batch_id -> Vec<record_id>
    EventCounter,                            // For generating unique event IDs
    RecordCounter,                           // For generating unique record IDs
    AlertCounter,                            // For generating unique alert IDs
}

// Storage key constants
const EVENT_COUNTER: Symbol = symbol_short!("EVT_CNT");
const RECORD_COUNTER: Symbol = symbol_short!("REC_CNT");
const ALERT_COUNTER: Symbol = symbol_short!("ALR_CNT");

// Counter generation functions
pub fn get_next_event_id(env: &Env) -> u64 {
    let current = env.storage().instance().get(&EVENT_COUNTER).unwrap_or(0u64);
    let next = current + 1;
    env.storage().instance().set(&EVENT_COUNTER, &next);
    next
}

pub fn get_next_record_id(env: &Env) -> u64 {
    let current = env.storage().instance().get(&RECORD_COUNTER).unwrap_or(0u64);
    let next = current + 1;
    env.storage().instance().set(&RECORD_COUNTER, &next);
    next
}

pub fn get_next_alert_id(env: &Env) -> u64 {
    let current = env.storage().instance().get(&ALERT_COUNTER).unwrap_or(0u64);
    let next = current + 1;
    env.storage().instance().set(&ALERT_COUNTER, &next);
    next
}

// Batch storage functions
pub fn get_batch(env: &Env, batch_id: &String) -> Option<VaccineBatch> {
    let key = StorageKey::Batch(batch_id.clone());
    env.storage().persistent().get(&key)
}

pub fn set_batch(env: &Env, batch: &VaccineBatch) {
    let key = StorageKey::Batch(batch.batch_id.clone());
    env.storage().persistent().set(&key, batch);
}

// Distribution event storage functions
pub fn get_distribution_event(env: &Env, event_id: u64) -> Option<DistributionEvent> {
    let key = StorageKey::DistributionEvent(event_id);
    env.storage().persistent().get(&key)
}

pub fn set_distribution_event(env: &Env, event: &DistributionEvent) {
    let key = StorageKey::DistributionEvent(event.event_id);
    env.storage().persistent().set(&key, event);
}

// Administration record storage functions
pub fn get_administration_record(env: &Env, record_id: u64) -> Option<AdministrationRecord> {
    let key = StorageKey::AdministrationRecord(record_id);
    env.storage().persistent().get(&key)
}

pub fn set_administration_record(env: &Env, record: &AdministrationRecord) {
    let key = StorageKey::AdministrationRecord(record.record_id);
    env.storage().persistent().set(&key, record);
}

// Cold chain alert storage functions
pub fn _get_cold_chain_alert(env: &Env, alert_id: u64) -> Option<ColdChainAlert> {
    let key = StorageKey::ColdChainAlert(alert_id);
    env.storage().persistent().get(&key)
}

pub fn set_cold_chain_alert(env: &Env, alert: &ColdChainAlert) {
    let key = StorageKey::ColdChainAlert(alert.alert_id);
    env.storage().persistent().set(&key, alert);
}

// Batch history storage functions
pub fn get_batch_history_ids(env: &Env, batch_id: &String) -> Vec<u64> {
    let key = StorageKey::BatchHistory(batch_id.clone());
    env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
}

pub fn add_batch_event(env: &Env, batch_id: &String, event_id: u64) {
    let key = StorageKey::BatchHistory(batch_id.clone());
    let mut events = get_batch_history_ids(env, batch_id);
    events.push_back(event_id);
    env.storage().persistent().set(&key, &events);
}

// Manufacturer batches storage functions
pub fn get_manufacturer_batch_ids(env: &Env, manufacturer: &Address) -> Vec<String> {
    let key = StorageKey::ManufacturerBatches(manufacturer.clone());
    env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
}

pub fn add_manufacturer_batch(env: &Env, manufacturer: &Address, batch_id: &String) {
    let key = StorageKey::ManufacturerBatches(manufacturer.clone());
    let mut batches = get_manufacturer_batch_ids(env, manufacturer);
    batches.push_back(batch_id.clone());
    env.storage().persistent().set(&key, &batches);
}

// Batches by status storage functions
pub fn get_batches_by_status_ids(env: &Env, status: &BatchStatus) -> Vec<String> {
    let key = StorageKey::BatchesByStatus(status.clone());
    env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
}

pub fn add_batch_to_status(env: &Env, status: &BatchStatus, batch_id: &String) {
    let key = StorageKey::BatchesByStatus(status.clone());
    let mut batches = get_batches_by_status_ids(env, status);
    batches.push_back(batch_id.clone());
    env.storage().persistent().set(&key, &batches);
}

pub fn remove_batch_from_status(env: &Env, status: &BatchStatus, batch_id: &String) {
    let key = StorageKey::BatchesByStatus(status.clone());
    let batches = get_batches_by_status_ids(env, status);
    let mut new_batches = Vec::new(env);
    
    for batch in batches.iter() {
        if batch != *batch_id {
            new_batches.push_back(batch);
        }
    }
    
    env.storage().persistent().set(&key, &new_batches);
}

// Batch administration records storage functions
pub fn get_batch_administration_ids(env: &Env, batch_id: &String) -> Vec<u64> {
    let key = StorageKey::BatchAdministrations(batch_id.clone());
    env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
}

pub fn add_batch_administration(env: &Env, batch_id: &String, record_id: u64) {
    let key = StorageKey::BatchAdministrations(batch_id.clone());
    let mut records = get_batch_administration_ids(env, batch_id);
    records.push_back(record_id);
    env.storage().persistent().set(&key, &records);
}