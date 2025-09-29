#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, String, Vec,
};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EhrInteroperabilityBridge);
    let client = EhrInteroperabilityBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // Mock the authorization
    env.mock_all_auths();

    client.initialize(&admin);

    // Verify initialization
    let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    assert_eq!(stored_admin, admin);
}

#[test]
fn test_register_ehr_system() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EhrInteroperabilityBridge);
    let client = EhrInteroperabilityBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let system_admin = Address::generate(&env);

    env.mock_all_auths();

    // Initialize contract
    client.initialize(&admin);

    // Register EHR system
    let system_id = String::from_str(&env, "hospital_a");
    let name = String::from_str(&env, "Hospital A");
    let endpoint = String::from_str(&env, "https://hospital-a.com/fhir");
    let mut supported_formats = Vec::new(&env);
    supported_formats.push_back(String::from_str(&env, "HL7_FHIR_R4"));
    supported_formats.push_back(String::from_str(&env, "JSON"));
    
    let public_key = env.crypto().sha256(b"hospital_a_public_key");

    let result = client.register_ehr_system(
        &admin,
        &system_id,
        &name,
        &endpoint,
        &supported_formats,
        &public_key,
        &system_admin,
    );

    assert!(result);

    // Verify system was registered
    let stored_system = client.get_ehr_system(&system_id);
    assert!(stored_system.is_some());
    
    let system = stored_system.unwrap();
    assert_eq!(system.system_id, system_id);
    assert_eq!(system.name, name);
    assert_eq!(system.is_active, true);
}

#[test]
fn test_request_data() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EhrInteroperabilityBridge);
    let client = EhrInteroperabilityBridgeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let requester = Address::generate(&env);
    let system_admin_a = Address::generate(&env);
    let system_admin_b = Address::generate(&env);

    env.mock_all_auths();

    // Initialize and register systems
    client.initialize(&admin);

    let system_a = String::from_str(&env, "hospital_a");
    let system_b = String::from_str(&env, "hospital_b");
    let mut formats = Vec::new(&env);
    formats.push_back(String::from_str(&env, "HL7_FHIR_R4"));
    
    let public_key_a = env.crypto().sha256(b"hospital_a_key");
    let public_key_b = env.crypto().sha256(b"hospital_b_key");

    client.register_ehr_system(
        &admin,
        &system_a,
        &String::from_str(&env, "Hospital A"),
        &String::from_str(&env, "https://hospital-a.com/fhir"),
        &formats,
        &public_key_a,
        &system_admin_a,
    );

    client.register_ehr_system(
        &admin,
        &system_b,
        &String::from_str(&env, "Hospital B"),
        &String::from_str(&env, "https://hospital-b.com/fhir"),
        &formats,
        &public_key_b,
        &system_admin_b,
    );

    // Request data
    let patient_id = String::from_str(&env, "patient_123");
    let mut data_types = Vec::new(&env);
    data_types.push_back(String::from_str(&env, "Patient"));
    data_types.push_back(String::from_str(&env, "Observation"));

    let request_id = client.request_data(
        &requester,
        &system_a,
        &system_b,
        &patient_id,
        &data_types,
        &24, // 24 hours expiry
    );

    // Verify request was created
    let request = client.get_data_request(&request_id);
    assert!(request.is_some());
    
    let req = request.unwrap();
    assert_eq!(req.sender_system, system_a);
    assert_eq!(req.receiver_system, system_b);
    assert_eq!(req.patient_id, patient_id);
    assert_eq!(req.requester, requester);
    assert_eq!(req.consent_verified, false);
}

// Mock client for testing
soroban_sdk::contractclient! {
    pub struct EhrInteroperabilityBridgeClient;
}