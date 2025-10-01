extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String, Vec};
use crate::{SecureMedicalRecordsSharing, SecureMedicalRecordsSharingClient};

/// Test fixture to hold common test setup
pub struct TestFixture<'a> {
    pub env: Env,
    pub client: SecureMedicalRecordsSharingClient<'a>,
    pub patient: Address,
    pub provider: Address,
    pub provider2: Address,
    pub unauthorized: Address,
}

impl<'a> TestFixture<'a> {
    /// Create a new test fixture with initialized contract
    pub fn new() -> Self {
        let env = Env::default();
        let contract_id = env.register(SecureMedicalRecordsSharing, ());
        let client = SecureMedicalRecordsSharingClient::new(&env, &contract_id);

        client.initialize();

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let provider2 = Address::generate(&env);
        let unauthorized = Address::generate(&env);

        Self {
            env,
            client,
            patient,
            provider,
            provider2,
            unauthorized,
        }
    }

    /// Helper to create a medical record
    pub fn create_record(&self, data_type: &str, pointer: &str) -> u64 {
        let data_type_str = String::from_str(&self.env, data_type);
        let pointer_str = String::from_str(&self.env, pointer);
        self.client.add_record(&self.patient, &data_type_str, &pointer_str)
    }

    /// Helper to create a medical record with custom patient
    pub fn create_record_for(&self, patient: &Address, data_type: &str, pointer: &str) -> u64 {
        let data_type_str = String::from_str(&self.env, data_type);
        let pointer_str = String::from_str(&self.env, pointer);
        self.client.add_record(patient, &data_type_str, &pointer_str)
    }

    /// Helper to grant access to a provider
    pub fn grant_access(&self, data_types: std::vec::Vec<&str>, expires_at: u64) {
        let types = self.str_vec_to_string_vec(data_types);
        self.client.grant_access(&self.patient, &self.provider, &types, &expires_at);
    }

    /// Helper to grant access to a specific provider
    pub fn grant_access_to(&self, provider: &Address, data_types: std::vec::Vec<&str>, expires_at: u64) {
        let types = self.str_vec_to_string_vec(data_types);
        self.client.grant_access(&self.patient, provider, &types, &expires_at);
    }

    /// Helper to grant access from specific patient to specific provider
    pub fn grant_access_from_to(&self, patient: &Address, provider: &Address, data_types: std::vec::Vec<&str>, expires_at: u64) {
        let types = self.str_vec_to_string_vec(data_types);
        self.client.grant_access(patient, provider, &types, &expires_at);
    }

    /// Helper to verify access
    pub fn verify_access(&self, data_type: &str) -> bool {
        let data_type_str = String::from_str(&self.env, data_type);
        self.client.verify_access(&self.patient, &self.provider, &data_type_str)
    }

    /// Helper to get a record
    pub fn get_record(&self, record_id: u64) -> crate::MedicalRecord {
        self.client.get_record(&self.patient, &self.patient, &record_id)
    }

    /// Helper to get a record as a provider
    pub fn get_record_as_provider(&self, record_id: u64) -> crate::MedicalRecord {
        self.client.get_record(&self.provider, &self.patient, &record_id)
    }

    /// Helper to convert Vec<&str> to Vec<String>
    pub fn str_vec_to_string_vec(&self, strs: std::vec::Vec<&str>) -> Vec<String> {
        let mut result = Vec::new(&self.env);
        for s in strs {
            result.push_back(String::from_str(&self.env, s));
        }
        result
    }

    /// Helper to create a string
    pub fn string(&self, s: &str) -> String {
        String::from_str(&self.env, s)
    }

    /// Get current timestamp
    pub fn now(&self) -> u64 {
        self.env.ledger().timestamp()
    }

    /// Advance time by seconds
    pub fn advance_time(&self, seconds: u64) {
        self.env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp + seconds;
        });
    }

    /// Helper to add emergency provider
    pub fn add_emergency_provider(&self) {
        self.client.add_emergency_provider(&self.patient, &self.provider);
    }

    /// Helper to get audit log
    pub fn get_audit_log(&self) -> Vec<crate::AuditEvent> {
        self.client.get_audit_log(&self.patient, &self.patient)
    }
}
