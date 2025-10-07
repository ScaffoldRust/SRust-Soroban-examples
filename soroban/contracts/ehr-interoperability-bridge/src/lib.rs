#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Vec};

mod bridge;
mod interoperability;
mod utils;

#[cfg(test)]
mod test;

pub use bridge::*;
pub use interoperability::*;
pub use utils::*;

// Data structures for EHR system management
#[derive(Clone)]
#[contracttype]
pub struct EhrSystem {
    pub system_id: String,
    pub name: String,
    pub endpoint: String,
    pub supported_formats: Vec<String>,
    pub public_key: BytesN<32>,
    pub admin: Address,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct DataRequest {
    pub request_id: BytesN<32>,
    pub sender_system: String,
    pub receiver_system: String,
    pub patient_id: String,
    pub data_types: Vec<String>,
    pub requester: Address,
    pub consent_verified: bool,
    pub status: RequestStatus,
    pub timestamp: u64,
    pub expiry: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct DataTransfer {
    pub transfer_id: BytesN<32>,
    pub request_id: BytesN<32>,
    pub data_hash: BytesN<32>,
    pub source_format: String,
    pub target_format: String,
    pub transfer_timestamp: u64,
    pub validator: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct ConsentRecord {
    pub patient_id: String,
    pub patient_address: Address,
    pub authorized_systems: Vec<String>,
    pub data_types_permitted: Vec<String>,
    pub consent_expiry: u64,
    pub revoked: bool,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum RequestStatus {
    Pending,
    ConsentVerified,
    Approved,
    InProgress,
    Completed,
    Rejected,
    Expired,
}

#[derive(Clone)]
#[contracttype]
pub enum DataFormat {
    Hl7V2,
    Hl7Fhir,
    Cda,
    Json,
    Xml,
    Custom(String),
}

#[derive(Clone)]
#[contracttype]
pub enum AccessRole {
    Patient,
    Doctor,
    Nurse,
    Administrator,
    System,
}

// Storage keys
#[contracttype]
pub enum DataKey {
    Admin,
    EhrSystem(String),
    DataRequest(BytesN<32>),
    DataTransfer(BytesN<32>),
    ConsentRecord(String),
    RequestQueue,
    SystemRegistry,
    AuditLog(BytesN<32>),
    NextRequestId,
}

#[contract]
pub struct EhrInteroperabilityBridge;

#[contractimpl]
impl EhrInteroperabilityBridge {
    /// Initialize the EHR bridge with admin
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextRequestId, &1u64);
        
        // Initialize empty registries
        let empty_systems: Map<String, EhrSystem> = Map::new(&env);
        let empty_queue: Vec<BytesN<32>> = Vec::new(&env);
        
        env.storage().instance().set(&DataKey::SystemRegistry, &empty_systems);
        env.storage().instance().set(&DataKey::RequestQueue, &empty_queue);
    }

    /// Register a new EHR system
    pub fn register_ehr_system(
        env: Env,
        admin: Address,
        system_id: String,
        name: String,
        endpoint: String,
        supported_formats: Vec<String>,
        public_key: BytesN<32>,
        system_admin: Address,
    ) -> bool {
        bridge::register_ehr_system(
            &env,
            admin,
            system_id,
            name,
            endpoint,
            supported_formats,
            public_key,
            system_admin,
        )
    }

    /// Request data from another EHR system
    pub fn request_data(
        env: Env,
        requester: Address,
        sender_system: String,
        receiver_system: String,
        patient_id: String,
        data_types: Vec<String>,
        expiry_hours: u64,
    ) -> BytesN<32> {
        bridge::request_data(
            &env,
            requester,
            sender_system,
            receiver_system,
            patient_id,
            data_types,
            expiry_hours,
        )
    }

    /// Verify patient consent for data request
    pub fn verify_consent(
        env: Env,
        patient: Address,
        request_id: BytesN<32>,
        consent_signature: BytesN<64>,
    ) -> bool {
        interoperability::verify_consent(&env, patient, request_id, consent_signature)
    }

    /// Transfer data after consent and validation
    pub fn transfer_data(
        env: Env,
        request_id: BytesN<32>,
        data_hash: BytesN<32>,
        source_format: String,
        target_format: String,
        validator: Address,
    ) -> BytesN<32> {
        bridge::transfer_data(
            &env,
            request_id,
            data_hash,
            source_format,
            target_format,
            validator,
        )
    }

    /// Validate and convert data format
    pub fn validate_format(
        env: Env,
        data_hash: BytesN<32>,
        source_format: String,
        target_format: String,
    ) -> bool {
        utils::validate_format(&env, data_hash, source_format, target_format)
    }

    /// Log successful data exchange for auditing
    pub fn log_exchange(
        env: Env,
        transfer_id: BytesN<32>,
        additional_metadata: String,
    ) {
        bridge::log_exchange(&env, transfer_id, additional_metadata)
    }

    /// Get EHR system information
    pub fn get_ehr_system(env: Env, system_id: String) -> Option<EhrSystem> {
        bridge::get_ehr_system(&env, system_id)
    }

    /// Get data request details
    pub fn get_data_request(env: Env, request_id: BytesN<32>) -> Option<DataRequest> {
        bridge::get_data_request(&env, request_id)
    }

    /// Get data transfer details
    pub fn get_data_transfer(env: Env, transfer_id: BytesN<32>) -> Option<DataTransfer> {
        bridge::get_data_transfer(&env, transfer_id)
    }

    /// Update request status
    pub fn update_request_status(
        env: Env,
        requester: Address,
        request_id: BytesN<32>,
        new_status: RequestStatus,
    ) -> bool {
        bridge::update_request_status(&env, requester, request_id, new_status)
    }

    /// Get pending requests for a system
    pub fn get_pending_requests(env: Env, system_id: String) -> Vec<BytesN<32>> {
        bridge::get_pending_requests(&env, system_id)
    }

    /// Set patient consent
    pub fn set_patient_consent(
        env: Env,
        patient: Address,
        patient_id: String,
        authorized_systems: Vec<String>,
        data_types_permitted: Vec<String>,
        consent_duration_hours: u64,
    ) -> bool {
        interoperability::set_patient_consent(
            &env,
            patient,
            patient_id,
            authorized_systems,
            data_types_permitted,
            consent_duration_hours,
        )
    }

    /// Revoke patient consent
    pub fn revoke_consent(
        env: Env,
        patient: Address,
        patient_id: String,
    ) -> bool {
        interoperability::revoke_consent(&env, patient, patient_id)
    }

    /// Check if systems are compatible
    pub fn check_compatibility(
        env: Env,
        source_system: String,
        target_system: String,
        data_type: String,
    ) -> bool {
        utils::check_compatibility(&env, source_system, target_system, data_type)
    }
}