#![cfg(test)]

use crate::{
    distribution_storage::BatchStatus,
};
use soroban_sdk::{
    testutils::Address as _,
    Address, Env, String,
};

use super::utils::create_test_contract;

#[test]
fn test_vaccine_administration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let administrator = Address::generate(&env);

    contract.initialize(&admin);

    // Create and distribute batch
    let batch_id = String::from_str(&env, "JOHNSON_001");
    let vaccine_type = String::from_str(&env, "COVID-19 Viral Vector");

    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &vaccine_type,
        &1640995200u64,
        &500u32,
        &1672531200u64,
    );

    // Verify administration
    let patient_id = String::from_str(&env, "PATIENT_12345");
    let location = String::from_str(&env, "City Hospital");
    let admin_quantity = 1u32;

    contract.verify_administration(
        &batch_id,
        &administrator,
        &patient_id,
        &admin_quantity,
        &location,
    );

    // Check inventory decreased
    let remaining = contract.inventory_check(&batch_id);
    assert_eq!(remaining, 499);

    // Check batch status changed to Administered
    let batch = contract.get_batch(&batch_id);
    assert_eq!(batch.status, BatchStatus::Administered);
}

#[test]
fn test_cold_chain_breach_reporting() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let reporter = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch
    let batch_id = String::from_str(&env, "NOVAVAX_001");
    let vaccine_type = String::from_str(&env, "COVID-19 Protein Subunit");

    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &vaccine_type,
        &1640995200u64,
        &600u32,
        &1672531200u64,
    );

    // Report cold chain breach
    let severity = String::from_str(&env, "HIGH");
    let description = String::from_str(&env, "Temperature exceeded 8°C for 2 hours");

    contract.report_cold_chain_breach(
        &batch_id,
        &reporter,
        &severity,
        &description,
    );

    // High severity should change batch status
    let batch = contract.get_batch(&batch_id);
    assert_eq!(batch.status, BatchStatus::ColdChainBreach);
}

#[test]
fn test_duplicate_patient_administration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let administrator = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch
    let batch_id = String::from_str(&env, "DUPLICATE_TEST");
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &100u32,
        &1672531200u64,
    );

    // First administration
    let patient_id = String::from_str(&env, "DUPLICATE_PATIENT");
    
    contract.verify_administration(
        &batch_id,
        &administrator,
        &patient_id,
        &1u32,
        &String::from_str(&env, "Hospital"),
    );

    // Try duplicate administration to same patient
    let result = contract.try_verify_administration(
        &batch_id,
        &administrator,
        &patient_id, // Same patient
        &1u32,
        &String::from_str(&env, "Hospital"),
    );
    assert!(result.is_err());
}
