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
fn test_batch_history_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);
    let administrator = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch
    let batch_id = String::from_str(&env, "SPUTNIK_001");
    let vaccine_type = String::from_str(&env, "COVID-19 Viral Vector");

    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &vaccine_type,
        &1640995200u64,
        &1200u32,
        &1672531200u64,
    );

    // Log distribution
    contract.log_distribution(
        &batch_id,
        &distributor,
        &String::from_str(&env, "Hospital A"),
        &200u32,
        &None,
    );

    // Verify administration
    contract.verify_administration(
        &batch_id,
        &administrator,
        &String::from_str(&env, "PATIENT_001"),
        &1u32,
        &String::from_str(&env, "Hospital A"),
    );

    // Get batch history
    let history = contract.get_history(&batch_id, &0u32, &10u32);
    
    // Should have at least 3 events: production, distribution, administration
    assert!(history.len() >= 3);
}

#[test]
fn test_manufacturer_batch_queries() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);

    contract.initialize(&admin);

    // Create multiple batches for same manufacturer
    let batch_ids = ["BATCH_001", "BATCH_002", "BATCH_003"];
    
    for batch_id in &batch_ids {
        contract.initialize_batch(
            &String::from_str(&env, batch_id),
            &manufacturer,
            &String::from_str(&env, "COVID-19 mRNA"),
            &1640995200u64,
            &500u32,
            &1672531200u64,
        );
    }

    // Query manufacturer batches
    let batches = contract.get_manufacturer_batches(&manufacturer, &0u32, &10u32);
    assert_eq!(batches.len(), 3);
}

#[test]
fn test_batches_by_status_queries() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);

    contract.initialize(&admin);

    // Create batches
    let batch_id_1 = String::from_str(&env, "PRODUCED_001");
    let batch_id_2 = String::from_str(&env, "PRODUCED_002");
    
    contract.initialize_batch(
        &batch_id_1,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &300u32,
        &1672531200u64,
    );

    contract.initialize_batch(
        &batch_id_2,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &400u32,
        &1672531200u64,
    );

    // Query batches by Produced status
    let produced_batches = contract.get_batches_by_status(&BatchStatus::Produced, &0u32, &10u32);
    assert_eq!(produced_batches.len(), 2);
}

#[test]
fn test_administration_records_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let administrator = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch
    let batch_id = String::from_str(&env, "RECORD_TEST_001");
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &100u32,
        &1672531200u64,
    );

    // Multiple administrations
    let patients = ["PATIENT_A", "PATIENT_B", "PATIENT_C"];
    
    for patient in &patients {
        contract.verify_administration(
            &batch_id,
            &administrator,
            &String::from_str(&env, patient),
            &1u32,
            &String::from_str(&env, "Hospital"),
        );
    }

    // Get administration records
    let records = contract.get_administration_records(&batch_id, &0u32, &10u32);
    assert_eq!(records.len(), 3);
}

#[test]
fn test_complex_distribution_scenario() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let distributor1 = Address::generate(&env);
    let distributor2 = Address::generate(&env);
    let administrator1 = Address::generate(&env);
    let administrator2 = Address::generate(&env);

    contract.initialize(&admin);

    // Create large batch
    let batch_id = String::from_str(&env, "COMPLEX_SCENARIO");
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &2000u32,
        &1672531200u64,
    );

    // Multiple distributions
    contract.log_distribution(
        &batch_id,
        &distributor1,
        &String::from_str(&env, "Hospital A"),
        &500u32,
        &Some(String::from_str(&env, "2-8°C maintained")),
    );

    contract.log_distribution(
        &batch_id,
        &distributor2,
        &String::from_str(&env, "Hospital B"),
        &300u32,
        &Some(String::from_str(&env, "Cold chain intact")),
    );

    // Multiple administrations
    let patients = ["P001", "P002", "P003", "P004", "P005"];
    
    for (i, patient) in patients.iter().enumerate() {
        let admin_addr = if i % 2 == 0 { &administrator1 } else { &administrator2 };
        let location = if i % 2 == 0 { "Hospital A" } else { "Hospital B" };
        
        contract.verify_administration(
            &batch_id,
            admin_addr,
            &String::from_str(&env, patient),
            &1u32,
            &String::from_str(&env, location),
        );
    }

    // Check final inventory
    let remaining = contract.inventory_check(&batch_id);
    assert_eq!(remaining, 2000 - 500 - 300 - 5); // 1195

    // Verify complex history
    let history = contract.get_history(&batch_id, &0u32, &20u32);
    assert!(history.len() >= 8); // Production + 2 distributions + 5 administrations

    // Check administration records
    let records = contract.get_administration_records(&batch_id, &0u32, &10u32);
    assert_eq!(records.len(), 5);
}
