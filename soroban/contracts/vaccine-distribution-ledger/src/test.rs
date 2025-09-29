#![cfg(test)]

use crate::{
    distribution_storage::BatchStatus,
    VaccineDistributionLedger, VaccineDistributionLedgerClient,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

fn create_test_contract(env: &Env) -> VaccineDistributionLedgerClient<'_> {
    VaccineDistributionLedgerClient::new(env, &env.register(VaccineDistributionLedger {}, ()))
}

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);

    // Initialize contract
    contract.initialize(&admin);

    // Verify admin can perform admin operations by trying to initialize again
    let result = contract.try_initialize(&admin);
    assert!(result.is_err()); // Should fail as already initialized
}

#[test]
fn test_batch_creation_and_basic_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);

    contract.initialize(&admin);

    // Create a vaccine batch
    let batch_id = String::from_str(&env, "PFIZER_001");
    let vaccine_type = String::from_str(&env, "COVID-19 mRNA");
    let production_date = 1640995200u64; // Jan 1, 2022
    let quantity = 1000u32;
    let expiry_date = 1672531200u64; // Jan 1, 2023

    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &vaccine_type,
        &production_date,
        &quantity,
        &expiry_date,
    );

    // Verify batch was created
    let batch = contract.get_batch(&batch_id);
    assert_eq!(batch.batch_id, batch_id);
    assert_eq!(batch.manufacturer, manufacturer);
    assert_eq!(batch.vaccine_type, vaccine_type);
    assert_eq!(batch.initial_quantity, quantity);
    assert_eq!(batch.current_quantity, quantity);
    assert_eq!(batch.status, BatchStatus::Produced);
}

#[test]
fn test_distribution_logging() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch
    let batch_id = String::from_str(&env, "MODERNA_001");
    let vaccine_type = String::from_str(&env, "COVID-19 mRNA");
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &vaccine_type,
        &1640995200u64,
        &1000u32,
        &1672531200u64,
    );

    // Log distribution
    let destination = String::from_str(&env, "Regional Health Center");
    let quantity = 100u32;
    let temp_log = Some(String::from_str(&env, "2-8°C maintained"));

    contract.log_distribution(
        &batch_id,
        &distributor,
        &destination,
        &quantity,
        &temp_log,
    );

    // Check batch inventory updated
    let remaining = contract.inventory_check(&batch_id);
    assert_eq!(remaining, 900);

    // Check batch status changed to InTransit
    let batch = contract.get_batch(&batch_id);
    assert_eq!(batch.status, BatchStatus::InTransit);
}

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
fn test_batch_status_updates() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch
    let batch_id = String::from_str(&env, "ASTRAZENECA_001");
    let vaccine_type = String::from_str(&env, "COVID-19 Viral Vector");

    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &vaccine_type,
        &1640995200u64,
        &800u32,
        &1672531200u64,
    );

    // Update status to Distributed
    let notes = Some(String::from_str(&env, "Distributed to regional centers"));
    
    contract.update_batch_status(
        &batch_id,
        &manufacturer,
        &BatchStatus::Distributed,
        &notes,
    );

    // Verify status changed
    let batch = contract.get_batch(&batch_id);
    assert_eq!(batch.status, BatchStatus::Distributed);
    assert_eq!(batch.notes, notes);
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
fn test_error_conditions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);

    contract.initialize(&admin);

    // Test duplicate batch creation
    let batch_id = String::from_str(&env, "DUPLICATE_TEST");
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &500u32,
        &1672531200u64,
    );

    // Try to create same batch again
    let result = contract.try_initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &500u32,
        &1672531200u64,
    );
    assert!(result.is_err());

    // Test invalid quantity
    let result = contract.try_initialize_batch(
        &String::from_str(&env, "INVALID_QTY"),
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &0u32, // Invalid quantity
        &1672531200u64,
    );
    assert!(result.is_err());

    // Test non-existent batch query
    let result = contract.try_get_batch(&String::from_str(&env, "NON_EXISTENT"));
    assert!(result.is_err());
}

#[test]
fn test_insufficient_inventory_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);

    contract.initialize(&admin);

    // Create small batch
    let batch_id = String::from_str(&env, "SMALL_BATCH");
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &50u32,
        &1672531200u64,
    );

    // Try to distribute more than available
    let result = contract.try_log_distribution(
        &batch_id,
        &distributor,
        &String::from_str(&env, "Hospital"),
        &100u32, // More than the 50 available
        &None,
    );
    assert!(result.is_err());
}

#[test]
fn test_expired_batch_handling() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch with past expiry date
    let batch_id = String::from_str(&env, "EXPIRED_BATCH");
    let production_date = 1640995200u64; // Jan 1, 2022
    let expiry_date = 1641081600u64;     // Jan 2, 2022 (very short expiry)
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &production_date,
        &500u32,
        &expiry_date,
    );

    // Fast forward time past expiry
    env.ledger().with_mut(|li| {
        li.timestamp = 1641168000; // Jan 3, 2022
    });

    // Try to distribute expired batch
    let result = contract.try_log_distribution(
        &batch_id,
        &distributor,
        &String::from_str(&env, "Hospital"),
        &10u32,
        &None,
    );
    assert!(result.is_err());
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