#![no_std]

mod verifier;
mod reporting;
mod utils;

#[cfg(test)]
mod test;
mod tests;


use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, Address, Env, Map, String, Vec, Bytes,
};

pub use verifier::*;
pub use reporting::*;
pub use utils::*;

#[contracttype]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum DataKey {
    Initialized = 0,
    Admin = 1,
    Verifiers = 2,
    ConsumptionReports = 3,
    VerificationStatus = 4,
    AuditLog = 5,
    MetersRegistry = 6,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumptionRecord {
    pub consumer: Address,
    pub meter_id: String,
    pub consumption_kwh: u64,
    pub timestamp: u64,
    pub meter_reading: u64,
    pub temperature: Option<i32>,
    pub voltage: Option<u32>,
    pub data_hash: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VerificationRecord {
    pub record_id: u64,
    pub verifier: Address,
    pub status: VerificationStatus,
    pub timestamp: u64,
    pub comments: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum VerificationStatus {
    Pending = 0,
    Verified = 1,
    Rejected = 2,
    Flagged = 3,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AuditLogEntry {
    pub id: u64,
    pub record_id: u64,
    pub action: String,
    pub actor: Address,
    pub timestamp: u64,
    pub details: Option<String>,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    InvalidInput = 3,
    RecordNotFound = 4,
    AlreadyVerified = 5,
    InvalidMeterData = 6,
    VerifierNotRegistered = 7,
    ConsumerNotRegistered = 8,
    NotInitialized = 9,
    InvalidTimestamp = 10,
    DataIntegrityError = 11,
}

#[contract]
pub struct EnergyConsumptionVerifier;

#[contractimpl]
impl EnergyConsumptionVerifier {
    /// Initialize the contract with admin and initial verifiers
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);

        let mut verifiers = Map::new(&env);
        verifiers.set(admin.clone(), true);
        env.storage().instance().set(&DataKey::Verifiers, &verifiers);

        let consumption_reports: Map<u64, ConsumptionRecord> = Map::new(&env);
        env.storage().instance().set(&DataKey::ConsumptionReports, &consumption_reports);

        let verification_status: Map<u64, VerificationRecord> = Map::new(&env);
        env.storage().instance().set(&DataKey::VerificationStatus, &verification_status);

        let audit_log: Vec<AuditLogEntry> = Vec::new(&env);
        env.storage().instance().set(&DataKey::AuditLog, &audit_log);

        let meters_registry: Map<String, Address> = Map::new(&env);
        env.storage().instance().set(&DataKey::MetersRegistry, &meters_registry);

        Ok(())
    }

    /// Submit energy consumption data for verification
    pub fn submit_data(
        env: Env,
        consumer: Address,
        meter_id: String,
        consumption_kwh: u64,
        meter_reading: u64,
        temperature: Option<i32>,
        voltage: Option<u32>,
    ) -> Result<u64, ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::NotInitialized);
        }

        consumer.require_auth();

        verifier::validate_consumption_data(&env, consumption_kwh, meter_reading, temperature, voltage)?;

        let record_id = utils::generate_record_id(&env);
        let timestamp = env.ledger().timestamp();

        let data_hash = utils::compute_data_hash(&env, &consumer, &meter_id, consumption_kwh, meter_reading, timestamp);

        let record = ConsumptionRecord {
            consumer: consumer.clone(),
            meter_id: meter_id.clone(),
            consumption_kwh,
            timestamp,
            meter_reading,
            temperature,
            voltage,
            data_hash,
        };

        reporting::store_consumption_record(&env, record_id, record)?;

        let verification_record = VerificationRecord {
            record_id,
            verifier: consumer.clone(),
            status: VerificationStatus::Pending,
            timestamp,
            comments: None,
        };

        reporting::store_verification_record(&env, record_id, verification_record)?;

        utils::log_audit_event(&env, record_id, String::from_str(&env, "DATA_SUBMITTED"), consumer, Some(String::from_str(&env, "Consumption data submitted for verification")))?;

        Ok(record_id)
    }

    /// Verify consumption data (verifiers only)
    pub fn verify_data(
        env: Env,
        verifier: Address,
        record_id: u64,
        status: VerificationStatus,
        comments: Option<String>,
    ) -> Result<(), ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::NotInitialized);
        }

        verifier.require_auth();

        if !verifier::is_authorized_verifier(&env, &verifier) {
            return Err(ContractError::VerifierNotRegistered);
        }

        let consumption_record = reporting::get_consumption_record(&env, record_id)?;

        verifier::verify_data_integrity(&env, &consumption_record)?;

        let timestamp = env.ledger().timestamp();
        let verification_record = VerificationRecord {
            record_id,
            verifier: verifier.clone(),
            status: status.clone(),
            timestamp,
            comments: comments.clone(),
        };

        reporting::update_verification_status(&env, record_id, verification_record)?;

        let action = match status {
            VerificationStatus::Verified => String::from_str(&env, "DATA_VERIFIED"),
            VerificationStatus::Rejected => String::from_str(&env, "DATA_REJECTED"),
            VerificationStatus::Flagged => String::from_str(&env, "DATA_FLAGGED"),
            _ => String::from_str(&env, "DATA_REVIEWED"),
        };

        utils::log_audit_event(&env, record_id, action, verifier, comments)?;

        Ok(())
    }

    /// Query verification status of consumption data
    pub fn get_verification(env: Env, record_id: u64) -> Result<VerificationRecord, ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::NotInitialized);
        }

        reporting::get_verification_record(&env, record_id)
    }

    /// Retrieve audit log for verification events
    pub fn audit_log(env: Env, offset: u32, limit: u32) -> Result<Vec<AuditLogEntry>, ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::NotInitialized);
        }

        utils::get_audit_log(&env, offset, limit)
    }

    /// Register a new verifier (admin only)
    pub fn register_verifier(env: Env, admin: Address, verifier: Address) -> Result<(), ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::NotInitialized);
        }

        admin.require_auth();

        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(ContractError::NotAuthorized);
        }

        let mut verifiers: Map<Address, bool> = env.storage().instance().get(&DataKey::Verifiers).unwrap();
        verifiers.set(verifier.clone(), true);
        env.storage().instance().set(&DataKey::Verifiers, &verifiers);

        utils::log_audit_event(&env, 0, String::from_str(&env, "VERIFIER_REGISTERED"), admin, Some(String::from_str(&env, "New verifier registered")))?;

        Ok(())
    }

    /// Register a smart meter (admin only)
    pub fn register_meter(env: Env, admin: Address, meter_id: String, meter_address: Address) -> Result<(), ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::NotInitialized);
        }

        admin.require_auth();

        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(ContractError::NotAuthorized);
        }

        let mut meters: Map<String, Address> = env.storage().instance().get(&DataKey::MetersRegistry).unwrap();
        meters.set(meter_id.clone(), meter_address);
        env.storage().instance().set(&DataKey::MetersRegistry, &meters);

        utils::log_audit_event(&env, 0, String::from_str(&env, "METER_REGISTERED"), admin, Some(String::from_str(&env, "New meter registered")))?;

        Ok(())
    }

    /// Get consumption record by ID
    pub fn get_consumption_record(env: Env, record_id: u64) -> Result<ConsumptionRecord, ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::NotInitialized);
        }

        reporting::get_consumption_record(&env, record_id)
    }

    /// Get consumption records by consumer
    pub fn get_consumer_records(
        env: Env,
        consumer: Address,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<u64>, ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::NotInitialized);
        }

        reporting::get_consumer_records(&env, consumer, offset, limit)
    }

    /// Get records by verification status
    pub fn get_records_by_status(
        env: Env,
        status: VerificationStatus,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<u64>, ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::NotInitialized);
        }

        reporting::get_records_by_status(&env, status, offset, limit)
    }
}