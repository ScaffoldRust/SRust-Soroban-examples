use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
use crate::{PatientConsentManagementSystem, PatientConsentManagementSystemClient, consent::DataScope};

/// Helper function to create and initialize a test contract
pub fn create_test_contract(env: &Env) -> PatientConsentManagementSystemClient {
    let client = PatientConsentManagementSystemClient::new(
        env,
        &env.register(PatientConsentManagementSystem, ()),
    );
    client.initialize();
    client
}

/// Helper to create a vector of data scopes
pub fn create_data_scopes(env: &Env, scopes: &[DataScope]) -> Vec<DataScope> {
    let mut vec = Vec::new(env);
    for scope in scopes {
        vec.push_back(scope.clone());
    }
    vec
}

/// Helper to create a standard test purpose string
pub fn create_purpose(env: &Env, text: &str) -> String {
    String::from_str(env, text)
}

/// Helper to generate multiple addresses for testing
pub fn generate_addresses(env: &Env, count: usize) -> Vec<Address> {
    let mut addresses = Vec::new(env);
    for _ in 0..count {
        addresses.push_back(Address::generate(env));
    }
    addresses
}

/// Helper to create a consent with default parameters
pub fn create_default_consent(
    contract: &PatientConsentManagementSystemClient,
    env: &Env,
    patient: &Address,
    authorized_party: &Address,
) -> u64 {
    let scopes = create_data_scopes(env, &[DataScope::Treatment]);
    let purpose = create_purpose(env, "Standard medical treatment");

    contract.create_consent(patient, authorized_party, &scopes, &purpose, &None)
}

/// Helper to create a consent with expiration
pub fn create_expiring_consent(
    contract: &PatientConsentManagementSystemClient,
    env: &Env,
    patient: &Address,
    authorized_party: &Address,
    expires_in_seconds: u64,
) -> u64 {
    let scopes = create_data_scopes(env, &[DataScope::Treatment]);
    let purpose = create_purpose(env, "Temporary treatment consent");
    let expires_at = Some(env.ledger().timestamp() + expires_in_seconds);

    contract.create_consent(patient, authorized_party, &scopes, &purpose, &expires_at)
}

/// Helper to verify consent is active
pub fn assert_consent_active(
    contract: &PatientConsentManagementSystemClient,
    consent_id: u64,
) {
    assert!(contract.is_consent_active(&consent_id), "Consent should be active");
}

/// Helper to verify consent is not active
pub fn assert_consent_not_active(
    contract: &PatientConsentManagementSystemClient,
    consent_id: u64,
) {
    assert!(!contract.is_consent_active(&consent_id), "Consent should not be active");
}
